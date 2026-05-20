#!/usr/bin/env bash
# update-screenshots.sh
#
# Updates screenshot-from-demo.json.png and screenshot-with-shadow.png by:
#   1. Refreshing timestamps in demo.json relative to today
#   2. Launching the app with demo.json as the database (isolated HOME —
#      the production database is never touched)
#   3. Taking a screenshot of the window
#   4. Generating a drop-shadow version
#
# Requirements:
#   - ImageMagick (import, convert) — for screenshot and shadow
#   - xwininfo                      — to detect when the window is ready
#   - curl                          — to wait for the Vite dev server
#   - Node.js + npm                 — for the dev server and timestamp script
#   - Xvfb                         — virtual framebuffer (any display size)
#   - xdotool                      — window geometry helpers
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
SCREENSHOT_OUT="$REPO_DIR/docs/screenshot-from-demo.json.png"
SHADOW_OUT="$REPO_DIR/docs/screenshot-with-shadow.png"

WINDOW_TITLE="Tick Later"
DEV_SERVER_PORT=1420
WINDOW_WAIT_TIMEOUT=30  # seconds to wait for window to appear
RENDER_WAIT=3           # seconds to wait after window appears before screenshotting

# Virtual framebuffer settings — allows a taller window than the physical display
VDISPLAY=:99
VSCREEN_W=1300
VSCREEN_H=2000

APP_PID=""
DEV_SERVER_PID=""
XVFB_PID=""
TEMP_HOME=""

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
  if [ -n "$TEMP_HOME" ] && [ -d "$TEMP_HOME" ]; then
    rm -rf "$TEMP_HOME"
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

for cmd in node npm curl import convert xwininfo Xvfb xdotool; do
  if ! command -v "$cmd" &>/dev/null; then
    echo "Error: '$cmd' is not installed"
    exit 1
  fi
done

if [ -z "${DISPLAY:-}" ]; then
  echo "Error: No DISPLAY set. Run under a graphical session."
  exit 1
fi

# ── 2. Update demo.json timestamps ────────────────────────────────────────
echo "Updating demo.json timestamps..."
node "$SCRIPT_DIR/update-demo-timestamps.js"

# ── 3. Set up isolated HOME with demo database ────────────────────────────
# The app is launched with HOME pointing at a temp directory so the production
# database is never read or written.
TEMP_HOME="$(mktemp -d)"
DB_DIR="$TEMP_HOME/.local/share/net.elektronenhirn.tick-later"
mkdir -p "$DB_DIR"
cp "$DEMO_JSON" "$DB_DIR/todos.json"
echo "Using isolated HOME: $TEMP_HOME"

# ── 4. Start virtual framebuffer ─────────────────────────────────────────
# Kill any stale Xvfb on our display number before starting a fresh one.
STALE_XVFB=$(pgrep -f "Xvfb ${VDISPLAY} " || true)
if [ -n "$STALE_XVFB" ]; then
  echo "Killing stale Xvfb on $VDISPLAY (PID $STALE_XVFB)..."
  kill "$STALE_XVFB" 2>/dev/null || true
  sleep 0.5
fi

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

# ── 6. Launch the app on the virtual display with isolated HOME ───────────
echo "Launching $APP_BINARY on $VDISPLAY..."
DISPLAY="$VDISPLAY" HOME="$TEMP_HOME" "$APP_BINARY" &>/dev/null &
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

# `import -window WIN_ID` only captures WebKit's painted area (content height),
# not the full X11 window. Capture the full virtual display instead and crop to
# the window's position + actual geometry so we always get the complete window.
WIN_INFO=$(DISPLAY="$VDISPLAY" xwininfo -id "$WIN_ID" 2>/dev/null)
WIN_X=$(echo "$WIN_INFO" | grep "Absolute upper-left X:" | awk '{print $NF}')
WIN_Y=$(echo "$WIN_INFO" | grep "Absolute upper-left Y:" | awk '{print $NF}')
WIN_W=$(echo "$WIN_INFO" | grep "  Width:"  | awk '{print $NF}')
WIN_H=$(echo "$WIN_INFO" | grep "  Height:" | awk '{print $NF}')
echo "Window geometry: ${WIN_W}x${WIN_H}+${WIN_X}+${WIN_Y}"

DISPLAY="$VDISPLAY" import -window root /tmp/tick_later_display.png
convert /tmp/tick_later_display.png \
  -crop "${WIN_W}x${WIN_H}+${WIN_X}+${WIN_Y}" +repage \
  "$SCREENSHOT_OUT"
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
