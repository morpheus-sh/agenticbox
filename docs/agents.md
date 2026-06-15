# Agent Manifests

Agents in AgenticBox are just directories with a TOML manifest. Think of them like Docker images — shareable, forkable, and runnable with a single command.

## Quick Start

```bash
# Built-in demo — no setup needed
agenticbox run demo

# Run a named agent
agenticbox run hermes

# Wrap any command ad-hoc
agenticbox run -- python3 script.py

# List available agents
agenticbox agents

# Create a new agent
agenticbox init my-agent
```

## Directory Layout

```
~/.agenticbox/
└── agents/
    ├── hermes/
    │   └── agent.toml       ← manifest
    ├── pi/
    │   ├── agent.toml
    │   └── run.py            ← entry point script
    └── reviewer/
        └── agent.toml
```

Each agent lives in `~/.agenticbox/agents/<name>/`. The manifest file must be named `agent.toml`.

## Manifest Format

```toml
# Required
name = "my-agent"
description = "What this agent does"

# Command to execute when the agent starts
command = "python3 main.py"

# Model configuration
[model]
provider = "openai"              # openai | anthropic | local
model = "gpt-4o"
api_key_env = "OPENAI_API_KEY"   # env var name (not the key itself)

# Permission policy — what the agent CAN do
[permissions]
terminal = true                  # shell command execution
filesystem = "readonly"          # readonly | readwrite | none
browser = false                  # headless browser automation
network = "allowlist"            # allowlist | localhost | offline | full
domains = ["api.openai.com"]     # used when network = "allowlist"
```

## Permission Fields

| Field | Type | Values | Description |
|-------|------|--------|-------------|
| `terminal` | bool | `true` / `false` | Allow shell command execution |
| `filesystem` | string | `readonly` / `readwrite` / `none` | File system access level |
| `browser` | bool | `true` / `false` | Headless browser automation (Phase 2) |
| `network` | string | `allowlist` / `localhost` / `offline` / `full` | Outbound network policy |
| `domains` | array | `["api.openai.com", "github.com"]` | Allowed domains when `network = "allowlist"` |

## CLI Overrides

Any permission field can be overridden at runtime without editing the manifest:

```bash
# Run with read-write filesystem
agenticbox run hermes --fs readwrite

# Run with full network access
agenticbox run hermes --network full

# Run with specific domains only
agenticbox run hermes --domains "api.github.com,raw.githubusercontent.com"

# Run without terminal access
agenticbox run hermes --terminal false

# Run standalone (no daemon — simulated sandbox)
agenticbox run hermes --standalone
```

## Creating Agents

### Using `agenticbox init`

```bash
$ agenticbox init my-agent --command "python3 main.py"
✓ Created agent manifest: /home/user/.agenticbox/agents/my-agent/agent.toml

→ Edit the manifest, then run:
  agenticbox run my-agent
```

### Manual creation

```bash
mkdir -p ~/.agenticbox/agents/my-agent
cat > ~/.agenticbox/agents/my-agent/agent.toml << 'EOF'
name = "my-agent"
description = "My custom agent"
command = "python3 main.py"

[model]
provider = "openai"
model = "gpt-4o"

[permissions]
terminal = true
filesystem = "readonly"
network = "allowlist"
domains = ["api.openai.com"]
EOF
```

## Sharing Agents

Agents are just directories with TOML files. Share them by:

1. **Git repo** — push to GitHub, others clone into `~/.agenticbox/agents/`
2. **Copy** — `cp -r ~/.agenticbox/agents/hermes /somewhere/`
3. **Registry** — (planned) `agenticbox pull hermes` from a marketplace

### Example: Fork and modify

```bash
# Clone the example
cp -r ~/.agenticbox/agents/hermes ~/.agenticbox/agents/hermes-custom

# Edit permissions
vim ~/.agenticbox/agents/hermes-custom/agent.toml

# Run your fork
agenticbox run hermes-custom
```

## Example Agents

AgenticBox ships with example manifests in the `agents/` directory:

| Agent | Description | Permissions |
|-------|-------------|-------------|
| `hermes` | General-purpose coding assistant | terminal, readwrite, allowlist |
| `pi` | Edge / IoT computing agent | terminal, readonly, localhost |
| `reviewer` | Automated code reviewer | no terminal, readonly, allowlist |

Copy them to your agents directory:

```bash
cp -r agents/* ~/.agenticbox/agents/
```

## Three Ways to Run

```
┌──────────────────────────────────────────────────────────────┐
│  Layer 1: Built-in Demo                                      │
│  agenticbox run demo                                         │
│  → Zero config. Scripted agent attempts caught in real-time. │
│  → Screenshot-worthy colored ALLOWED/BLOCKED output.         │
├──────────────────────────────────────────────────────────────┤
│  Layer 2: Named Agent                                        │
│  agenticbox run hermes                                       │
│  → Resolves ~/.agenticbox/agents/hermes/agent.toml          │
│  → Deploys to sandbox with manifest permissions.            │
├──────────────────────────────────────────────────────────────┤
│  Layer 3: Ad-hoc Command                                     │
│  agenticbox run -- python3 script.py                        │
│  → Wraps any command in a sandbox                            │
│  → Defaults: terminal=on, fs=readonly, network=allowlist    │
└──────────────────────────────────────────────────────────────┘
```

## See Also

- [README.md](../README.md) — Overview and architecture
- [Permission Guards](../crates/fs-guard/) — Filesystem guard implementation
- [Network Control](../crates/network-control/) — Network policy enforcement
