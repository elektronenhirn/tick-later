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

/// Switch to a specific desktop by index (0-based)
pub fn switch_to_desktop(desktop: u32) -> Result<(), String> {
    let (conn, screen_num) = RustConnection::connect(None)
        .map_err(|e| format!("Failed to connect to X server: {}", e))?;

    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    // Verify the desktop index is valid
    let total = get_desktop_count()?;
    if desktop >= total {
        return Err(format!(
            "Desktop index {} out of range (0-{})",
            desktop,
            total - 1
        ));
    }

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
