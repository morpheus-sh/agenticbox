# `agenticbox run` — User Journey & DX Design

> **Status:** Implemented (spike/acp-transport branch)
> **Date:** June 2026

---

## The Pitch (One Sentence)

`agenticbox run` wraps ANY agent CLI in a sandboxed Docker container — installs the agent at runtime, gives it a PTY, relays stdin/stdout, and cleans up on exit. The container is generic. The agent is decoupled.

---

## The Three Modes

### Mode 1: Ad-hoc (ship today)

```bash
agenticbox run -- python3 script.py
agenticbox run -- npm test
agenticbox run -- make build
```

**What happens:**
1. Pull container image (`python:3.12-slim` by default)
2. Mount `cwd` → `/workspace` (readonly by default)
3. Run the command inside the container
4. Stream stdout/stderr to terminal in real-time
5. Exit with the container's exit code
6. Clean up container

**No agent involved.** This is just "run a command in a sandbox." It's useful on its own and it's the foundation.

**Status:** ✅ Works. Tested on Windows + WSL Ubuntu.

```
▶ Wrapping command in sandbox
  cmd: echo hello from container
  Permissions: terminal=on  fs=readonly  network=allowlist(...)

✓  Container sandbox-873a2b50 (fs=ro, net=bridge)

hello from container

✓ Container exited (code 0)
```

---

### Mode 2: Named Agent (the product)

```bash
agenticbox run hermes
agenticbox run pi
agenticbox run my-custom-agent
```

**What happens:**
1. Read agent profile from `~/.agenticbox/agents/<name>/agent.toml`
2. Pull the base container image (e.g., `node:22-slim`)
3. Create + start container (`sleep infinity` to keep alive)
4. Run each `[image].setup` command inside the container via `docker exec`
5. Launch the agent via `docker exec` with interactive stdio
6. Relay host stdin → container stdin, container stdout → host stdout
7. On agent exit or stdin EOF: stop + remove container

**The agent CLI runs INSIDE the container.** The host relays I/O. No pre-built per-agent Docker images.

```
┌─────────────────────────────────┐
│  Host                           │
│                                 │
│  agenticbox CLI                  │
│       │                         │
│       ├── reads agent.toml      │
│       │                         │
│       ├── creates container ────────► ┌─────────────────────┐
│       │                         │   │ Container (sandbox)  │
│       │   stdin relay           │   │                     │
│       │──────────────────────────────►│  agent CLI runs     │
│       │                         │   │  here, sandboxed    │
│       │   stdout/stderr relay   │   │                     │
│       │◄──────────────────────────────│                     │
│       │                         │   │  /workspace (mount) │
│       └── exit code via inspect  │   └─────────────────────┘
│                                 │
└─────────────────────────────────┘
```

**Why runtime install instead of pre-built images?**
- You don't maintain N Dockerfiles for N agents
- No version drift between host and container
- Can "wrap ANY agent" — users add new agents by writing a TOML file
- The agent is decoupled from the sandbox

**Status:** ✅ Works. Tested with Pi agent (`curl -fsSL https://pi.dev/install.sh | sh`) on Windows + WSL Ubuntu. Real npm install (131 packages), real `pi` binary launched.

```
✓  Container sandbox-a3f2b1c4 (fs=rw, net=bridge)
↓  Installing agent: apt-get update && apt-get install -y curl
...  (apt output)
↓  Installing agent: curl -fsSL https://pi.dev/install.sh | sh
  Pi Installer
  npm install -g @earendil-works/pi-coding-agent
  added 131 packages in 11s
  Pi was installed successfully.
✓  Agent installed
▶  Launching agent: pi
✓  Agent exited cleanly
```

---

### Mode 3: Daemon (long-running sessions)

```bash
agenticbox deploy --name hermes --watch
agenticbox logs <session-id> -f
agenticbox stop <session-id>
```

**What happens:**
1. The daemon manages persistent containers
2. Sessions survive CLI exit
3. Logs are streamed on reconnect
4. Multiple agents can run concurrently

