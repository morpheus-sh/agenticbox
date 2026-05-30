# AgenticBox

Local-first, vendor-neutral AI agent sandbox runtime.

Run autonomous AI agents in isolated sandboxes with browser automation, filesystem tools, and terminal access — all on your own machine. No cloud required.

```
User
 ↓
Desktop App (Tauri)
 ↓
Rust Supervisor Daemon
 ↓
Sandbox Runtime (Rancher Desktop / Docker)
 ↓
Agent Session
 ↓
Model Endpoint (OpenAI, Ollama, vLLM, ...)
```

## Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/ndimas/agentbox/main/install.sh | bash
```

Then start:

```bash
agenticbox dev
```

## Prerequisites

| Tool | Purpose | Install |
|------|---------|---------|
| Rust | Daemon & crates | `curl --proto '=https' --tlsv1.2 -sSf https://rustup.rs \| sh` |
| Node + pnpm | Desktop UI | [Node.js](https://nodejs.org) or via Hermes |
| Python 3.11+ | Agent runtime | Usually pre-installed |
| Rancher Desktop *(recommended)* | Container runtime | [rancherdesktop.io](https://rancherdesktop.io/) |
| Docker *(alternative)* | Container runtime | [docker.com](https://docs.docker.com/get-docker/) |

### WSL Users (recommended setup)

1. Install [Rancher Desktop](https://rancherdesktop.io/) on Windows — it auto-integrates with WSL.
2. Install Rust inside WSL.
3. Install Node/pnpm (Hermes is convenient).
4. Run the install command above.

## Development

```bash
# Clone manually
git clone https://github.com/ndimas/agentbox.git
cd agentbox

# Setup everything (Rust deps, frontend, Python runtime)
./scripts/setup.sh

# Start full stack (daemon + desktop)
./scripts/dev.sh
```

Individual components:

```bash
# Daemon only
cargo run --bin daemon

# Desktop only
cd apps/desktop && pnpm tauri dev

# Build release daemon
cargo build --release --bin daemon
```

## Architecture

- **Daemon** (`apps/daemon`): Rust axum server, orchestrates sandboxes and models.
- **Desktop** (`apps/desktop`): Tauri v2 + React + Tailwind UI.
- **Agent Runtime** (`apps/agent-runtime`): Python FastAPI server with Playwright tools.
- **sandbox-core**: Docker-compatible container runtime (Rancher Desktop / Docker).
- **session-manager**: SQLite-backed session storage.
- **model-router**: Unified model provider adapter.

See `docs/ARCHITECTURE.md` for full details.

## Features

| Feature | Phase | Status |
|---------|-------|--------|
| Container sandbox lifecycle | 1 | Ready |
| Terminal tool (shell streaming) | 1 | Ready |
| Filesystem mounts + guard | 1 | Ready |
| OpenAI-compatible API | 1 | Ready |
| Tauri desktop app | 1 | Ready |
| Session persistance (SQLite) | 1 | Ready |
| Permission system | 1 | UI scaffold |
| Browser automation (Playwright) | 2 | Planned |
| Firecracker microVMs | 3 | Planned |
| Policy engine | 3 | Planned |

## Extension Support

AgenticBox is designed to support multiple agent backends:

- **Built-in**: The default Python agent runtime.
- **Hermes** *(future)* — Switch via settings.
- **Pi Agent** *(future)* — Switch via settings.

Extension points are in `apps/daemon/src/extensions/` and future agent backends will implement the `AgentBackend` trait.

## License

MIT OR Apache-2.0
