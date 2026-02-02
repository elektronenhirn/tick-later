//! Virtual desktop control for Linux using X11 EWMH protocol.
//!
//! This module provides functions to query and switch virtual desktops
//! on Linux systems running X11-compatible window managers.

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{self, AtomEnum, ConnectionExt};
use x11rb::rust_connection::RustConnection;

/// Information about the current virtual desktop state
#[derive(Debug, Clone, serde::Serialize)]
pub struct DesktopInfo {
    /// The current desktop index (0-based)
    pub current: u32,
    /// Total number of desktops
    pub total: u32,
    /// Names of all desktops (if available)
    pub names: Vec<String>,
}

/// Get an interned atom by name
fn get_atom(conn: &RustConnection, name: &str) -> Result<u32, String> {
    conn.intern_atom(false, name.as_bytes())
        .map_err(|e| format!("Failed to send intern_atom request: {}", e))?
        .reply()
        .map_err(|e| format!("Failed to get atom '{}': {}", name, e))
        .map(|r| r.atom)
}

/// Get a u32 property value from a window
fn get_u32_property(conn: &RustConnection, window: u32, atom: u32) -> Result<u32, String> {
    let reply = conn
        .get_property(false, window, atom, AtomEnum::CARDINAL, 0, 1)
        .map_err(|e| format!("Failed to send get_property request: {}", e))?
        .reply()
        .map_err(|e| format!("Failed to get property: {}", e))?;

    if reply.value.len() < 4 {
        return Err("Property value too short".to_string());
    }

    Ok(u32::from_ne_bytes([
        reply.value[0],
        reply.value[1],
        reply.value[2],
        reply.value[3],
    ]))
}

/// Get desktop names from _NET_DESKTOP_NAMES property
fn get_desktop_names(conn: &RustConnection, root: u32) -> Result<Vec<String>, String> {
    let atom = get_atom(conn, "_NET_DESKTOP_NAMES")?;
    let utf8_atom = get_atom(conn, "UTF8_STRING")?;

    let reply = conn
        .get_property(false, root, atom, utf8_atom, 0, 1024)
        .map_err(|e| format!("Failed to send get_property request: {}", e))?
        .reply()
        .map_err(|e| format!("Failed to get desktop names: {}", e))?;

    // Names are null-separated UTF-8 strings
    let names: Vec<String> = reply
        .value
        .split(|&b| b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8_lossy(s).to_string())
        .collect();

    Ok(names)
}

/// Get information about all virtual desktops
pub fn get_desktop_info() -> Result<DesktopInfo, String> {
    let (conn, screen_num) = RustConnection::connect(None)
        .map_err(|e| format!("Failed to connect to X server: {}", e))?;

    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let current_desktop_atom = get_atom(&conn, "_NET_CURRENT_DESKTOP")?;
    let num_desktops_atom = get_atom(&conn, "_NET_NUMBER_OF_DESKTOPS")?;

    let current = get_u32_property(&conn, root, current_desktop_atom)?;
    let total = get_u32_property(&conn, root, num_desktops_atom)?;
    let names = get_desktop_names(&conn, root).unwrap_or_default();

    Ok(DesktopInfo {
        current,
        total,
        names,
    })
}

/// Get the current desktop index (0-based)
#[allow(dead_code)]
pub fn get_current_desktop() -> Result<u32, String> {
    let (conn, screen_num) = RustConnection::connect(None)
        .map_err(|e| format!("Failed to connect to X server: {}", e))?;

    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let atom = get_atom(&conn, "_NET_CURRENT_DESKTOP")?;
    get_u32_property(&conn, root, atom)
}

/// Get the total number of desktops
pub fn get_desktop_count() -> Result<u32, String> {
    let (conn, screen_num) = RustConnection::connect(None)
        .map_err(|e| format!("Failed to connect to X server: {}", e))?;

    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let atom = get_atom(&conn, "_NET_NUMBER_OF_DESKTOPS")?;
    get_u32_property(&conn, root, atom)
}

/// Set the number of desktops
pub fn set_desktop_count(count: u32) -> Result<(), String> {
    let (conn, screen_num) = RustConnection::connect(None)
        .map_err(|e| format!("Failed to connect to X server: {}", e))?;

    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let atom = get_atom(&conn, "_NET_NUMBER_OF_DESKTOPS")?;

    // Send a client message to the root window to request changing the desktop count
    let event = xproto::ClientMessageEvent {
        response_type: xproto::CLIENT_MESSAGE_EVENT,
        format: 32,
        sequence: 0,
        window: root,
        type_: atom,
        data: xproto::ClientMessageData::from([count, 0, 0, 0, 0]),
    };

    conn.send_event(
        false,
        root,
        xproto::EventMask::SUBSTRUCTURE_NOTIFY | xproto::EventMask::SUBSTRUCTURE_REDIRECT,
        event,
    )
    .map_err(|e| format!("Failed to send desktop count event: {}", e))?;

    conn.flush()
        .map_err(|e| format!("Failed to flush X connection: {}", e))?;

    Ok(())
}

