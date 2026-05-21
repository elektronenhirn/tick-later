use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tauri::{AppHandle, Manager, State};

mod terminal;
use terminal::{TerminalManager, TerminalState};

mod github_sync;

#[cfg(target_os = "linux")]
mod virtual_desktop;

/// An application that can be launched as part of a workspace setup
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceApp {
    #[serde(default)]
    pub command: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub revisit_at: DateTime<Utc>,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub virtual_desktop: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_apps: Option<Vec<WorkspaceApp>>,
}

impl Todo {
    pub fn new(
        title: String,
        description: Option<String>,
        revisit_at: DateTime<Utc>,
        virtual_desktop: Option<u32>,
        color: Option<String>,
        workspace_apps: Option<Vec<WorkspaceApp>>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description,
            revisit_at,
            completed: false,
            created_at: Utc::now(),
            completed_at: None,
            virtual_desktop,
            color,
            workspace_apps,
        }
    }
}

// ── GitHub sync config (persisted to settings.json) ───────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubSyncConfig {
    pub enabled: bool,
    pub token: String,
    pub repo: String,
    #[serde(default = "default_file_path")]
    pub file_path: String,
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,
}

fn default_file_path() -> String { "todos.json".to_string() }
fn default_poll_interval() -> u64 { 120 }

impl Default for GitHubSyncConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            token: String::new(),
            repo: String::new(),
            file_path: default_file_path(),
            poll_interval_secs: default_poll_interval(),
        }
    }
}

// State to track the current database file path
pub struct DatabasePath(pub Mutex<Option<PathBuf>>);

// State for GitHub sync ETag cache, last-known remote SHA, and dirty flag
pub struct GitHubSyncState {
    pub etag: Mutex<Option<String>>,
    pub remote_sha: Mutex<String>,  // blob SHA of the file as last fetched/pushed
    pub local_dirty: Mutex<bool>,   // true when local todos changed since last push
}

impl GitHubSyncState {
    fn new() -> Self {
        Self {
            etag: Mutex::new(None),
            remote_sha: Mutex::new(String::new()),
            local_dirty: Mutex::new(false),
        }
    }
}

fn get_default_todos_file_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;

    Ok(app_data_dir.join("todos.json"))
}

fn get_todos_file_path(app_handle: &AppHandle, db_path: &State<DatabasePath>) -> Result<PathBuf, String> {
    let path_guard = db_path.0.lock().map_err(|e| format!("Failed to lock database path: {}", e))?;

    match &*path_guard {
        Some(path) => Ok(path.clone()),
        None => get_default_todos_file_path(app_handle),
    }
}

fn load_todos_from_file(app_handle: &AppHandle, db_path: &State<DatabasePath>) -> Result<Vec<Todo>, String> {
    let file_path = get_todos_file_path(app_handle, db_path)?;

    if !file_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read todos file: {}", e))?;

    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse todos file: {}", e))
}

fn save_todos_to_file(app_handle: &AppHandle, db_path: &State<DatabasePath>, todos: &[Todo]) -> Result<(), String> {
    let file_path = get_todos_file_path(app_handle, db_path)?;
    let content = serde_json::to_string_pretty(todos)
        .map_err(|e| format!("Failed to serialize todos: {}", e))?;

    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write todos file: {}", e))?;

    // Mark that local todos differ from the last-pushed state
    if let Some(sync_state) = app_handle.try_state::<GitHubSyncState>() {
        if let Ok(mut dirty) = sync_state.local_dirty.lock() {
            *dirty = true;
        }
    }

    Ok(())
}

#[tauri::command]
fn load_todos(app_handle: AppHandle, db_path: State<DatabasePath>) -> Result<Vec<Todo>, String> {
    load_todos_from_file(&app_handle, &db_path)
}

#[derive(Debug, Deserialize)]
struct SaveTodoPayload {
    title: String,
    description: Option<String>,
    revisit_at: DateTime<Utc>,
    virtual_desktop: Option<u32>,
    color: Option<String>,
    workspace_apps: Option<Vec<WorkspaceApp>>,
}

#[tauri::command]
fn save_todo(
    app_handle: AppHandle,
    db_path: State<DatabasePath>,
    payload: SaveTodoPayload,
) -> Result<Todo, String> {
    let mut todos = load_todos_from_file(&app_handle, &db_path)?;
    let new_todo = Todo::new(
        payload.title,
        payload.description,
        payload.revisit_at,
        payload.virtual_desktop,
        payload.color,
        payload.workspace_apps,
    );

    todos.push(new_todo.clone());
    save_todos_to_file(&app_handle, &db_path, &todos)?;

    Ok(new_todo)
}

