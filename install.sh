#!/usr/bin/env bash
# AgenticBox one-line installer
# Usage: curl -fsSL https://agenticbox.co/install.sh | bash
#        or: wget -qO- https://agenticbox.co/install.sh | bash

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
BOLD='\033[1m'
DIM='\033[2m'
RESET='\033[0m'

# Config
REPO="morpheus-sh/agenticbox"
INSTALL_DIR="${HOME}/.agenticbox"
BIN_DIR="${INSTALL_DIR}/bin"
DAEMON_BIN="daemon"
CLI_BIN="agenticbox"

AGENTS_DIR="${INSTALL_DIR}/agents"
PROFILES_REPO="https://raw.githubusercontent.com/${REPO}/main/agents"

print_step() { echo -e "${CYAN}▶${RESET} ${BOLD}$1${RESET}"; }
print_ok() { echo -e "${GREEN}✓${RESET} $1"; }
print_warn() { echo -e "${YELLOW}⚠${RESET} $1"; }
print_err() { echo -e "${RED}✗${RESET} $1"; }
print_info() { echo -e "${DIM}$1${RESET}"; }

header() {
  echo -e "${MAGENTA}"
  cat <<'EOF'
    █████╗ ██████╗ ██╗   ██╗███████╗██████╗ ██████╗  ██████╗ ██████╗ ███████╗
   ██╔══██╗██╔══██╗██║   ██║██╔════╝██╔══██╗██╔══██╗██╔═══██╗██╔══██╗██╔════╝
   ███████║██████╔╝██║   ██║█████╗  ██████╔╝██████╔╝██║   ██║██████╔╝███████╗
   ██╔══██║██╔══██╗╚██╗ ██╔╝██╔══╝  ██╔══██╗██╔══██╗██║   ██║██╔══██╗╚════██║
   ██║  ██║██║  ██║ ╚████╔╝ ███████╗██║  ██║██║  ██║╚██████╔╝██║  ██║███████║
   ╚═╝  ╚═╝╚═╝  ╚═╝  ╚═══╝  ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝
EOF
  echo -e "${RESET}"
  echo -e "${BOLD}Governance Layer for AI Agents${RESET}"
  echo -e "${DIM}Open source • Local-first • Rust + Tauri${RESET}\n"
}

detect_os() {
  case "$(uname -s)" in
    Linux*)  OS="linux" ;;
    Darwin*) OS="macos" ;;
    CYGWIN*|MINGW*|MSYS*) OS="windows" ;;
    *) print_err "Unsupported OS: $(uname -s)"; exit 1 ;;
  esac
  ARCH="$(uname -m)"
  case "$ARCH" in
    x86_64|amd64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *) print_err "Unsupported arch: $ARCH"; exit 1 ;;
  esac
  print_info "Detected: $OS/$ARCH"
}

check_cmd() { command -v "$1" >/dev/null 2>&1; }

install_rust() {
  if check_cmd cargo; then
    print_ok "Rust already installed ($(cargo --version | cut -d' ' -f2))"
    return
  fi
  print_step "Installing Rust via rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "$HOME/.cargo/env"
  print_ok "Rust installed"
}

install_docker() {
  if check_cmd docker; then
    print_ok "Docker already installed"
    return
  fi
  print_warn "Docker not found. AgenticBox requires Docker for sandbox execution."
  print_info "Install Docker: https://docs.docker.com/engine/install/"
  if [[ "$OS" == "linux" ]]; then
    print_info "Quick install: curl -fsSL https://get.docker.com | sh"
  elif [[ "$OS" == "macos" ]]; then
    print_info "Install Docker Desktop: https://www.docker.com/products/docker-desktop/"
  fi
  read -rp "Continue anyway? [y/N] " -n 1; echo
  [[ $REPLY =~ ^[Yy]$ ]] || exit 1
}

fetch_release() {
  print_step "Fetching latest release..."
  local api_url="https://api.github.com/repos/${REPO}/releases/latest"
  local tag_name
  tag_name=$(curl -fsSL "$api_url" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
  if [[ -z "$tag_name" ]]; then
    print_err "Failed to fetch release info"
    exit 1
  fi
  print_ok "Latest release: $tag_name"

  local asset_name="agenticbox-${tag_name}-${OS}-${ARCH}.tar.gz"
  local download_url="https://github.com/${REPO}/releases/download/${tag_name}/${asset_name}"

  print_step "Downloading $asset_name..."
  mkdir -p "$INSTALL_DIR"
  if ! curl -fsSL -o "${INSTALL_DIR}/${asset_name}" "$download_url"; then
    print_err "Download failed. Asset may not exist for $OS/$ARCH."
    print_info "Falling back to building from source..."
    build_from_source
    return
  fi
  print_ok "Downloaded"

  print_step "Extracting..."
  tar -xzf "${INSTALL_DIR}/${asset_name}" -C "$INSTALL_DIR"
  rm "${INSTALL_DIR}/${asset_name}"
  print_ok "Extracted"
}

build_from_source() {
  print_step "Building from source (requires Rust + Docker)..."
  install_rust
  install_docker
  local tmp_dir
  tmp_dir=$(mktemp -d)
  git clone --depth 1 "https://github.com/${REPO}.git" "$tmp_dir"
  cd "$tmp_dir"
  cargo build --release --bin daemon --bin agenticbox
  cp target/release/daemon target/release/agenticbox "$BIN_DIR/"
  cd -
  rm -rf "$tmp_dir"
  print_ok "Built from source"
}

install_binaries() {
  print_step "Installing binaries to $BIN_DIR..."
  mkdir -p "$BIN_DIR"
  # If we extracted a release, binaries are in $INSTALL_DIR/
  if [[ -f "${INSTALL_DIR}/daemon" && -f "${INSTALL_DIR}/agenticbox" ]]; then
    mv "${INSTALL_DIR}/daemon" "${INSTALL_DIR}/agenticbox" "$BIN_DIR/"
  fi
  chmod +x "$BIN_DIR/$DAEMON_BIN" "$BIN_DIR/$CLI_BIN"
  print_ok "Binaries installed"
}

setup_path() {
  local shell_rc=""
  case "$(basename "${SHELL:-bash}")" in
    zsh) shell_rc="${ZDOTDIR:-$HOME}/.zshrc" ;;
    fish) shell_rc="${HOME}/.config/fish/config.fish" ;;
    *) shell_rc="${HOME}/.bashrc" ;;
  esac

  local path_entry="export PATH=\"${BIN_DIR}:\$PATH\""
  if [[ -f "$shell_rc" ]] && grep -q "$BIN_DIR" "$shell_rc"; then
    print_ok "PATH already configured in $shell_rc"
  else
    echo "" >> "$shell_rc"
    echo "# AgenticBox" >> "$shell_rc"
    echo "$path_entry" >> "$shell_rc"
    print_ok "Added $BIN_DIR to PATH in $shell_rc"
    print_info "Run: source $shell_rc"
  fi
}

