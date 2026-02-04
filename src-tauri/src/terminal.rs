use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Mutex;
use std::thread;

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

/// Event payload for terminal output
#[derive(Clone, Serialize)]
pub struct TerminalOutput {
    pub todo_id: String,
    pub data: String,
}

/// Event payload for terminal exit
#[derive(Clone, Serialize)]
pub struct TerminalExited {
    pub todo_id: String,
}

/// Manages a single terminal session
pub struct TerminalSession {
    pub writer: Box<dyn Write + Send>,
    pub pty_pair: portable_pty::PtyPair,
    pub shell_pid: Option<u32>,
}

/// Manages all terminal sessions
pub struct TerminalManager {
    sessions: HashMap<String, TerminalSession>,
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn create_session(
        &mut self,
        todo_id: String,
        app_handle: AppHandle,
        working_dir: Option<String>,
    ) -> Result<(), String> {
        // Don't create duplicate sessions
        if self.sessions.contains_key(&todo_id) {
            return Ok(());
        }

        let pty_system = native_pty_system();

        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to open PTY: {}", e))?;

        // Get the user's default shell
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());

        let mut cmd = CommandBuilder::new(&shell);
        cmd.arg("-l"); // Login shell for proper environment

        // Set working directory if provided
        if let Some(dir) = working_dir {
            cmd.cwd(dir);
        }

        let mut child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn shell: {}", e))?;

        // Get the shell's PID before moving child to the monitoring thread
        let shell_pid = child.process_id();

        // Spawn a thread to monitor the child process and emit exit event
        let todo_id_for_child = todo_id.clone();
        let app_handle_for_child = app_handle.clone();
        thread::spawn(move || {
            // Wait for the child process to exit
            let _ = child.wait();
            // Emit exit event
            let _ = app_handle_for_child.emit(
                "terminal-exited",
                TerminalExited {
                    todo_id: todo_id_for_child,
                },
            );
        });

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| format!("Failed to get PTY writer: {}", e))?;

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to get PTY reader: {}", e))?;

        // Spawn a thread to read PTY output and emit events
        let todo_id_clone = todo_id.clone();
        thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        // EOF - shell exited
                        let _ = app_handle.emit(
                            "terminal-exited",
                            TerminalExited {
                                todo_id: todo_id_clone.clone(),
                            },
                        );
                        break;
                    }
                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buffer[..n]).to_string();
                        let _ = app_handle.emit(
                            "terminal-output",
                            TerminalOutput {
                                todo_id: todo_id_clone.clone(),
                                data,
                            },
                        );
                    }
                    Err(_) => {
                        // Error - also treat as exit
                        let _ = app_handle.emit(
                            "terminal-exited",
                            TerminalExited {
                                todo_id: todo_id_clone.clone(),
                            },
                        );
                        break;
                    }
                }
            }
        });

        self.sessions.insert(
            todo_id,
            TerminalSession {
                writer,
                pty_pair: pair,
                shell_pid,
            },
        );

        Ok(())
    }

    pub fn write_to_session(&mut self, todo_id: &str, data: &str) -> Result<(), String> {
        let session = self
            .sessions
            .get_mut(todo_id)
            .ok_or_else(|| "session_not_found".to_string())?;

        session
            .writer
            .write_all(data.as_bytes())
            .map_err(|_| "write_failed".to_string())?;

        session
            .writer
            .flush()
            .map_err(|_| "flush_failed".to_string())?;

        Ok(())
    }

    pub fn resize_session(&mut self, todo_id: &str, rows: u16, cols: u16) -> Result<(), String> {
        let session = self
            .sessions
            .get_mut(todo_id)
            .ok_or_else(|| format!("No terminal session for todo: {}", todo_id))?;

        session
            .pty_pair
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to resize terminal: {}", e))?;

        Ok(())
    }

    pub fn close_session(&mut self, todo_id: &str) -> Result<(), String> {
        self.sessions.remove(todo_id);
        Ok(())
    }

    pub fn has_session(&self, todo_id: &str) -> bool {
        self.sessions.contains_key(todo_id)
    }

    /// Check if the terminal is busy (has child processes running)
    /// Returns None if session doesn't exist, Some(true) if busy, Some(false) if idle
    #[cfg(target_os = "linux")]
    pub fn is_session_busy(&self, todo_id: &str) -> Option<bool> {
        let session = self.sessions.get(todo_id)?;
        let pid = session.shell_pid?;

        // Check if the shell has any child processes
        let children_path = format!("/proc/{}/task/{}/children", pid, pid);
        if let Ok(content) = std::fs::read_to_string(&children_path) {
            Some(!content.trim().is_empty())
        } else {
            // If we can't read the file, assume not busy (process might have exited)
            Some(false)
        }
    }

    #[cfg(not(target_os = "linux"))]
    pub fn is_session_busy(&self, todo_id: &str) -> Option<bool> {
        // On non-Linux platforms, we can't easily check for child processes
        // Return None to indicate unknown state
        if self.sessions.contains_key(todo_id) {
            None
        } else {
            None
        }
    }
}

/// Tauri-managed state for terminal sessions
pub struct TerminalState(pub Mutex<TerminalManager>);