#[tauri::command]
fn toggle_todo_completion(app_handle: AppHandle, db_path: State<DatabasePath>, id: String) -> Result<bool, String> {
    let mut todos = load_todos_from_file(&app_handle, &db_path)?;

    let todo = todos.iter_mut()
        .find(|t| t.id == id)
        .ok_or("Todo not found")?;

    todo.completed = !todo.completed;
    let new_status = todo.completed;

    // Set or clear completed_at timestamp
    if new_status {
        todo.completed_at = Some(Utc::now());
    } else {
        todo.completed_at = None;
    }

    save_todos_to_file(&app_handle, &db_path, &todos)?;
    Ok(new_status)
}

#[derive(Debug, Deserialize)]
struct UpdateTodoPayload {
    id: String,
    title: Option<String>,
    description: Option<String>,
    revisit_at: Option<DateTime<Utc>>,
    clear_description: Option<bool>,
    virtual_desktop: Option<u32>,
    clear_virtual_desktop: Option<bool>,
    color: Option<String>,
    clear_color: Option<bool>,
    workspace_apps: Option<Vec<WorkspaceApp>>,
    clear_workspace_apps: Option<bool>,
}

#[tauri::command]
fn update_todo(
    app_handle: AppHandle,
    db_path: State<DatabasePath>,
    payload: UpdateTodoPayload,
) -> Result<Todo, String> {
    let mut todos = load_todos_from_file(&app_handle, &db_path)?;

    let todo = todos.iter_mut()
        .find(|t| t.id == payload.id)
        .ok_or("Todo not found")?;

    if let Some(new_title) = payload.title {
        todo.title = new_title;
    }

    if payload.clear_description.unwrap_or(false) {
        todo.description = None;
    } else if let Some(new_description) = payload.description {
        todo.description = Some(new_description);
    }

    if let Some(new_revisit_at) = payload.revisit_at {
        todo.revisit_at = new_revisit_at;
    }

    if payload.clear_virtual_desktop.unwrap_or(false) {
        todo.virtual_desktop = None;
    } else if payload.virtual_desktop.is_some() {
        todo.virtual_desktop = payload.virtual_desktop;
    }

    if payload.clear_color.unwrap_or(false) {
        todo.color = None;
    } else if payload.color.is_some() {
        todo.color = payload.color;
    }

    if payload.clear_workspace_apps.unwrap_or(false) {
        todo.workspace_apps = None;
    } else if payload.workspace_apps.is_some() {
        todo.workspace_apps = payload.workspace_apps;
    }

    let updated_todo = todo.clone();
    save_todos_to_file(&app_handle, &db_path, &todos)?;

    Ok(updated_todo)
}

#[tauri::command]
fn delete_todo(app_handle: AppHandle, db_path: State<DatabasePath>, id: String) -> Result<(), String> {
    let mut todos = load_todos_from_file(&app_handle, &db_path)?;
    todos.retain(|t| t.id != id);
    save_todos_to_file(&app_handle, &db_path, &todos)
}

#[tauri::command]
fn switch_database(db_path: State<DatabasePath>, path: String) -> Result<(), String> {
    let new_path = PathBuf::from(&path);

    // Verify the file exists and is readable
    if !new_path.exists() {
        // Create an empty database file
        fs::write(&new_path, "[]")
            .map_err(|e| format!("Failed to create database file: {}", e))?;
    }

    let mut path_guard = db_path.0.lock().map_err(|e| format!("Failed to lock database path: {}", e))?;
    *path_guard = Some(new_path);

    Ok(())
}