/// Switch to a specific desktop by index (0-based).
/// If the desktop doesn't exist, it will be created automatically.
pub fn switch_to_desktop(desktop: u32) -> Result<(), String> {
    // Check if we need to create more desktops
    let total = get_desktop_count()?;
    if desktop >= total {
        // Create enough desktops to include the target
        set_desktop_count(desktop + 1)?;
    }

    let (conn, screen_num) = RustConnection::connect(None)
        .map_err(|e| format!("Failed to connect to X server: {}", e))?;

    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let atom = get_atom(&conn, "_NET_CURRENT_DESKTOP")?;

    // Send a client message to the root window to request the desktop switch
    let event = xproto::ClientMessageEvent {
        response_type: xproto::CLIENT_MESSAGE_EVENT,
        format: 32,
        sequence: 0,
        window: root,
        type_: atom,
        data: xproto::ClientMessageData::from([desktop, 0, 0, 0, 0]),
    };

    conn.send_event(
        false,
        root,
        xproto::EventMask::SUBSTRUCTURE_NOTIFY | xproto::EventMask::SUBSTRUCTURE_REDIRECT,
        event,
    )
    .map_err(|e| format!("Failed to send desktop switch event: {}", e))?;

    conn.flush()
        .map_err(|e| format!("Failed to flush X connection: {}", e))?;

    Ok(())
}

/// Switch to the next desktop (wraps around)
pub fn switch_to_next_desktop() -> Result<u32, String> {
    let info = get_desktop_info()?;
    let next = (info.current + 1) % info.total;
    switch_to_desktop(next)?;
    Ok(next)
}

/// Switch to the previous desktop (wraps around)
pub fn switch_to_previous_desktop() -> Result<u32, String> {
    let info = get_desktop_info()?;
    let prev = if info.current == 0 {
        info.total - 1
    } else {
        info.current - 1
    };
    switch_to_desktop(prev)?;
    Ok(prev)
}

/// Get the currently active (focused) window
pub fn get_active_window() -> Result<u32, String> {
    let (conn, screen_num) = RustConnection::connect(None)
        .map_err(|e| format!("Failed to connect to X server: {}", e))?;

    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let atom = get_atom(&conn, "_NET_ACTIVE_WINDOW")?;

    let reply = conn
        .get_property(false, root, atom, AtomEnum::WINDOW, 0, 1)
        .map_err(|e| format!("Failed to send get_property request: {}", e))?
        .reply()
        .map_err(|e| format!("Failed to get active window: {}", e))?;

    if reply.value.len() < 4 {
        return Err("No active window".to_string());
    }

    Ok(u32::from_ne_bytes([
        reply.value[0],
        reply.value[1],
        reply.value[2],
        reply.value[3],
    ]))
}

/// Move a window to a specific desktop
pub fn move_window_to_desktop(window: u32, desktop: u32) -> Result<(), String> {
    // Ensure the target desktop exists
    let total = get_desktop_count()?;
    if desktop >= total {
        set_desktop_count(desktop + 1)?;
    }

    let (conn, screen_num) = RustConnection::connect(None)
        .map_err(|e| format!("Failed to connect to X server: {}", e))?;

    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let atom = get_atom(&conn, "_NET_WM_DESKTOP")?;

    // Send a client message to move the window to the specified desktop
    // Source indication: 2 = pager/direct user action
    let event = xproto::ClientMessageEvent {
        response_type: xproto::CLIENT_MESSAGE_EVENT,
        format: 32,
        sequence: 0,
        window,
        type_: atom,
        data: xproto::ClientMessageData::from([desktop, 2, 0, 0, 0]),
    };

    conn.send_event(
        false,
        root,
        xproto::EventMask::SUBSTRUCTURE_NOTIFY | xproto::EventMask::SUBSTRUCTURE_REDIRECT,
        event,
    )
    .map_err(|e| format!("Failed to send window move event: {}", e))?;

    conn.flush()
        .map_err(|e| format!("Failed to flush X connection: {}", e))?;

    Ok(())
}

/// Move the currently active window to a specific desktop
pub fn move_active_window_to_desktop(desktop: u32) -> Result<(), String> {
    let window = get_active_window()?;
    move_window_to_desktop(window, desktop)
}