verify_install() {
  print_step "Verifying installation..."
  if "$BIN_DIR/$CLI_BIN" --version >/dev/null 2>&1; then
    local version
    version=$("$BIN_DIR/$CLI_BIN" --version)
    print_ok "agenticbox CLI: $version"
  else
    print_warn "CLI not in PATH yet. Run: source $(detect_shell_rc)"
  fi
}

detect_shell_rc() {
  case "$(basename "${SHELL:-bash}")" in
    zsh) echo "${ZDOTDIR:-$HOME}/.zshrc" ;;
    fish) echo "${HOME}/.config/fish/config.fish" ;;
    *) echo "${HOME}/.bashrc" ;;
  esac
}

install_profile() {
  local profile="$1"
  print_step "Installing agent profile: $profile"
  local profile_dir="${AGENTS_DIR}/${profile}"
  mkdir -p "$profile_dir"

  # Fetch agent.toml
  if ! curl -fsSL "${PROFILES_REPO}/${profile}/agent.toml" -o "${profile_dir}/agent.toml"; then
    print_err "Failed to download agent.toml for '$profile'"
    print_info "Available profiles: https://github.com/${REPO}/tree/main/agents"
    return 1
  fi
  print_ok "agent.toml downloaded"

  # Fetch any sample files referenced in the profile
  # Look for samples/ directory in the repo
  local samples_dir="${profile_dir}/samples"
  mkdir -p "$samples_dir"
  local samples_base="${PROFILES_REPO}/${profile}/samples"

  # Try to fetch known sample files (best-effort, don't fail if none)
  for sample_file in sample_optimize_cache.sh incident_report.txt; do
    if curl -fsSL "${samples_base}/${sample_file}" -o "${samples_dir}/${sample_file}" 2>/dev/null; then
      print_ok "sample: ${sample_file}"
    fi
  done

  print_ok "Profile '$profile' installed to ${profile_dir}"
}

run_setup() {
  local cli="$1"
  if "$cli" --version >/dev/null 2>&1; then
    print_step "Running agenticbox setup..."
    "$cli" setup || print_warn "Setup interrupted — run 'agenticbox setup' manually later"
    print_ok "Setup complete"
  else
    print_warn "CLI not in PATH yet. Run 'agenticbox setup' after sourcing your shell rc."
  fi
}

main() {
  local profile="${1:-}"
  header
  detect_os
  install_rust
  install_docker
  fetch_release
  install_binaries
  setup_path
  verify_install

  # Install agent profile if specified
  if [[ -n "$profile" ]]; then
    install_profile "$profile"
  fi

  # Run setup if profile was specified (builtin mode needs LLM config)
  if [[ -n "$profile" ]]; then
    run_setup "$BIN_DIR/$CLI_BIN"
  fi

  echo
  echo -e "${GREEN}${BOLD}Installation complete!${RESET}"
  echo

  if [[ -n "$profile" ]]; then
    echo -e "${BOLD}Next steps:${RESET}"
    echo -e "  1. ${CYAN}source $(detect_shell_rc)${RESET}  (or restart your shell)"
    if [[ "$profile" == "security-analyst" ]]; then
      echo -e "  2. ${CYAN}agenticbox run security-analyst${RESET}  (start analyzing)"
    else
      echo -e "  2. ${CYAN}agenticbox run ${profile}${RESET}"
    fi
    echo
    echo -e "${DIM}Docs: https://github.com/${REPO}/blob/main/docs/agents.md${RESET}"
  else
    echo -e "${BOLD}Next steps:${RESET}"
    echo -e "  1. ${CYAN}source $(detect_shell_rc)${RESET}  (or restart your shell)"
    echo -e "  2. ${CYAN}agenticbox setup${RESET}        (configure LLM inference)"
    echo -e "  3. ${CYAN}agenticbox run demo${RESET}      (see it in action)"
    echo -e "  4. ${CYAN}agenticbox run -- python3 script.py${RESET}  (sandbox any command)"
    echo
    echo -e "${DIM}Install a profile:  curl -fsSL https://agenticbox.co/install.sh | bash -s -- security-analyst${RESET}"
    echo -e "${DIM}Docs: https://github.com/${REPO}/blob/main/docs/agents.md${RESET}"
  fi
  echo -e "${DIM}GitHub: https://github.com/${REPO}${RESET}"
}

main "$@"