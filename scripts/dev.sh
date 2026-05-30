#!/usr/bin/env bash
# AgenticBox Dev Stack Launcher
# Starts daemon + desktop concurrently, with cleanup on exit.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$SCRIPT_DIR/.."
cd "$REPO_ROOT"

mkdir -p data

export RUST_LOG=info
export DATABASE_URL="sqlite:data/sessions.db"

echo "Starting AgenticBox development stack..."

# Detect package manager
if command -v pnpm &> /dev/null; then
    PKG_MGR="pnpm"
elif command -v npm &> /dev/null; then
    PKG_MGR="npm"
else
    echo "ERROR: No package manager found. Run ./scripts/setup.sh first."
    exit 1
fi

echo "Using package manager: $PKG_MGR"

# Check if release daemon exists, else use debug
if [ -f "target/release/daemon" ]; then
    DAEMON_BIN="target/release/daemon"
    echo "Using release daemon build."
else
    DAEMON_BIN="target/debug/daemon"
    echo "Using debug daemon build (run ./scripts/setup.sh for release)."
fi

echo ""
echo "Starting daemon    -> http://127.0.0.1:8080"
echo "Starting desktop   -> Tauri window"
echo ""

# Start daemon in background
"$DAEMON_BIN" &
DAEMON_PID=$!

# Start desktop dev server in background
cd apps/desktop
$PKG_MGR tauri dev &
TAURI_PID=$!
cd "$REPO_ROOT"

cleanup() {
    echo ""
    echo "Shutting down AgenticBox dev stack..."
    kill $DAEMON_PID $TAURI_PID 2>/dev/null || true
    wait $DAEMON_PID $TAURI_PID 2>/dev/null || true
    echo "Done."
}
trap cleanup EXIT INT TERM

# Wait for either process to exit
wait -n $DAEMON_PID $TAURI_PID
EXIT_CODE=$?

echo ""
echo "One of the services exited (code $EXIT_CODE). Stopping the other..."
exit $EXIT_CODE