#[tauri::command]
fn get_current_database_path(app_handle: AppHandle, db_path: State<DatabasePath>) -> Result<String, String> {
    let file_path = get_todos_file_path(&app_handle, &db_path)?;
    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
fn create_new_database(db_path: State<DatabasePath>, path: String) -> Result<(), String> {
    let new_path = PathBuf::from(&path);

    // Create an empty database file
    fs::write(&new_path, "[]")
        .map_err(|e| format!("Failed to create database file: {}", e))?;

    let mut path_guard = db_path.0.lock().map_err(|e| format!("Failed to lock database path: {}", e))?;
    *path_guard = Some(new_path);

    Ok(())
}

// Terminal commands
#[tauri::command]
fn create_terminal_session(
    app_handle: AppHandle,
    terminal_state: State<TerminalState>,
    todo_id: String,
    working_dir: Option<String>,
) -> Result<(), String> {
    let mut manager = terminal_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock terminal state: {}", e))?;

    manager.create_session(todo_id, app_handle, working_dir)
}

#[tauri::command]
fn write_to_terminal(
    terminal_state: State<TerminalState>,
    todo_id: String,
    data: String,
) -> Result<(), String> {
    let mut manager = terminal_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock terminal state: {}", e))?;

    manager.write_to_session(&todo_id, &data)
}

#[tauri::command]
fn resize_terminal(
    terminal_state: State<TerminalState>,
    todo_id: String,
    rows: u16,
    cols: u16,
) -> Result<(), String> {
    let mut manager = terminal_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock terminal state: {}", e))?;

    manager.resize_session(&todo_id, rows, cols)
}

#[tauri::command]
fn close_terminal_session(
    terminal_state: State<TerminalState>,
    todo_id: String,
) -> Result<(), String> {
    let mut manager = terminal_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock terminal state: {}", e))?;

    manager.close_session(&todo_id)
}

#[tauri::command]
fn has_terminal_session(
    terminal_state: State<TerminalState>,
    todo_id: String,
) -> Result<bool, String> {
    let manager = terminal_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock terminal state: {}", e))?;

    Ok(manager.has_session(&todo_id))
}

/// Check if a terminal session is busy (executing a command)
/// Returns: Some(true) if busy, Some(false) if idle, None if unknown/unsupported
#[tauri::command]
fn is_terminal_busy(
    terminal_state: State<TerminalState>,
    todo_id: String,
) -> Result<Option<bool>, String> {
    let manager = terminal_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock terminal state: {}", e))?;

    Ok(manager.is_session_busy(&todo_id))
}

// Virtual desktop commands (Linux only)
#[cfg(target_os = "linux")]
#[tauri::command]
fn get_virtual_desktop_info() -> Result<virtual_desktop::DesktopInfo, String> {
    virtual_desktop::get_desktop_info()
}

#[cfg(target_os = "linux")]
#[tauri::command]
fn switch_virtual_desktop(desktop: u32) -> Result<(), String> {
    virtual_desktop::switch_to_desktop(desktop)
}

#[cfg(target_os = "linux")]
#[tauri::command]
fn switch_to_next_virtual_desktop() -> Result<u32, String> {
    virtual_desktop::switch_to_next_desktop()
}

#[cfg(target_os = "linux")]
#[tauri::command]
fn switch_to_previous_virtual_desktop() -> Result<u32, String> {
    virtual_desktop::switch_to_previous_desktop()
}

#[cfg(target_os = "linux")]
#[tauri::command]
fn set_virtual_desktop_count(count: u32) -> Result<(), String> {
    virtual_desktop::set_desktop_count(count)
}

#[cfg(target_os = "linux")]
#[tauri::command]
fn move_active_window_to_desktop(desktop: u32) -> Result<(), String> {
    virtual_desktop::move_active_window_to_desktop(desktop)
}

/// Open a URL on a specific desktop.
///
/// Algorithm:
/// - If there is NO browser window on the target desktop:
///   Switch to the target desktop, then open the URL in a NEW browser window
/// - If there IS a browser window on the target desktop:
///   Switch to the target desktop, then open the URL in the existing browser
#[cfg(target_os = "linux")]
#[tauri::command]
fn open_url_on_desktop(url: String, desktop: u32) -> Result<(), String> {
    use std::process::Command;

    // Ensure the target desktop exists
    let total = virtual_desktop::get_desktop_count()?;
    if desktop >= total {
        virtual_desktop::set_desktop_count(desktop + 1)?;
    }

    // Check if there's already a browser window on the target desktop
    let browser_exists = virtual_desktop::find_browser_on_desktop(desktop)?.is_some();

    // Switch to the target desktop first
    virtual_desktop::switch_to_desktop(desktop)?;

    if browser_exists {
        // Browser exists on target desktop - open URL in existing browser (new tab)
        Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    } else {
        // No browser on target desktop - open URL in a NEW browser window
        let browser = virtual_desktop::detect_default_browser()?;

        Command::new(&browser)
            .arg("--new-window")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open new browser window: {}", e))?;
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct LaunchWorkspacePayload {
    apps: Vec<WorkspaceApp>,
    virtual_desktop: Option<u32>,
}

/// Launch workspace applications.
/// On Linux with a virtual desktop specified, switches to that desktop first.
#[tauri::command]
fn launch_workspace(payload: LaunchWorkspacePayload) -> Result<(), String> {
    use std::process::Command;

    let virtual_desktop = payload.virtual_desktop;
    let apps = payload.apps;

    // On Linux, switch to the virtual desktop first if specified
    #[cfg(target_os = "linux")]
    if let Some(desktop) = virtual_desktop {
        // Ensure the desktop exists
        let total = virtual_desktop::get_desktop_count()?;
        if desktop >= total {
            virtual_desktop::set_desktop_count(desktop + 1)?;
        }
        virtual_desktop::switch_to_desktop(desktop)?;
    }

    // Suppress unused variable warning on non-Linux platforms
    #[cfg(not(target_os = "linux"))]
    let _ = virtual_desktop;

    // Launch all applications using the user's shell as a login shell for full PATH access
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string());

    for app in apps {
        if app.command.trim().is_empty() {
            continue;
        }

        // Use the user's shell as a login shell (-l) to source profile and get full PATH
        let mut cmd = Command::new(&shell);
        cmd.arg("-l");
        cmd.arg("-c");
        cmd.arg(&app.command);

        // Set working directory if specified (expand ~ to home directory)
        if let Some(ref dir) = app.working_dir {
            let expanded_dir = if dir.starts_with("~/") {
                if let Some(home) = std::env::var_os("HOME") {
                    std::path::PathBuf::from(home).join(&dir[2..])
                } else {
                    std::path::PathBuf::from(dir)
                }
            } else if dir == "~" {
                std::env::var_os("HOME")
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| std::path::PathBuf::from(dir))
            } else {
                std::path::PathBuf::from(dir)
            };
            cmd.current_dir(expanded_dir);
        }

        cmd.spawn()
            .map_err(|e| format!("Failed to launch '{}': {}", app.command, e))?;
    }

    Ok(())
}

// ── Settings file helpers ─────────────────────────────────────────────────

fn get_settings_file_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    Ok(dir.join("settings.json"))
}

