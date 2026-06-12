# AgenticBox

> **Vercel for AI Agents** — Deploy autonomous agents without worrying about sandboxes, permissions, browser sessions, secret management, observability, or cost controls.

[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://rustup.rs)
[![Tauri](https://img.shields.io/badge/Tauri-v2-green.svg)](https://tauri.app)
[![Python](https://img.shields.io/badge/Python-3.11%2B-blue.svg)](https://python.org)
[![Phase](https://img.shields.io/badge/Phase-1%20Ready-brightgreen.svg)](#roadmap)

---

## What Is This?

**AgenticBox** is an open-source, local-first runtime for deploying AI agents to production. Think of it as the infrastructure layer that handles everything agents need to run safely — so you can focus on building agent logic, not managing containers, permissions, or browser sessions.

### The Problem

AI agents are powerful but fragile in production. They need:

- **Sandboxing** — isolated execution environments
- **Permissions** — what can the agent read, write, execute?
- **Browser automation** — headless browsers for web interaction
- **Secret management** — API keys, tokens, credentials
- **Observability** — logs, metrics, traces per agent
- **Cost controls** — billing by usage, not just compute

Most teams build this from scratch or hack together Docker + custom scripts. AgenticBox makes it a solved problem.

### The Solution

```
┌─────────────────────────────────────────────────────────────┐
│                     AgenticBox Stack                         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Tauri     │  │   Rust      │  │   Python            │  │
│  │   Desktop   │◄─┤   Daemon    │◄─┤   Agent Runtime     │  │
│  │   (UI)      │  │  (Axum +    │  │  (FastAPI + MCP)    │  │
│  └─────────────┘  │   Bollard)  │  │  Terminal, FS,      │  │
│                   │   SQLite    │  │  Browser, HTTP      │  │
│                   └─────────────┘  └─────────────────────┘  │
│                          │                    │               │
│                   ┌─────┴─────┐        ┌────┴────┐          │
│                   │  Docker   │        │Playwright│          │
│                   │ Containers│        │(Phase 2) │          │
│                   └───────────┘        └─────────┘          │
└─────────────────────────────────────────────────────────────┘
```

---

## Current Status: Phase 1 Ready ✅

| Feature | Status | Details |
|---------|--------|---------|
| **Container Sandbox Lifecycle** | ✅ Shipped | Create/start/stop/remove Docker containers per agent |
| **Terminal Tool** | ✅ Shipped | Execute shell commands with timeout & output capture |
| **Filesystem Access + Guards** | ✅ Shipped | Read/write with path resolution preventing escapes |
| **Permission System** | ✅ Shipped | Terminal, FS (RO/RW), Browser, Network (allowlist/localhost/offline) |
| **Session Persistence** | ✅ Shipped | SQLite-backed with model config, permissions, status |
| **Tauri Desktop App** | ✅ Shipped | Native UI (no Electron) for managing agents |
| **OpenAI-Compatible API** | ✅ Shipped | Drop-in replacement for OpenAI endpoints |
| **Browser Automation (Playwright)** | 🟡 In Dev | Headless browser sessions for web interaction |
| **Secret Management** | 🔴 Planned | Secure injection via keyring/Vault |
| **Observability** | 🔴 Planned | Logs, metrics, traces per agent |
| **Cost Controls** | 🔴 Planned | Per-agent billing, quotas, budget alerts |
| **Firecracker MicroVMs** | 🔴 Future | Lightweight microVMs for stronger isolation |

See the full [Roadmap](#roadmap).

---

## Quick Start (Local)

### Prerequisites

- **Rust** 1.75+ — `curl --proto '=https' --tlsv1.2 -sSf https://rustup.rs | sh`
- **Node.js** + **pnpm** — `curl -fsSL https://get.pnpm.io/install.sh | sh`
- **Python** 3.11+ — for agent runtime
- **Rancher Desktop** (recommended) or **Docker** — for container sandbox

### One-Command Setup

```bash
git clone https://github.com/agenticbox/agenticbox.git
cd agenticbox
./scripts/setup.sh
```

### Run the Full Stack

```bash
./scripts/dev.sh
```

This starts:
- **Rust Daemon** → `http://127.0.0.1:8080` (REST + WebSocket)
- **Tauri Desktop App** → Native window for UI

### Run Daemon Only (for API access)

```bash
# After setup.sh completes
target/release/daemon
# → Daemon listening on http://127.0.0.1:8080
```

### Run Python Agent Runtime

```bash
cd apps/agent-runtime
source .venv/bin/activate  # or wherever your venv is
python -m agent_runtime.main
# → Agent Runtime on http://127.0.0.1:9000
```

---

## Architecture

### Crates (Rust)

| Crate | Purpose |
|-------|---------|
| `sandbox-core` | Docker container lifecycle (create/start/stop/remove/logs) |
| `session-manager` | SQLite-backed session CRUD + status transitions |
| `fs-guard` | Filesystem path resolution with escape prevention |
| `policy-engine` | Permission evaluation (terminal, FS, browser, network) |
| `shared-types` | Common types: Session, ModelConfig, PermissionSet, etc. |
| `model-router` | (Planned) Route requests to OpenAI/Ollama/vLLM/local |
| `network-control` | (Planned) Network policy enforcement |
| `tool-protocol` | (Planned) Standardized tool calling interface |

### Apps

| App | Tech | Purpose |
|-----|------|---------|
| `apps/daemon` | Rust + Axum | REST API, WebSocket, session/sandbox orchestration |
| `apps/desktop` | Tauri v2 + React | Native desktop UI |
| `apps/agent-runtime` | Python + FastAPI | MCP server exposing tools (terminal, fs, browser, http) |

---

## API Reference

### Create a Session

```bash
curl -X POST http://127.0.0.1:8080/sessions \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-research-agent",
    "model_config": {
      "provider": "openai",
      "model": "gpt-4o",
      "api_key_env": "OPENAI_API_KEY"
    },
    "permissions": {
      "terminal": true,
      "filesystem": "readwrite",
      "browser": true,
      "network": { "type": "allowlist", "domains": ["github.com", "api.openai.com"] }
    }
  }'
```

### List Sessions

```bash
curl http://127.0.0.1:8080/sessions
```

### WebSocket (Real-time)

```bash
# Connect to ws://127.0.0.1:8080/ws
# Send JSON messages for tool invocations
```

### Agent Runtime Tools (MCP)

```bash
# List available tools
curl http://127.0.0.1:9000/tools

# Invoke via WebSocket
ws://127.0.0.1:9000/ws
```

---

## Roadmap

### Phase 1 — Ready ✅
Core sandbox runtime: container lifecycle, terminal, filesystem, permissions, sessions, Tauri desktop, OpenAI-compatible API.

### Phase 2 — In Development 🟡
- **Browser Automation** — Playwright integration for navigate/click/type/extract
- **Secret Management** — Keyring (local) / Vault (cloud) injection at runtime
- **Basic Observability** — Log streaming via WebSocket, structured JSON logs

### Phase 2.5 — Near Term 🟡
- **CLI** — `agenticbox deploy <agent> --sandbox`
- **Dashboard Polish** — Real-time log streaming, session history, cost estimates
- **Waitlist → Beta** — Onboarding flow for managed cloud

### Phase 3 — Future 🔴
- **Firecracker MicroVMs** — Stronger isolation, faster cold starts
- **Advanced Policy Engine** — OPA-style policies, audit logging
- **Cost Controls** — Per-agent billing, quotas, budget alerts
- **Multi-Agent Orchestration** — Coordinated workflows, agent-to-agent communication
- **Managed Cloud** — Hosted AgenticBox with SSO, RBAC, VPC options

---

## Why AgenticBox?

| Dimension | Docker/K8s | Cloudflare Workers | OpenAI Assistants | **AgenticBox** |
|-----------|------------|---------------------|-------------------|----------------|
| **Sandboxing** | Manual | Built-in | Built-in | ✅ First-class |
| **Permissions** | Manual | Limited | Limited | ✅ Granular |
| **Browser** | DIY | ❌ | ❌ | ✅ Playwright |
| **Secrets** | DIY | Built-in | Built-in | ✅ Keyring/Vault |
| **Observability** | DIY | Built-in | Limited | ✅ Per-agent |
| **Local-first** | ✅ | ❌ | ❌ | ✅ Native |
| **Vendor-neutral** | ✅ | ❌ Cloudflare | ❌ OpenAI | ✅ Any model |
| **License** | Apache-2.0 | Proprietary | Proprietary | **MIT OR Apache-2.0** |

---

## Contributing

We welcome contributions! Priority areas:

1. **Browser tool** — Playwright integration in `apps/agent-runtime/src/agent_runtime/tools/browser.py`
2. **Secret management** — Keyring/Vault abstraction in a new `secrets` crate
3. **Log streaming** — WebSocket log tailing from `sandbox-core`
4. **CLI** — Thin wrapper over daemon API in `apps/cli` (new)
5. **Tests** — Unit + integration tests for all crates

### Development Workflow

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

## License

**MIT OR Apache-2.0** — Choose whichever suits your project.

This dual license ensures maximum compatibility:
- **MIT** — Simple, permissive, GPL-compatible
- **Apache-2.0** — Patent grant, better for corporate adoption

---

## Community

- **GitHub** — [github.com/agenticbox/agenticbox](https://github.com/agenticbox/agenticbox)
- **Discord** — [Coming Soon]
- **Twitter** — [@agenticbox](https://twitter.com/agenticbox)
- **Email** — hello@agenticbox.co

---

## Built With

- **Rust** — Daemon, sandbox, permissions, sessions
- **Tauri v2** — Native desktop (no Electron)
- **Python + FastAPI** — Agent runtime, MCP server
- **Playwright** — Browser automation (Phase 2)
- **SQLite** — Local persistence
- **Docker/containerd** — Sandbox runtime

---

> **AgenticBox** — Making agent deployment boring. That's the feature.