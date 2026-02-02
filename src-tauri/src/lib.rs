use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tauri::{AppHandle, Manager, State};

mod terminal;
use terminal::{TerminalManager, TerminalState};

#[cfg(target_os = "linux")]
mod virtual_desktop;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub revisit_at: DateTime<Utc>,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub virtual_desktop: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

impl Todo {
    pub fn new(title: String, description: Option<String>, revisit_at: DateTime<Utc>, virtual_desktop: Option<u32>, color: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description,
            revisit_at,
            completed: false,
            created_at: Utc::now(),
            virtual_desktop,
            color,
        }
    }
}

// State to track the current database file path
pub struct DatabasePath(pub Mutex<Option<PathBuf>>);

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
        .map_err(|e| format!("Failed to write todos file: {}", e))
}

#[tauri::command]
fn load_todos(app_handle: AppHandle, db_path: State<DatabasePath>) -> Result<Vec<Todo>, String> {
    load_todos_from_file(&app_handle, &db_path)
}

#[tauri::command]
fn save_todo(app_handle: AppHandle, db_path: State<DatabasePath>, title: String, description: Option<String>, revisit_at: DateTime<Utc>, virtual_desktop: Option<u32>, color: Option<String>) -> Result<Todo, String> {
    let mut todos = load_todos_from_file(&app_handle, &db_path)?;
    let new_todo = Todo::new(title, description, revisit_at, virtual_desktop, color);

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

    save_todos_to_file(&app_handle, &db_path, &todos)?;
    Ok(new_status)
}

#[tauri::command]
fn update_todo(
    app_handle: AppHandle,
    db_path: State<DatabasePath>,
    id: String,
    title: Option<String>,
    description: Option<String>,
    revisit_at: Option<DateTime<Utc>>,
    clear_description: Option<bool>,
    virtual_desktop: Option<u32>,
    clear_virtual_desktop: Option<bool>,
    color: Option<String>,
    clear_color: Option<bool>,
) -> Result<Todo, String> {
    let mut todos = load_todos_from_file(&app_handle, &db_path)?;

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

    if clear_virtual_desktop.unwrap_or(false) {
        todo.virtual_desktop = None;
    } else if virtual_desktop.is_some() {
        todo.virtual_desktop = virtual_desktop;
    }

    if clear_color.unwrap_or(false) {
        todo.color = None;
    } else if color.is_some() {
        todo.color = color;
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(DatabasePath(Mutex::new(None)))
        .manage(TerminalState(Mutex::new(TerminalManager::new())))
        .invoke_handler(tauri::generate_handler![
            load_todos,
            save_todo,
            toggle_todo_completion,
            update_todo,
            delete_todo,
            switch_database,
            get_current_database_path,
            create_new_database,
            create_terminal_session,
            write_to_terminal,
            resize_terminal,
            close_terminal_session,
            has_terminal_session,
            #[cfg(target_os = "linux")]
            get_virtual_desktop_info,
            #[cfg(target_os = "linux")]
            switch_virtual_desktop,
            #[cfg(target_os = "linux")]
            switch_to_next_virtual_desktop,
            #[cfg(target_os = "linux")]
            switch_to_previous_virtual_desktop,
            #[cfg(target_os = "linux")]
            set_virtual_desktop_count
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