// ── GitHub sync commands ──────────────────────────────────────────────────

#[tauri::command]
fn get_github_sync_config(app_handle: AppHandle) -> Result<GitHubSyncConfig, String> {
    let path = get_settings_file_path(&app_handle)?;
    if !path.exists() {
        return Ok(GitHubSyncConfig::default());
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read settings: {}", e))?;
    if content.trim().is_empty() {
        return Ok(GitHubSyncConfig::default());
    }
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings: {}", e))
}

#[tauri::command]
fn save_github_sync_config(
    app_handle: AppHandle,
    sync_state: State<GitHubSyncState>,
    config: GitHubSyncConfig,
) -> Result<(), String> {
    // Invalidate ETag so the next sync does a full fetch with the new config.
    let mut etag = sync_state.etag.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    *etag = None;

    let path = get_settings_file_path(&app_handle)?;
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    fs::write(&path, content)
        .map_err(|e| format!("Failed to write settings: {}", e))
}

fn build_commit_message(trigger: &str, hostname: &str) -> String {
    format!("Tick Later sync from {hostname}\n\nTrigger: {trigger}")
}

fn get_hostname() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "unknown-host".to_string())
}

/// Fetch from GitHub, merge with local, push merged result back.
/// `trigger` describes what initiated this sync: "manual", "startup", "periodic", or "settings-saved".
/// Returns a SyncReport describing what changed.
#[tauri::command]
async fn sync_with_github(
    app_handle: AppHandle,
    db_path: State<'_, DatabasePath>,
    sync_state: State<'_, GitHubSyncState>,
    trigger: String,
) -> Result<github_sync::SyncReport, String> {
    let config = get_github_sync_config(app_handle.clone())?;

    if !config.enabled {
        return Err("GitHub sync is not enabled".to_string());
    }
    if config.token.is_empty() || config.repo.is_empty() {
        return Err("GitHub sync is not fully configured".to_string());
    }

    let hostname = get_hostname();

    let etag = {
        let guard = sync_state.etag.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        guard.clone()
    };

    let fetch_result = github_sync::fetch_remote(&config, etag.as_deref()).await?;

    match fetch_result {
        None => {
            // 304: remote hasn't changed — push local edits if any
            let is_dirty = *sync_state.local_dirty.lock()
                .map_err(|e| format!("Lock error: {}", e))?;

            if !is_dirty {
                return Ok(github_sync::SyncReport {
                    pulled: 0, pushed: 0, conflicts_resolved: 0, unchanged: true,
                });
            }

            let cached_sha = sync_state.remote_sha.lock()
                .map_err(|e| format!("Lock error: {}", e))?.clone();

            let local_todos = load_todos_from_file(&app_handle, &db_path)?;
            let pushed = local_todos.len();
            let msg = build_commit_message("local-change", &hostname);
            let new_sha = github_sync::push_remote(&config, &local_todos, &cached_sha, &msg).await?;

            // Invalidate ETag so the next poll does a full fetch and picks up the new SHA
            *sync_state.etag.lock().map_err(|e| format!("Lock error: {}", e))? = None;
            *sync_state.remote_sha.lock().map_err(|e| format!("Lock error: {}", e))? = new_sha;
            *sync_state.local_dirty.lock().map_err(|e| format!("Lock error: {}", e))? = false;

            Ok(github_sync::SyncReport {
                pulled: 0, pushed, conflicts_resolved: 0, unchanged: false,
            })
        }
        Some((remote_todos, sha, new_etag)) => {
            let local_todos = load_todos_from_file(&app_handle, &db_path)?;
            let (merged, conflicts, new_from_remote, local_only) =
                github_sync::merge(local_todos, remote_todos);

            let has_remote_changes = new_from_remote > 0 || conflicts > 0;
            let needs_push = has_remote_changes || local_only > 0;

            // Apply remote changes to local file only when the remote brought something new
            if has_remote_changes {
                save_todos_to_file(&app_handle, &db_path, &merged)?;
            }

            let new_sha = if needs_push {
                let msg = build_commit_message(&trigger, &hostname);
                github_sync::push_remote(&config, &merged, &sha, &msg).await?
            } else {
                sha  // nothing to push; cache the current remote SHA for future dirty pushes
            };

            *sync_state.etag.lock().map_err(|e| format!("Lock error: {}", e))?
                = if new_etag.is_empty() { None } else { Some(new_etag) };
            *sync_state.remote_sha.lock().map_err(|e| format!("Lock error: {}", e))? = new_sha;
            if needs_push {
                *sync_state.local_dirty.lock().map_err(|e| format!("Lock error: {}", e))? = false;
            }

            Ok(github_sync::SyncReport {
                pulled: new_from_remote,
                pushed: if needs_push { merged.len() } else { 0 },
                conflicts_resolved: conflicts,
                unchanged: !needs_push,
            })
        }
    }
}

