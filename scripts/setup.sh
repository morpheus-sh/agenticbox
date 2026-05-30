#!/usr/bin/env bash
# AgenticBox Setup Script
# Works on Linux, macOS, WSL

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$SCRIPT_DIR/.."
cd "$REPO_ROOT"

echo "AgenticBox - Setup"
echo "================================"

# --- Detect and setup PATH for tools ---
find_in_path() {
    command -v "$1" 2>/dev/null
}

find_hermes_node() {
    local hermes_node="$HOME/.hermes/node/bin"
    if [ -d "$hermes_node" ] && [ -f "$hermes_node/pnpm" ]; then
        echo "$hermes_node"
        return 0
    fi
    local pnpm_home="${PNPM_HOME:-$HOME/.local/share/pnpm}"
    if [ -d "$pnpm_home" ] && [ -f "$pnpm_home/pnpm" ]; then
        echo "$pnpm_home"
        return 0
    fi
    return 1
}

# Try to add hermes/pnpm path if not already there
HERMES_NODE_DIR=$(find_hermes_node || true)
if [ -n "$HERMES_NODE_DIR" ]; then
    export PATH="$HERMES_NODE_DIR:$PATH"
    echo "Found node tooling at: $HERMES_NODE_DIR"
fi

# Detect package manager
if command -v pnpm &> /dev/null; then
    PKG_MGR="pnpm"
elif command -v npm &> /dev/null; then
    PKG_MGR="npm"
else
    echo "ERROR: No Node package manager found. Install pnpm or npm."
    echo "  pnpm:   curl -fsSL https://get.pnpm.io/install.sh | sh -"
    echo "  npm:    Install Node.js from https://nodejs.org"
    exit 1
fi
echo "Using package manager: $PKG_MGR"

# Detect python and pip
if command -v python3 &> /dev/null; then
    PYTHON="python3"
elif command -v python &> /dev/null; then
    PYTHON="python"
else
    echo "ERROR: Python not found. Install Python 3.11+."
    exit 1
fi
PY_VER=$($PYTHON -c 'import sys; print("." .join(map(str, sys.version_info[:2])))')
echo "Using python: $PYTHON (v$PY_VER)"

# Detect pip
if command -v pip3 &> /dev/null; then
    PIP="pip3"
elif command -v pip &> /dev/null; then
    PIP="pip"
else
    echo "WARNING: pip/pip3 not found. Python agent runtime setup will be skipped."
    PIP=""
fi

# Rust check
if ! command -v cargo &> /dev/null; then
    echo "ERROR: Rust/Cargo not found."
    echo "  Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://rustup.rs | sh"
    exit 1
fi
echo "Rust: $(cargo --version)"

# Rancher Desktop check (provides Docker-compatible containerd)
if command -v nerdctl &> /dev/null && (command -v docker &> /dev/null || [ -S "/run/rancher-desktop/lima/docker.sock" ] || [ -S "/run/docker.sock" ]); then
    echo "Rancher Desktop detected. nerdctl: $(nerdctl --version 2>/dev/null || true)"
elif command -v docker &> /dev/null; then
    echo "Docker detected: $(docker --version)"
else
    echo "WARNING: No container runtime found. Sandbox features require Rancher Desktop (recommended) or Docker."
    echo "  WSL (recommended): https://rancherdesktop.io/  (install Rancher Desktop with containerd)"
    echo "  macOS/Linux:       https://rancherdesktop.io/"
    echo "  Docker alternative: https://docs.docker.com/get-docker/"
fi

# --- Install dependencies ---

echo ""
echo "Installing Rust dependencies..."
cargo fetch

echo ""
echo "Installing frontend dependencies..."
cd apps/desktop
$PKG_MGR install --ignore-scripts
cd "$REPO_ROOT"

echo ""
if [ -z "$PIP" ]; then
    echo "Skipping Python agent runtime setup (no pip detected)."
else
    echo "Setting up Python agent runtime..."
    cd apps/agent-runtime
    VENV_CREATED=""
    if [ ! -d ".venv" ]; then
        if $PYTHON -m venv .venv 2>/dev/null; then
            VENV_CREATED=1
        else
            echo "WARNING: python3 -m venv failed (missing python3-venv package?). Skipping venv, using pip --user or system packages."
            VENV_CREATED=""
        fi
    fi
    if [ -n "$VENV_CREATED" ] || [ -f ".venv/bin/activate" ]; then
        source .venv/bin/activate
        $PIP install -e ".[dev]" 2>&1 || echo "WARNING: pip install failed."
        PLAYWRIGHT_EXEC=".venv/bin/playwright"
    else
        $PIP install -e ".[dev]" --user 2>&1 || echo "WARNING: pip install failed."
        PLAYWRIGHT_EXEC=$(command -v playwright 2>/dev/null || true)
    fi

    # Playwright
    if [ -n "$PLAYWRIGHT_EXEC" ] && [ -f "$PLAYWRIGHT_EXEC" ]; then
        $PLAYWRIGHT_EXEC install chromium 2>&1 || echo "WARNING: Playwright chromium install failed."
    elif command -v playwright &> /dev/null; then
        playwright install chromium 2>&1 || echo "WARNING: Playwright chromium install failed."
    else
        echo "WARNING: playwright CLI not found. Run: 'pip install playwright && playwright install chromium'"
    fi
    cd "$REPO_ROOT"
fi

# --- Data directory ---
mkdir -p data

# --- Build daemon for quick start ---
echo ""
echo "Building Rust daemon..."
cargo build --release --bin daemon 2>&1

echo ""
echo "=================================="
echo "AgenticBox setup complete."
echo ""
echo "Start the full stack:  ./scripts/dev.sh"
echo "Start daemon only:     target/release/daemon"
echo "Desktop dev:           cd apps/desktop && pnpm tauri dev"
echo ""
