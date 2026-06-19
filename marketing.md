# AgenticBox — Marketing & Positioning

## One-Liner

**AgenticBox: The workplace for AI agents.** Scoped permissions, bounded execution, full accountability. Give agents what they need to work — nothing more. Open source. Local-first. Run on your machine or ours.

## Problem Statement

AI agents are powerful but unbounded in production. Today every agent runs with your keys, your tokens, full filesystem access, no guardrails. The problem isn't that agents are smart — it's that they have no workplace. No scoped permissions. No boundaries. No accountability.

An agent needs:

- **Sandbox execution** — isolated environments with lifecycle control
- **Permissions** — what can the agent read, write, execute, browse?
- **Browser automation** — headless browsers for web interaction
- **Secret governance** — API keys, tokens, credentials injected securely at runtime
- **Observability & audit** — logs, metrics, traces per agent
- **Cost governance** — billing by usage, quotas, budget alerts

Most teams build a workplace from scratch or hack together Docker + custom scripts. AgenticBox makes it a solved problem.

## Target Customers

1. **Solo developers** building agent-powered products ($49/mo cloud beta)
2. **Teams** deploying multiple agents to production ($199/mo)
3. **Enterprises** needing custom SLAs, RBAC, on-prem ($999/mo)

## Competitive Landscape

| Competitor | Strength | Weakness |
|------------|----------|----------|
| Docker/K8s | Mature, widely adopted | Manual governance, DIY permissions |
| Cloudflare Workers AI | Edge deployment, fast | Limited to Cloudflare ecosystem, no agent governance |
| OpenAI Assistants API | Simple, well-documented | Vendor lock-in, expensive, no local control |
| LangGraph/LangChain | Rich agent tooling | No execution governance, no sandbox |

## Unique Value Propositions

1. **Workplace-first** — not just runtime, but boundaries: scoped permissions, isolated execution, full audit trail
2. **Local-first** — run on your own machine, no cloud required
3. **Vendor-neutral** — works with OpenAI, Ollama, vLLM, any model provider
4. **Rust-native** — fast, safe, low memory footprint
5. **Tauri desktop console** — beautiful governance UI without Electron bloat

## What We're NOT (and Why That Matters)

- Not a model provider (we work with OpenAI, Ollama, vLLM, any model)
- Not an agent framework (we don't dictate how agents are built)
- Not just containers (we add scoped permissions, browser, secrets, observability)
- Not cloud-only (local-first is core to our identity)
- Not "an AI employee" (that's identity — our moat, not our pitch today. We ship the workplace first.)

## Content Strategy

- Blog posts on agent workplace patterns
- Benchmark comparisons (AgenticBox vs Docker vs Cloudflare)
- Tutorial series: "Setting up your first agent workplace with AgenticBox"
- Case studies from early adopters
- X/Twitter updates about features and progress

## Launch Plan

1. **MVP launch** — Phase 1 features, open beta
2. **Community building** — Discord, GitHub discussions
3. **Paid tiers** — Starter ($49), Pro ($199), Enterprise ($999)
4. **Partnerships** — model providers (OpenRouter, Ollama)

## Passive Marketing via Repository

The repository itself is a marketing asset:

- Clean README with clear governance positioning
- Well-documented architecture and components
- Active development visible through commits
- Open source license (MIT OR Apache-2.0)
- Professional structure that signals quality

## Success Condition

When developers say "I'm using AgenticBox" and it means:

- My agents have a workplace
- They're bounded — they can only do what I authorized
- I control what they can touch
- I can see what they're doing
- I know how much they cost
- I don't have to think about any of that anymore

Then we've won.