# AgenticBox — AGENTS.md

> **Context for autonomous coding agents (Pi, OpenCode, Claude Code, Codex, etc.) working on this repository.**
>
> This is a **stable index** — it rarely changes. Read it once to learn where everything lives, then go read the actual source files for current truth.

---

## What Is This Project?

Read these, in order:

1. **[README.md](./README.md)** — public intro, quick start, feature table
2. **[VISION.md](./VISION.md)** — core thesis, governance layer positioning, competitive moat
3. **[Cargo.toml](./Cargo.toml)** — workspace structure (all crates + apps), shared dependencies

---

## What Should I Work On?

**Single source of truth:** [`kanban.md`](./kanban.md)

It has 🔴 NOW / 🟡 NEXT / 🔵 LATER with explicit **"Done when"** criteria. Check it before every change.

---

## How Does the Architecture Work?

**Fastest overview:** the ASCII architecture diagram in [`README.md`](./README.md) (search for `┌─────────────────────────────────────────────────────┐`).

**Deep dive:** browse the crate directories under [`crates/`](./crates/) and app directories under [`apps/`](./apps/). Each has its own `Cargo.toml` and `src/`.

**Key technical decisions** (stable, won't churn):

| Decision | Rationale |
|----------|-----------|
| CLI talks directly to container runtime via bollard | Supports Docker + Podman (auto-detected). No daemon needed for `run`. Daemon is only for persistent sessions (`deploy`) |
| Agent CLI runs inside container, host relays stdio | Agent is sandboxed. No pre-built images per agent. |
| Runtime agent install via `[image].setup` commands | Wrap ANY agent — just write a TOML profile, no Dockerfile |
| `exec_interactive` with PTY support (crossterm) | Interactive agents (pi, hermes) get a real terminal |
| MCP for tools | Standard protocol, agent-agnostic |
| Centralized `PolicyEngine` evaluates Allow/Deny | Clean extension point for all permission types |
| `FsGuard` canonicalizes paths | Prevents escape via symlinks/`../` |
| CLI is a standalone binary | Works without the daemon |

---

## How Do I Build / Test / Run?

**Dev workflow:** [`scripts/dev.sh`](./scripts/dev.sh) — full stack (daemon + agent-runtime + desktop).

**Quick commands:**

| What | Command |
|------|---------|
| Daemon only (fastest) | `cargo run --bin daemon` |
| CLI | `cargo build --release --bin agenticbox -p agenticbox-cli` |
| All Rust (fmt + clippy + build + test) | `cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && cargo build && cargo test` |
| Desktop | `cd apps/desktop && pnpm tauri dev` |
| Agent runtime | `cd apps/agent-runtime && python -m agent_runtime.main` |

**CI workflow:** [`.github/workflows/ci.yml`](./.github/workflows/ci.yml) — runs fmt → clippy → build → test on every push/PR, then deploys `public/` to Pages on main.

---

## What Should I Know About the Codebase?

### Phase status (what works vs what's a stub)

Read the feature table in [`README.md`](./README.md) — it has ✅/⚠️/❌ for every component. No need to duplicate that here.

### Coding conventions (stable)

| Language | Conventions |
|----------|-------------|
| **Rust** | Edition 2021, resolver "2", `anyhow`/`thiserror`, `tracing`, `sqlx` (sqlite), `serde`. No panics in library code. |
| **Python** (agent-runtime) | FastAPI async, Pydantic v2, type hints everywhere, `uv` for deps |
| **TypeScript** (desktop) | React 18 + Tailwind, strict mode, pnpm workspace |
| **Git** | Branch from `main`, PR back to `main`. Conventional commits (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `ci:`). Linear history (rebase). |

### Testing

**Current state:** zero tests. Every new feature should include them. Priority order:
1. Unit tests for `policy-engine`, `fs-guard`, `network-control` (pure logic, no external deps)
2. Integration tests for `sandbox-core` (container lifecycle)
3. CLI tests with mock daemon

---

## Files Index (Read on Demand)

| File | What it contains | When to read it |
|------|-----------------|-----------------|
| [`README.md`](./README.md) | Project intro, feature table, quick start, architecture diagram | First time, and when checking what's shipped |
| [`VISION.md`](./VISION.md) | Core thesis, governance layer positioning, competitive analysis | Before making strategic contributions |
| [`kanban.md`](./kanban.md) | 🔴/🟡/🔵 priorities with "Done when" criteria | **Before every change** |
| [`Cargo.toml`](./Cargo.toml) | Workspace members, shared deps, build config | Before adding crates or changing deps |
| [`docs/designs/dx-user-journey.md`](./docs/designs/dx-user-journey.md) | The three modes (ad-hoc, named, daemon), container lifecycle, transport decisions | When modifying `agenticbox run` |
| [`soul.md`](./soul.md) | Hermes cofounder identity (mission, beliefs, decision framework) | **Hermes only** — not relevant for Pi/OpenCode/Codex |
| `agents/*/agent.toml` | Agent package manifests | When adding or modifying agent packages |
| [`founder_contract.md`](./founder_contract.md) | Founder–agent contract (daily minimums, scorecard, streak system) | Only if asked about gamification or progress tracking |
| [`game.json`](./game.json) / [`game.py`](./game.py) | RPG progression system (XP, levels, achievements, quests) | Only if asked about gamification |

---

## Known Pitfalls (Rarely Changes)

- **Tauri in CI**: Desktop crate needs GUI system libs (`webkit2gtk` etc.). Must be excluded from CI clippy/build. Use explicit `-p` flags, not `--workspace`.
- **GitHub Pages**: If the Pages site doesn't exist, `configure-pages` errors. Fix: add `enablement: true` in the action step + manual enable in repo Settings → Pages.
- **Docker dependency**: Sandbox requires Docker socket. CI needs Docker-in-Docker or mock.
- **No fake demos**: If a demo feature doesn't exist yet, label it "illustrative" or "Coming Soon". Never present animations/hardcoded output as real.
- **Cross-profile guard**: Hermes agent sessions have a cross-profile write guard. Don't modify another profile's data without explicit direction.
- **Always check `kanban.md`** before starting work — priorities shift frequently. Don't assume the feature you're about to build is still wanted.
