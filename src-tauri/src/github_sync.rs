use std::collections::{HashMap, HashSet};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use crate::Todo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncReport {
    pub pulled: usize,
    pub pushed: usize,
    pub conflicts_resolved: usize,
    pub unchanged: bool,
}

#[derive(Deserialize)]
struct GitHubContentsResponse {
    content: String,
    sha: String,
}

#[derive(Serialize)]
struct GitHubPutBody {
    message: String,
    content: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    sha: String,
}

#[derive(Deserialize)]
struct GitHubPutContentField {
    sha: String,
}

#[derive(Deserialize)]
struct GitHubPutResponse {
    content: GitHubPutContentField,
}

fn normalize_repo(repo: &str) -> &str {
    repo.trim()
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/")
        .trim_end_matches('/')
        .trim_end_matches(".git")
}

fn api_url(repo: &str, file_path: &str) -> String {
    format!("https://api.github.com/repos/{}/contents/{}", normalize_repo(repo), file_path)
}

fn build_request(
    method: reqwest::Method,
    url: &str,
    token: &str,
) -> reqwest::RequestBuilder {
    reqwest::Client::new()
        .request(method, url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "tick-later")
}

/// Verify the repository itself exists and is accessible.
/// Returns Ok(true) if accessible, Ok(false) if 404, Err for auth/network errors.
pub async fn check_repo_exists(config: &crate::GitHubSyncConfig) -> Result<bool, String> {
    let url = format!("https://api.github.com/repos/{}", normalize_repo(&config.repo));
    let response = build_request(reqwest::Method::GET, &url, &config.token)
        .send()
        .await
        .map_err(|e| format!("GitHub request failed: {}", e))?;

    match response.status().as_u16() {
        200 => Ok(true),
        404 => Ok(false),
        401 | 403 => Err("Authentication failed. Check your token and its permissions.".to_string()),
        status => {
            let body = response.text().await.unwrap_or_default();
            Err(format!("GitHub API error {}: {}", status, body))
        }
    }
}

/// Fetch remote todos.
/// Returns:
///   Ok(None)                      — 304 Not Modified (ETag matched)
///   Ok(Some((todos, sha, etag)))  — 200 with content
///   Ok(Some(([], "", "")))        — 404 file not found yet (first push)
///   Err(...)                      — network / API error
pub async fn fetch_remote(
    config: &crate::GitHubSyncConfig,
    etag: Option<&str>,
) -> Result<Option<(Vec<Todo>, String, String)>, String> {
    let url = api_url(&config.repo, &config.file_path);
    let mut req = build_request(reqwest::Method::GET, &url, &config.token);

    if let Some(etag_val) = etag {
        req = req.header("If-None-Match", etag_val);
    }

    let response = req
        .send()
        .await
        .map_err(|e| format!("GitHub request failed: {}", e))?;

    match response.status().as_u16() {
        304 => Ok(None),
        404 => Ok(Some((Vec::new(), String::new(), String::new()))),
        200 => {
            let new_etag = response
                .headers()
                .get("etag")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();

            let data: GitHubContentsResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse GitHub response: {}", e))?;

            let clean: String = data.content.chars().filter(|c| !c.is_whitespace()).collect();
            let decoded = BASE64
                .decode(clean)
                .map_err(|e| format!("Failed to decode file content: {}", e))?;

            let todos: Vec<Todo> = serde_json::from_slice(&decoded)
                .map_err(|e| format!("Failed to parse todos from GitHub: {}", e))?;

            Ok(Some((todos, data.sha, new_etag)))
        }
        status => {
            let body = response.text().await.unwrap_or_default();
            Err(format!("GitHub API error {}: {}", status, body))
        }
    }
}

/// Push todos to GitHub. Pass `sha` = "" for the first push (file doesn't exist yet).
/// Returns the new blob SHA of the written file.
pub async fn push_remote(
    config: &crate::GitHubSyncConfig,
    todos: &[Todo],
    sha: &str,
    commit_message: &str,
) -> Result<String, String> {
    let url = api_url(&config.repo, &config.file_path);

    let json = serde_json::to_string_pretty(todos)
        .map_err(|e| format!("Failed to serialize todos: {}", e))?;

    let body = GitHubPutBody {
        message: commit_message.to_string(),
        content: BASE64.encode(json.as_bytes()),
        sha: sha.to_string(),
    };

    let response = build_request(reqwest::Method::PUT, &url, &config.token)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("GitHub push failed: {}", e))?;

    let status = response.status();
    if status.is_success() {
        let put_response: GitHubPutResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse push response: {}", e))?;
        Ok(put_response.content.sha)
    } else if status.as_u16() == 404 {
        Err(format!(
            "Repository '{}' not found. Make sure it exists on GitHub and your token has write access.",
            config.repo
        ))
    } else {
        let body = response.text().await.unwrap_or_default();
        Err(format!("GitHub push error {}: {}", status.as_u16(), body))
    }
}

/// Merge local and remote todos.
/// - Union by ID; same ID on both sides → keep the one with the later timestamp.
/// - Returns (merged_todos, conflicts_resolved, new_from_remote, local_only).
///   `local_only` counts items that exist locally but are absent from remote (need pushing).
pub fn merge(local: Vec<Todo>, remote: Vec<Todo>) -> (Vec<Todo>, usize, usize, usize) {
    let mut result: Vec<Todo> = Vec::new();
    let mut conflicts = 0usize;
    let mut new_from_remote = 0usize;
    let mut local_only = 0usize;

    let remote_map: HashMap<String, Todo> =
        remote.into_iter().map(|t| (t.id.clone(), t)).collect();

    let mut seen: HashSet<String> = HashSet::new();

    for local_todo in local {
        let id = local_todo.id.clone();
        if let Some(remote_todo) = remote_map.get(&id) {
            let local_ts = max_timestamp(&local_todo);
            let remote_ts = max_timestamp(remote_todo);
            if remote_ts > local_ts {
                result.push(remote_todo.clone());
                conflicts += 1;
            } else {
                result.push(local_todo);
                if remote_ts != local_ts {
                    conflicts += 1;
                }
            }
        } else {
            result.push(local_todo);
            local_only += 1;
        }
        seen.insert(id);
    }

    for (id, remote_todo) in remote_map {
        if !seen.contains(&id) {
            result.push(remote_todo);
            new_from_remote += 1;
        }
    }

    (result, conflicts, new_from_remote, local_only)
}

fn max_timestamp(todo: &Todo) -> DateTime<chrono::Utc> {
    let mut ts = todo.created_at.max(todo.revisit_at);
    if let Some(completed_at) = todo.completed_at {
        ts = ts.max(completed_at);
    }
    ts
}
