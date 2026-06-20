# AgenticBox

> **Give your AI agent a workplace, not root access.** Okta + Docker for AI agents. Open source. Local-first.

[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://rustup.rs)
[![CI](https://github.com/morpheus-sh/agenticbox/actions/workflows/ci.yml/badge.svg)](https://github.com/morpheus-sh/agenticbox/actions)

---

## What Is This?

Agents need workplaces. From that single idea, the rest follows naturally:

- **Permissions** — what the coworker is allowed to do
- **Identity** — who the coworker is
- **Accountability** — what the coworker did
- **Ownership boundaries** — what belongs to the coworker vs. the organization

Give your AI agent a workplace instead of root access. Okta + Docker for AI agents — scoped permissions, isolated execution, full accountability.

```
┌─────────────────────────────────────────────────────┐
│                  AgenticBox                          │
├─────────────────────────────────────────────────────┤
│                                                      │
│   agenticbox run <agent>                             │
│        │                                             │
│        ▼                                             │
│   ┌──────────┐    ┌──────────────┐                  │
│   │  CLI     │───▶│  Docker      │                  │
│   │          │    │  Container   │                  │
│   │  relay   │◄──▶│  (sandbox)   │                  │
│   └──────────┘    └──────┬───────┘                  │
│                          │                          │
│                    ┌─────┴──────┐                   │
│                    │ Agent CLI  │                   │
│                    │ runs here  │                   │
│                    │ /workspace │                   │
│                    └────────────┘                   │
│                                                      │
└─────────────────────────────────────────────────────┘
```

The agent CLI runs **inside** a sandboxed container. The host relays stdin/stdout. No pre-built images per agent — agents install at runtime from a TOML profile.

### The Four Pillars

| Pillar | What it means |
|--------|---------------|
| **Permissions** | Terminal, filesystem, network, browser — scoped and enforced. The agent can only do what it's authorized to do. |
| **Ownership Boundaries** | Clear boundaries: resources, outputs, budgets, assets. |
| **Accountability** | Every action attributed, logged, auditable. Full audit trail. |
| **Identity** | Agents get their own credentials, accounts, digital identity — provisioned and revocable. *(The moat.)* |

---

## What's Shipped

| Feature | Status |
|---------|--------|
| **Real Docker Execution** | ✅ `agenticbox run` spawns real containers via bollard, streams output, cleans up |
| **Ad-hoc Commands** | ✅ `agenticbox run -- python3 script.py` — any command in a sandbox |
| **Named Agent Profiles** | ✅ `agenticbox run pi` — runtime install + exec in container |
| **TTY Support** | ✅ Interactive agents get a real PTY (crossterm raw mode) |
| **Permission Guards** | ✅ Terminal, filesystem (RO/RW), network (allowlist/localhost/offline) |
| **Filesystem Governance** | ✅ Path resolution with escape prevention, protected paths (SSH keys, AWS creds) |
| **Network Control** | ✅ Domain allowlist enforcement |
| **Agent Packages** | ✅ TOML manifests with `[image]` section for container + install steps |
| **Built-in Demo** | ✅ `agenticbox run demo` — scripted permission guard showcase |
| **Session Management** | ⚠️ SQLite-backed, exists but daemon doesn't create containers yet |
| **Desktop Console** | ⚠️ Tauri v2 app exists, needs integration with new container runtime |
| **ACP Permission Interception** | 🔵 Next — parse JSON-RPC, block/allow tool calls |
| **Agent Identity** | 🔵 Future — agents get own credentials, provisioned and revocable |

---

## Quick Start

### Prerequisites

- **Docker** (Docker Desktop on macOS/Windows, Docker CE on Linux/WSL)
- **Rust 1.75+** (to build from source)

### Build from source

```bash
git clone https://github.com/morpheus-sh/agenticbox.git
cd agenticbox
cargo build --release
```

The binary will be at `target/release/agenticbox`.

### See it work immediately

```bash
# Ad-hoc command in a real container
./target/release/agenticbox run -- echo "hello from sandbox"

# Run python in an isolated container
./target/release/agenticbox run -- python3 -c "print('sandboxed!')"

# Built-in demo (scripted permission guard showcase)
./target/release/agenticbox run demo
```

---

## Running Agents

The `agenticbox run` command is the primary interface. Like `docker run <image>` → `agenticbox run <agent>`.

### Ad-hoc Commands

```bash
agenticbox run -- python3 script.py
agenticbox run -- npm test
agenticbox run -- make build
```

Wraps any command in a sandboxed Docker container. Defaults: `terminal=on`, `fs=readonly`, `network=allowlist`.

### Named Agents

```bash
agenticbox run pi          # Pi coding agent (pi.dev)
agenticbox run hermes      # Hermes agent (Nous Research)
```

Reads the agent profile from `~/.agenticbox/agents/<name>/agent.toml`, pulls a base container image, installs the agent at runtime, and launches it with interactive stdio relay.

### Built-in Demo

```bash
agenticbox run demo
```

Runs a scripted permission guard demo — an agent tries to read SSH keys, exfiltrate data, write to system paths, and each attempt is caught or allowed in real-time.

### Installing Agent Profiles

Copy the example profiles from this repo:

```bash
mkdir -p ~/.agenticbox/agents
cp -r agents/* ~/.agenticbox/agents/
```

---

## Agent Profiles

Agents are TOML manifests in `~/.agenticbox/agents/<name>/agent.toml`:

```toml
# ~/.agenticbox/agents/pi/agent.toml
name = "pi"
description = "Pi Agent — edge computing, IoT device management"

# Command that launches the agent inside the container
command = "pi"

[model]
provider = "anthropic"
model = "claude-sonnet-4-20250514"
api_key_env = "ANTHROPIC_API_KEY"

[permissions]
terminal = true
filesystem = "readwrite"
browser = false
network = "allowlist"
domains = ["pi.dev", "registry.npmjs.org", "api.anthropic.com"]

# Container image + runtime install steps
[image]
base = "node:22-slim"
setup = [
    "apt-get update && apt-get install -y curl",
    "curl -fsSL https://pi.dev/install.sh | sh"
]
```

Each `setup` command runs as `sh -c "<command>"` — pipes, flags, and `&&` chains all work.

### Managing Agents

```bash
agenticbox agents                # list available agents
agenticbox agents --paths        # show config directory
agenticbox init my-agent         # create a new agent manifest
```

See [`docs/agents.md`](docs/agents.md) for the full agent manifest reference.

---

## CLI Reference

```bash
# Run agents
agenticbox run demo                    # built-in permission guard demo
agenticbox run pi                      # named agent from manifest
agenticbox run -- python3 script.py    # ad-hoc command wrapping

# Manage
agenticbox agents                      # list available agents
agenticbox init my-agent               # create new agent manifest
agenticbox list                        # list sessions (daemon mode)
agenticbox get <SESSION_ID>            # session details
agenticbox logs <SESSION_ID> -f        # stream logs
agenticbox stop <SESSION_ID>           # stop session
agenticbox health                      # health check
```

| Flag | Description | Default |
|------|-------------|---------|
| `--terminal` | Enable terminal access | `true` |
| `--fs` | Filesystem permission: readonly, readwrite, none | `readonly` |
| `--network` | Network policy: allowlist, localhost, offline, full | `allowlist` |
| `--domains` | Allowed domains (comma-separated) | `api.openai.com,github.com` |
| `--browser` | Enable browser automation | `false` |

---

## Architecture

### How `run` works

```
1. Read agent.toml → get image base + setup commands + agent command
2. docker create (base image, sleep infinity, mount cwd → /workspace, env vars)
3. docker start
4. for each setup command: docker exec (install agent)
5. docker exec -it (agent command) ← interactive stdio relay
6. host stdin → container stdin, container stdout → host stdout
7. on exit: docker stop + docker rm
```

**No daemon required for `run`.** The CLI talks directly to your container runtime via [bollard](https://github.com/fussybeaver/bollard-rust). Docker and Podman are both supported (auto-detected at startup). Set `AGENTICBOX_CONTAINER_SOCKET=/path/to/socket` to override. The daemon is only needed for persistent, background sessions (`deploy`).

### Crates (Rust)

| Crate | Purpose |
|-------|---------|
| `sandbox-core` | Docker container lifecycle: create/start/stop/remove, exec (interactive + wait), log streaming, image pull |
| `fs-guard` | Filesystem path resolution with escape prevention |
| `shared-types` | Common types: Session, ModelConfig, PermissionSet |
| `network-control` | Network policy enforcement (allowlist/localhost/offline) |

### Apps

| App | Tech | Purpose |
|-----|------|---------|
| `apps/cli` | Rust + Clap | Command-line interface — the primary entry point |
| `apps/daemon` | Rust + Axum | REST API, WebSocket, persistent session management |
| `apps/desktop` | Tauri v2 + React | Native desktop console |

### Design Docs

- [`docs/designs/dx-user-journey.md`](docs/designs/dx-user-journey.md) — The three modes (ad-hoc, named agent, daemon), container lifecycle, ACP transport decisions

---

## Roadmap

### Now ✅
Real Docker execution — `agenticbox run` spawns containers, installs agents at runtime, relays interactive stdio with PTY support.

### Next 🟡
ACP permission interception — parse JSON-RPC traffic between host and agent, enforce allow/deny on tool calls based on the permission profile.

### Later 🔵
Agent identity — agents get their own credentials and accounts. Browser automation. The bounded agent becomes an coworker.

---

## Development

```bash
# Build
cargo build

# Run tests
cargo test --workspace

# Run ad-hoc test
cargo run -p agenticbox-cli -- run -- echo "test"
```

**Windows:** If `cargo build` fails with linker errors, ensure MSVC tools are first in PATH:
```bash
export PATH="/c/Program Files (x86)/Microsoft Visual Studio/2022/BuildTools/VC/Tools/MSVC/<version>/bin/Hostx64/x64:$PATH"
```

---

## Community

- **GitHub** — [github.com/morpheus-sh/agenticbox](https://github.com/morpheus-sh/agenticbox)
- **Website** — [agenticbox.co](https://agenticbox.co)
- **Twitter** — [@agenticbox](https://twitter.com/agenticbox)

---

## License

**MIT OR Apache-2.0** — Choose whichever suits your project.

---

> **AgenticBox** — Give your agent a workplace.