/// Fetch-only connection test. Returns the number of todos found on GitHub.
#[tauri::command]
async fn test_github_connection(config: GitHubSyncConfig) -> Result<usize, String> {
    if !github_sync::check_repo_exists(&config).await? {
        return Err(format!(
            "Repository '{}' not found. Create it on GitHub first (can be private).",
            config.repo
        ));
    }
    match github_sync::fetch_remote(&config, None).await? {
        Some((todos, _, _)) => Ok(todos.len()),
        None => Ok(0),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(DatabasePath(Mutex::new(None)))
        .manage(TerminalState(Mutex::new(TerminalManager::new())))
        .manage(GitHubSyncState::new())
        .invoke_handler(tauri::generate_handler![
            load_todos,
            save_todo,
            toggle_todo_completion,
            update_todo,
            delete_todo,
            switch_database,
            get_current_database_path,
            create_new_database,
            get_github_sync_config,
            save_github_sync_config,
            sync_with_github,
            test_github_connection,
            create_terminal_session,
            write_to_terminal,
            resize_terminal,
            close_terminal_session,
            has_terminal_session,
            is_terminal_busy,
            #[cfg(target_os = "linux")]
            get_virtual_desktop_info,
            #[cfg(target_os = "linux")]
            switch_virtual_desktop,
            #[cfg(target_os = "linux")]
            switch_to_next_virtual_desktop,
            #[cfg(target_os = "linux")]
            switch_to_previous_virtual_desktop,
            #[cfg(target_os = "linux")]
            set_virtual_desktop_count,
            #[cfg(target_os = "linux")]
            move_active_window_to_desktop,
            #[cfg(target_os = "linux")]
            open_url_on_desktop,
            launch_workspace
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