/// Get list of all client windows
pub fn get_all_windows() -> Result<Vec<u32>, String> {
    let (conn, screen_num) = RustConnection::connect(None)
        .map_err(|e| format!("Failed to connect to X server: {}", e))?;

    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let atom = get_atom(&conn, "_NET_CLIENT_LIST")?;

    let reply = conn
        .get_property(false, root, atom, AtomEnum::WINDOW, 0, 1024)
        .map_err(|e| format!("Failed to send get_property request: {}", e))?
        .reply()
        .map_err(|e| format!("Failed to get client list: {}", e))?;

    // Parse window IDs from the reply (array of 32-bit values)
    let windows: Vec<u32> = reply
        .value
        .chunks_exact(4)
        .map(|chunk| u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    Ok(windows)
}

/// Get the desktop a window is on
pub fn get_window_desktop(window: u32) -> Result<u32, String> {
    let (conn, _) = RustConnection::connect(None)
        .map_err(|e| format!("Failed to connect to X server: {}", e))?;

    let atom = get_atom(&conn, "_NET_WM_DESKTOP")?;

    let reply = conn
        .get_property(false, window, atom, AtomEnum::CARDINAL, 0, 1)
        .map_err(|e| format!("Failed to send get_property request: {}", e))?
        .reply()
        .map_err(|e| format!("Failed to get window desktop: {}", e))?;

    if reply.value.len() < 4 {
        return Err("Window has no desktop property".to_string());
    }

    Ok(u32::from_ne_bytes([
        reply.value[0],
        reply.value[1],
        reply.value[2],
        reply.value[3],
    ]))
}

/// Get the WM_CLASS of a window (returns instance and class names)
pub fn get_window_class(window: u32) -> Result<(String, String), String> {
    let (conn, _) = RustConnection::connect(None)
        .map_err(|e| format!("Failed to connect to X server: {}", e))?;

    let reply = conn
        .get_property(false, window, xproto::AtomEnum::WM_CLASS, xproto::AtomEnum::STRING, 0, 256)
        .map_err(|e| format!("Failed to send get_property request: {}", e))?
        .reply()
        .map_err(|e| format!("Failed to get WM_CLASS: {}", e))?;

    // WM_CLASS contains two null-terminated strings: instance name and class name
    let parts: Vec<&[u8]> = reply.value.split(|&b| b == 0).collect();

    let instance = parts.first()
        .map(|s| String::from_utf8_lossy(s).to_string())
        .unwrap_or_default();
    let class = parts.get(1)
        .map(|s| String::from_utf8_lossy(s).to_string())
        .unwrap_or_default();

    Ok((instance, class))
}

/// Known browser class names (lowercase for comparison)
const BROWSER_CLASSES: &[&str] = &[
    "firefox",
    "firefox-esr",
    "chromium",
    "chromium-browser",
    "google-chrome",
    "brave-browser",
    "vivaldi",
    "opera",
    "microsoft-edge",
    "epiphany",
    "webkit",
    "navigator",  // Firefox sometimes uses this
];

/// Check if a window is a browser based on its WM_CLASS
pub fn is_browser_window(window: u32) -> bool {
    if let Ok((instance, class)) = get_window_class(window) {
        let instance_lower = instance.to_lowercase();
        let class_lower = class.to_lowercase();

        BROWSER_CLASSES.iter().any(|&browser| {
            instance_lower.contains(browser) || class_lower.contains(browser)
        })
    } else {
        false
    }
}

/// Find browser windows on a specific desktop
pub fn find_browser_on_desktop(desktop: u32) -> Result<Option<u32>, String> {
    let windows = get_all_windows()?;

    for window in windows {
        // Check if window is on the target desktop
        if let Ok(win_desktop) = get_window_desktop(window) {
            if win_desktop == desktop && is_browser_window(window) {
                return Ok(Some(window));
            }
        }
    }

    Ok(None)
}

/// Detect the default browser command
pub fn detect_default_browser() -> Result<String, String> {
    use std::process::Command;

    // Try xdg-settings to get default browser
    let output = Command::new("xdg-settings")
        .args(["get", "default-web-browser"])
        .output()
        .map_err(|e| format!("Failed to run xdg-settings: {}", e))?;

    let desktop_file = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Map common .desktop files to browser commands
    let browser_cmd = if desktop_file.contains("firefox") {
        "firefox"
    } else if desktop_file.contains("chromium") {
        "chromium"
    } else if desktop_file.contains("google-chrome") {
        "google-chrome"
    } else if desktop_file.contains("brave") {
        "brave-browser"
    } else if desktop_file.contains("vivaldi") {
        "vivaldi"
    } else if desktop_file.contains("opera") {
        "opera"
    } else if desktop_file.contains("edge") {
        "microsoft-edge"
    } else {
        // Fallback: try to find a browser in PATH
        for browser in &["firefox", "chromium", "google-chrome", "chromium-browser"] {
            if Command::new("which")
                .arg(browser)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                return Ok(browser.to_string());
            }
        }
        return Err("Could not detect default browser".to_string());
    };

    Ok(browser_cmd.to_string())
}
