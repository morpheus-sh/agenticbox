#!/usr/bin/env bash
# AgenticBox One-Click Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/ndimas/agentbox/main/install.sh | bash

set -e

echo "AgenticBox Installer"
echo "===================="

# --- Config ---
REPO_URL="https://github.com/ndimas/agentbox.git"
INSTALL_DIR="${AGENTICBOX_DIR:-$HOME/.agenticbox}"
BIN_DIR="$INSTALL_DIR/bin"

# --- OS detection ---
OS=""
case "$(uname -s)" in
    Linux*)     OS=linux;;
    Darwin*)    OS=macos;;
    MINGW*|MSYS*|CYGWIN*) OS=windows;;
    *)          OS=unknown;;
esac

if [ "$OS" = "unknown" ]; then
    echo "ERROR: Unsupported OS: $(uname -s)"
    exit 1
fi

echo "Detected OS: $OS"

# --- Prerequisites ---
MISSING=""

command -v git &> /dev/null  || MISSING="$MISSING git"
command -v cargo &> /dev/null || MISSING="$MISSING rust"
command -v node &> /dev/null || MISSING="$MISSING node"
command -v pnpm &> /dev/null || MISSING="$MISSING pnpm"

if [ -n "$MISSING" ]; then
    echo ""
    echo "MISSING PREREQUISITES:$MISSING"
    echo ""
    echo "Please install them first:"
    echo ""
    echo "  Rust:      curl --proto '=https' --tlsv1.2 -sSf https://rustup.rs | sh"
    echo "  Node:      https://nodejs.org or use Hermes: curl -fsSL https://hermes-agent.nousresearch.com/install.sh | bash"
    echo "  pnpm:      npm install -g pnpm"
    echo "  git:       sudo apt install git  (or equivalent)"
    echo ""
    echo "Optional (recommended for Windows/WSL):"
    echo "  Rancher Desktop: https://rancherdesktop.io/"
    echo ""
    exit 1
fi

echo "All prerequisites found."

# --- Clone or update ---
if [ -d "$INSTALL_DIR/.git" ]; then
    echo "Updating existing installation in $INSTALL_DIR ..."
    cd "$INSTALL_DIR"
    git pull origin main
else
    echo "Cloning into $INSTALL_DIR ..."
    git clone "$REPO_URL" "$INSTALL_DIR"
    cd "$INSTALL_DIR"
fi

# --- Run setup ---
echo ""
echo "Running setup ..."
./scripts/setup.sh

# --- Create bin wrapper ---
mkdir -p "$BIN_DIR"

cat > "$BIN_DIR/agenticbox" <<'EOF'
#!/usr/bin/env bash
# AgenticBox CLI wrapper
set -e
INSTALL_DIR="${AGENTICBOX_DIR:-$HOME/.agenticbox}"
cd "$INSTALL_DIR"

case "$1" in
    dev)
        shift
        ./scripts/dev.sh "$@"
        ;;
    setup)
        shift
        ./scripts/setup.sh "$@"
        ;;
    daemon)
        shift
        mkdir -p data
        export RUST_LOG=info
        exec "$INSTALL_DIR/target/release/daemon" "$@"
        ;;
    version|--version|-v)
        echo "AgenticBox $(git describe --tags --always 2>/dev/null || echo 'dev')"
        ;;
    *)
        echo "AgenticBox CLI"
        echo ""
        echo "Commands:"
        echo "  agenticbox dev      Start the development stack"
        echo "  agenticbox daemon   Start the Rust daemon only"
        echo "  agenticbox setup    Re-run setup"
        echo "  agenticbox version  Show version"
        echo ""
        echo "Config dir: $INSTALL_DIR"
        echo "Docs:       https://github.com/ndimas/agentbox#readme"
        ;;
esac
EOF
chmod +x "$BIN_DIR/agenticbox"

# --- Shell integration hint ---
SHELL_RC=""
SHELL_NAME="$(basename "$SHELL")"
case "$SHELL_NAME" in
    bash) SHELL_RC="$HOME/.bashrc" ;;
    zsh)  SHELL_RC="$HOME/.zshrc" ;;
    fish) SHELL_RC="$HOME/.config/fish/config.fish" ;;
    *)    SHELL_RC="" ;;
esac

echo ""
echo "=================================="
echo "AgenticBox installed successfully!"
echo ""
echo "Installation dir: $INSTALL_DIR"
echo "Binary:           $BIN_DIR/agenticbox"
echo ""
if [ -n "$SHELL_RC" ]; then
    if ! grep -q "$BIN_DIR" "$SHELL_RC" 2>/dev/null; then
        echo "Add to PATH:"
        echo "  echo 'export PATH=\"$BIN_DIR:\$PATH\"' >> $SHELL_RC"
    fi
fi
echo "Start now:        $BIN_DIR/agenticbox dev"
echo ""
