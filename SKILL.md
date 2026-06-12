---
name: agenticbox
description: AgenticBox — Vercel for AI agents. Local-first, vendor-neutral sandbox runtime for autonomous AI agents with browser automation, filesystem tools, and terminal access.
category: infrastructure
---

# AgenticBox Company Brain

## Identity
- **Domain**: agenticbox.co
- **Positioning**: "Vercel for AI agents" — deploy agents without worrying about sandboxes, permissions, browser sessions, secret management, observability, cost controls
- **Target audience**: Developers and companies deploying AI agents to production
- **Tech stack**: Rust (daemon + crates), Tauri v2 desktop, Python FastAPI agent runtime, Docker/Rancher Desktop, SQLite
- **License**: MIT OR Apache-2.0

## Pricing Model
| Tier | Price | Target |
|------|-------|--------|
| Starter | $49/mo | Solo devs, small projects |
| Pro | $199/mo | Teams, production workloads |
| Enterprise | $999/mo | Large orgs, custom SLAs |

## Key Features (by phase)
- **Phase 1** (Ready): Container sandbox lifecycle, terminal tool, filesystem mounts + guard, OpenAI-compatible API, Tauri desktop app, session persistence
- **Phase 2** (Planned): Browser automation (Playwright), permission system UI
- **Phase 3** (Planned): Firecracker microVMs, policy engine

## Workflow
1. Load this skill when working on AgenticBox
2. Check `docs/ARCHITECTURE.md` for component details
3. Use `cargo run --bin daemon` for daemon-only dev
4. Use `cd apps/desktop && pnpm tauri dev` for desktop dev
5. Run `./scripts/dev.sh` for full stack

## Marketing Notes
- Emphasize "local-first, vendor-neutral" — no cloud lock-in
- Highlight Rust performance + Tauri desktop experience
- Target: developers who are tired of managing agent infrastructure
- Content strategy: blog posts on agent deployment patterns, sandbox security, benchmark comparisons

## Supermemory Container Tag
Use `containerTag: "agenticbox"` when storing memories about this project.