`run` is for interactive, foreground, ephemeral sessions. `deploy` is for background, persistent, managed sessions.

**Status:** ⚠️ Daemon exists but doesn't create containers yet. Separate workstream.

---

## Agent Profiles (`agent.toml`)

An agent profile defines everything `agenticbox run <name>` needs to know — without being a Docker image.

### `~/.agenticbox/agents/pi/agent.toml`

```toml
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

### Profile fields

| Field | What it does | Required |
|-------|-------------|----------|
| `command` | The command that launches the agent inside the container | Yes |
| `model.provider` | Which LLM provider | Yes |
| `model.model` | Which model | Yes |
| `model.api_key_env` | Env var name holding the API key | Yes |
| `model.base_url` | Optional endpoint override (Ollama, local, custom) | No |
| `permissions.*` | Permission model (terminal, filesystem, network, browser) | Defaults applied |
| `image.base` | Container base image (e.g., `node:22-slim`, `python:3.12-slim`) | Yes |
| `image.setup` | List of shell commands run inside the container before the agent starts | Yes |

Each `setup` command runs as `sh -c "<command>"` — pipes, flags, and `&&` chains all work.

---

## Model Endpoint Plumbing

The agent inside the container needs LLM access:

1. Read `api_key_env` from the profile (e.g., `ANTHROPIC_API_KEY`)
2. Read the actual key from the host's environment
3. Pass it into the container as an env var
4. If `base_url` is set, pass that too

**No proxy. No daemon middleman.** The key goes from host env → container env. The container can use it but network is allowlisted, preventing exfiltration.

---

## Container Lifecycle

```
create(image, cmd=["sleep","infinity"], mounts, env, network)
    │
    start()
    │
    for each setup command:
        exec_and_wait(["sh","-c",cmd]) → exit code
        if exit != 0: abort, remove container
    │
    exec_interactive(agent_cmd, tty=<is_terminal>)
    │  ┌─ relay stdin → container stdin
    │  └─ relay container stdout → host stdout
    │
    on agent exit or stdin EOF:
    │
    stop(timeout=3s)
    remove(force=true)
```

---

## TTY Support

When stdin is a real terminal:
- crossterm enters raw mode on the host
- Docker allocates a PTY for the exec (`tty: true`)
- Keystrokes pass through directly (no line buffering)
- Terminal resize, Ctrl+C, arrow keys all work

When stdin is piped (non-interactive):
- Line-based relay
- No raw mode, no PTY

Detection: `std::io::IsTerminal::is_terminal(&std::io::stdin())`

---

## What We're NOT Doing (Yet)

- **Pre-built Docker images per agent.** The container is generic. Agents install at runtime.
- **ACP permission interception.** Currently the agent runs with full stdio relay. Next step: parse JSON-RPC traffic and block/allow tool calls based on the permission profile.
- **Agent install caching.** `npm install` on every run takes 2-3 minutes. Deferred — will use named volumes for package manager cache.
- **The daemon for `run`.** `run` is foreground + ephemeral. `deploy` is for the daemon.

---

## Spikes

Two spikes validated the transport architecture before implementation:

### Spike 001: stdio pipe (`spikes/001-stdio-pipe/`)
**Validated:** bollard `exec` with `attach_stdin` + `attach_stdout` gives bidirectional pipe for JSON-RPC round-trips. Three messages successfully round-tripped between a Rust host and Python echo server inside a container.

### Spike 002: socket mount (`spikes/002-socket-mount/`)
**Validated:** container-to-container communication via mounted Unix socket works, but host socket mount is problematic on Windows Docker Desktop. **Decision: stdio for MVP.** Socket can be revisited for multi-agent scenarios.

---

## Next Steps

1. **ACP permission interception** — parse JSON-RPC, enforce allow/deny on tool calls
2. **Agent install caching** — named volumes for npm/pip cache
3. **Merge to main** — integrate spike/acp-transport back to main
4. **More agent profiles** — Claude Code, Codex, OpenCode
