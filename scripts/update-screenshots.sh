#!/usr/bin/env bash
# update-screenshots.sh
#
# Updates screenshot-from-demo.json.png and screenshot-with-shadow.png by:
#   1. Refreshing timestamps in demo.json relative to today
#   2. Launching the app with demo.json as the database
#   3. Taking a screenshot of the window
#   4. Generating a drop-shadow version
#
# Requirements:
#   - ImageMagick (import, convert) — for screenshot and shadow
#   - xwininfo                      — to detect when the window is ready
#   - curl                          — to wait for the Vite dev server
#   - Node.js + npm                 — for the dev server and timestamp script
#   - A pre-built debug binary at src-tauri/target/debug/tick-later
#     (run `cargo build` in src-tauri/ if not present)
#
# Usage:
#   cd /path/to/tick-later
#   ./scripts/update-screenshots.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"

APP_BINARY="$REPO_DIR/src-tauri/target/debug/tick-later"
DEMO_JSON="$REPO_DIR/demo.json"
DB_PATH="$HOME/.local/share/net.elektronenhirn.tick-later/todos.json"
BACKUP_PATH="$HOME/.local/share/net.elektronenhirn.tick-later/todos.json.bak"
SCREENSHOT_OUT="$REPO_DIR/screenshot-from-demo.json.png"
SHADOW_OUT="$REPO_DIR/screenshot-with-shadow.png"

WINDOW_TITLE="Tick Later"
DEV_SERVER_PORT=1420
WINDOW_WAIT_TIMEOUT=30  # seconds to wait for window to appear
RENDER_WAIT=3           # seconds to wait after window appears before screenshotting

APP_PID=""
DEV_SERVER_PID=""
XVFB_PID=""

cleanup() {
  if [ -n "$APP_PID" ] && kill -0 "$APP_PID" 2>/dev/null; then
    echo "Killing app (PID $APP_PID)..."
    kill "$APP_PID" 2>/dev/null || true
  fi
  if [ -n "$DEV_SERVER_PID" ] && kill -0 "$DEV_SERVER_PID" 2>/dev/null; then
    echo "Killing Vite dev server (PID $DEV_SERVER_PID)..."
    kill "$DEV_SERVER_PID" 2>/dev/null || true
  fi
  if [ -n "$XVFB_PID" ] && kill -0 "$XVFB_PID" 2>/dev/null; then
    echo "Killing Xvfb (PID $XVFB_PID)..."
    kill "$XVFB_PID" 2>/dev/null || true
  fi
  if [ -f "$BACKUP_PATH" ]; then
    echo "Restoring database backup..."
    mv "$BACKUP_PATH" "$DB_PATH"
  fi
}
trap cleanup EXIT

# ── 1. Check prerequisites ─────────────────────────────────────────────────
echo "Checking prerequisites..."

if [ ! -f "$APP_BINARY" ]; then
  echo "Error: App binary not found at $APP_BINARY"
  echo "Run: cd src-tauri && cargo build"
  exit 1
fi

for cmd in node npm curl import convert xwininfo Xvfb; do
  if ! command -v "$cmd" &>/dev/null; then
    echo "Error: '$cmd' is not installed"
    exit 1
  fi
done

if [ -z "${DISPLAY:-}" ]; then
  echo "Error: No DISPLAY set. Run under a graphical session."
  exit 1
fi

# Virtual framebuffer settings — allows a taller window than the physical display
VDISPLAY=:99
VSCREEN_W=1300
VSCREEN_H=2000

# ── 2. Update demo.json timestamps ────────────────────────────────────────
echo "Updating demo.json timestamps..."
node "$SCRIPT_DIR/update-demo-timestamps.js"

# ── 3. Backup real database and install demo.json ─────────────────────────
DB_DIR="$(dirname "$DB_PATH")"
mkdir -p "$DB_DIR"

if [ -f "$DB_PATH" ]; then
  echo "Backing up existing database to $BACKUP_PATH..."
  cp "$DB_PATH" "$BACKUP_PATH"
fi

echo "Installing demo.json as app database..."
cp "$DEMO_JSON" "$DB_PATH"

# ── 4. Start virtual framebuffer ─────────────────────────────────────────
echo "Starting Xvfb on $VDISPLAY (${VSCREEN_W}x${VSCREEN_H})..."
Xvfb "$VDISPLAY" -screen 0 "${VSCREEN_W}x${VSCREEN_H}x24" &>/dev/null &
XVFB_PID=$!
sleep 1  # brief pause for Xvfb to be ready

# ── 5. Start the Vite dev server ─────────────────────────────────────────
echo "Starting Vite dev server on port $DEV_SERVER_PORT..."
npm --prefix "$REPO_DIR" run dev &>/dev/null &
DEV_SERVER_PID=$!

# Wait until the dev server is accepting connections
echo "Waiting for dev server..."
elapsed=0
while ! curl -sf "http://localhost:$DEV_SERVER_PORT" &>/dev/null; do
  sleep 1
  elapsed=$((elapsed + 1))
  if [ $elapsed -ge 30 ]; then
    echo "Error: Vite dev server did not start within 30s"
    exit 1
  fi
done
echo "Dev server is ready."

# ── 6. Launch the app on the virtual display ──────────────────────────────
echo "Launching $APP_BINARY on $VDISPLAY..."
DISPLAY="$VDISPLAY" "$APP_BINARY" &>/dev/null &
APP_PID=$!

# ── 7. Wait for the window to appear ──────────────────────────────────────
echo "Waiting for '$WINDOW_TITLE' window (up to ${WINDOW_WAIT_TIMEOUT}s)..."
elapsed=0
while ! DISPLAY="$VDISPLAY" xwininfo -name "$WINDOW_TITLE" &>/dev/null; do
  sleep 1
  elapsed=$((elapsed + 1))
  if [ $elapsed -ge $WINDOW_WAIT_TIMEOUT ]; then
    echo "Error: Window '$WINDOW_TITLE' did not appear within ${WINDOW_WAIT_TIMEOUT}s"
    exit 1
  fi
done
echo "Window appeared. Waiting ${RENDER_WAIT}s for rendering..."
sleep "$RENDER_WAIT"

# ── 8. Take the screenshot ────────────────────────────────────────────────
echo "Taking screenshot -> $SCREENSHOT_OUT"
WIN_ID=$(DISPLAY="$VDISPLAY" xwininfo -name "$WINDOW_TITLE" 2>/dev/null \
  | grep "Window id:" \
  | awk '{print $4}')

if [ -z "$WIN_ID" ]; then
  echo "Error: Could not get window ID for '$WINDOW_TITLE'"
  exit 1
fi

# Bring window to front and wait a moment before capturing
DISPLAY="$VDISPLAY" xwininfo -id "$WIN_ID" -stats &>/dev/null
sleep 0.5

DISPLAY="$VDISPLAY" import -window "$WIN_ID" "$SCREENSHOT_OUT"
echo "Screenshot saved: $SCREENSHOT_OUT"

# ── 9. Generate drop-shadow version ──────────────────────────────────────
echo "Generating drop-shadow version -> $SHADOW_OUT"
convert "$SCREENSHOT_OUT" \
  \( +clone -background black -shadow 60x20+0+0 \) \
  +swap -background none -layers merge +repage \
  "$SHADOW_OUT"
echo "Shadow screenshot saved: $SHADOW_OUT"

echo ""
echo "Done!"
echo "  $SCREENSHOT_OUT"
echo "  $SHADOW_OUT"
