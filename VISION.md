# Vercel for AI Agents — The Core Thesis

## What Vercel Solved (and Why It Matters)

Before Vercel, deploying a web app meant:

- Setting up servers
- Configuring load balancers
- Managing SSL certificates
- Handling caching and CDN
- Scaling horizontally
- Monitoring uptime

Vercel made this invisible. You pushed code, it worked. That's the pattern we're replicating for AI agents.

## What Vercel Didn't Solve (and Why It Matters)

Vercel is great at serving static content and simple APIs. But AI agents need something different:

- **Sandboxing** — isolated execution environments
- **Permissions** — what can the agent read, write, execute?
- **Browser automation** — headless browsers for web interaction
- **Secret management** — API keys, tokens, credentials
- **Observability** — logs, metrics, traces per-agent
- **Cost controls** — billing by usage, not just compute

These are the problems Vercel doesn't address. They're the problems AI agents face every day.

## The Core Insight

"Vercel for AI agents" means:

> Deploy autonomous agents without worrying about sandboxes, permissions, browser sessions, secret management, observability, or cost controls.

It's not just a metaphor. It's a real product positioning because it maps to something developers already understand.

## What This Means for the Product

### Phase 1 (Now)
- Container sandbox lifecycle
- Terminal tool + filesystem mounts
- OpenAI-compatible API
- Tauri desktop app
- Session persistence

**This is the "Vercel" part.** It's the core deployment experience.

### Phase 2 (Planned)
- Browser automation (Playwright)
- Permission system UI
- Secret management
- Basic observability

**This is where we differentiate from generic containers.**

### Phase 3 (Future)
- Firecracker microVMs
- Policy engine
- Advanced cost controls
- Multi-agent orchestration

**This is where we become essential infrastructure.**

## Why This Positioning Works

1. **It's specific** — not "AI platform" or "agent framework"
2. **It's memorable** — developers know Vercel, they get it instantly
3. **It scales** — the metaphor holds from Phase 1 through Phase 3
4. **It's defensible** — competitors can copy features but not the positioning

## What It Doesn't Mean (and Why That Matters)

- We're NOT a model provider (we work with OpenAI, Ollama, vLLM, any model)
- We're NOT a framework (we don't dictate how agents are built)
- We're NOT just containers (we add permissions, browser automation, observability)
- We're NOT cloud-only (local-first is core to our identity)

## The Competitive Moat

Vercel's moat isn't technology — it's **developer experience**. They made deployment so easy that developers stopped thinking about infrastructure.

AgenticBox's moat will be the same: making agent deployment so simple that developers stop worrying about sandboxes, permissions, and browser sessions.

## Pricing Strategy (Aligned with Vercel)

| Tier | Price | Target |
|------|-------|--------|
| Starter | $49/mo | Solo devs, small projects |
| Pro | $199/mo | Teams, production workloads |
| Enterprise | $999/mo | Large orgs, custom SLAs |

This mirrors Vercel's own pricing structure. It feels natural to developers.

## The Long Game

Vercel started as a deployment tool and became the default platform for web apps.

AgenticBox should start as a sandbox runtime and become the default platform for AI agents.

The path is clear:
1. Solve the core problem (sandboxing + permissions)
2. Add differentiation (browser automation, observability)
3. Scale to the full stack (policy engine, multi-agent orchestration)
4. Become essential infrastructure

## Success Condition

When developers say "I'm using AgenticBox" and it means:
- My agents are deployed
- They're sandboxed
- I can see what they're doing
- I know how much they cost
- I don't have to think about any of that anymore

Then we've won.
