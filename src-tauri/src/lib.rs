use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub revisit_at: DateTime<Utc>,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
}

impl Todo {
    pub fn new(title: String, description: Option<String>, revisit_at: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description,
            revisit_at,
            completed: false,
            created_at: Utc::now(),
        }
    }
}

fn get_todos_file_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    
    fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;
    
    Ok(app_data_dir.join("todos.json"))
}

fn load_todos_from_file(app_handle: &AppHandle) -> Result<Vec<Todo>, String> {
    let file_path = get_todos_file_path(app_handle)?;
    
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

fn save_todos_to_file(app_handle: &AppHandle, todos: &[Todo]) -> Result<(), String> {
    let file_path = get_todos_file_path(app_handle)?;
    let content = serde_json::to_string_pretty(todos)
        .map_err(|e| format!("Failed to serialize todos: {}", e))?;
    
    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write todos file: {}", e))
}

#[tauri::command]
fn load_todos(app_handle: AppHandle) -> Result<Vec<Todo>, String> {
    load_todos_from_file(&app_handle)
}

#[tauri::command]
fn save_todo(app_handle: AppHandle, title: String, description: Option<String>, revisit_at: DateTime<Utc>) -> Result<Todo, String> {
    let mut todos = load_todos_from_file(&app_handle)?;
    let new_todo = Todo::new(title, description, revisit_at);
    
    todos.push(new_todo.clone());
    save_todos_to_file(&app_handle, &todos)?;
    
    Ok(new_todo)
}

#[tauri::command]
fn toggle_todo_completion(app_handle: AppHandle, id: String) -> Result<bool, String> {
    let mut todos = load_todos_from_file(&app_handle)?;
    
    let todo = todos.iter_mut()
        .find(|t| t.id == id)
        .ok_or("Todo not found")?;
    
    todo.completed = !todo.completed;
    let new_status = todo.completed;
    
    save_todos_to_file(&app_handle, &todos)?;
    Ok(new_status)
}

#[tauri::command]
fn update_todo(
    app_handle: AppHandle,
    id: String,
    title: Option<String>,
    description: Option<String>,
    revisit_at: Option<DateTime<Utc>>,
    clear_description: Option<bool>,
) -> Result<Todo, String> {
    let mut todos = load_todos_from_file(&app_handle)?;

    let todo = todos.iter_mut()
        .find(|t| t.id == id)
        .ok_or("Todo not found")?;

    if let Some(new_title) = title {
        todo.title = new_title;
    }

    if clear_description.unwrap_or(false) {
        todo.description = None;
    } else if let Some(new_description) = description {
        todo.description = Some(new_description);
    }

    if let Some(new_revisit_at) = revisit_at {
        todo.revisit_at = new_revisit_at;
    }

    let updated_todo = todo.clone();
    save_todos_to_file(&app_handle, &todos)?;

    Ok(updated_todo)
}

#[tauri::command]
fn delete_todo(app_handle: AppHandle, id: String) -> Result<(), String> {
    let mut todos = load_todos_from_file(&app_handle)?;
    todos.retain(|t| t.id != id);
    save_todos_to_file(&app_handle, &todos)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            load_todos,
            save_todo,
            toggle_todo_completion,
            update_todo,
            delete_todo
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
