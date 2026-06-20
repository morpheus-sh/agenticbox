# AgenticBox — Marketing & Positioning

## One-Liner

**AgenticBox: Deploy AI agents into production — safely.** Pick a template, connect your tools, set what the agent can do, ship it. Scoped permissions, bounded execution, full audit trail. Open source. Local-first. Run on your machine or ours.

## Problem Statement

Every company wants AI agents that do real work — touch real customer data, take real actions, move real money.

The agents are smart enough. The problem is deployment.

Today, putting an agent into production business operations means either:

- **Building custom guardrails from scratch** — expensive, slow, fragile, and every team reinvents the same thing
- **Handing the agent root access and hoping** — a security nightmare and a liability wall

The result: most agents never make it past the demo. They can write code in a sandbox but can't touch the production database. They can draft an email but can't send it. They can suggest a refund but can't process it.

AgenticBox makes production deployment a solved problem.

## How It Works

1. **Pick a vertical template** — support, ops, finance, sales. Each template ships with a sane permission set and tool connectors.
2. **Connect your tools** — helpdesk, CRM, database, email, API keys.
3. **Set what the agent can do** — plain-language limits: "can refund up to $50, cannot delete accounts."
4. **Deploy** — the agent runs in a bounded container with scoped permissions, isolated execution, and full audit trail.

No custom guardrails. No Docker expertise. No building governance from scratch.

## The Vercel for Agent Deployment

Vercel didn't build web apps. They made deployment so smooth that developers stopped managing servers.

AgenticBox doesn't build vertical agents. We make agent deployment into production so safe and smooth that companies stop building custom guardrails.

| Vercel | AgenticBox |
|--------|-----------|
| `git push` → live site | `agenticbox deploy` → agent in production |
| Edge network + serverless | Bounded container runtime (Docker/Podman) |
| DDoS protection, rate limits, auth | Permissions, scopes, audit trails |
| Next.js templates per use case | Vertical agent templates (support, ops, finance) |
| Developers ship web apps | Companies ship agent workforces |

## Vertical Agent Templates

Templates are the `create-next-app` moment for AI agents. Each template bundles:
- An agent profile (model, system prompt, tool set)
- MCP server connectors for relevant tools (Zendesk, Salesforce, Stripe, etc.)
- A default permission/governance profile tuned for the use case

**Wedge vertical: Customer Support / CX**
- #1 automation target for every company right now
- Requires scoped access to customer data, bounded actions (refund yes / delete no), full audit for compliance
- ROI is immediately measurable: tickets resolved, cost per ticket, response time
- Entry drug: once a company trusts an agent in support, they expand to sales ops, IT, finance

**Roadmap verticals:**
- Sales / Revenue Ops (CRM updates, prospecting, pipeline management)
- Internal IT / Employee Ops (provisioning, password resets, ticket handling)
- Finance / Accounting Ops (reconciliation, reporting, anomaly flagging)
- Compliance / Legal Ops (contract review, regulatory monitoring)

## Two Paths to the Same Engine

### Developer path (today)
Full control. CLI-driven. Write `agent.toml`, configure permissions explicitly, deploy via `agenticbox deploy`. Local-first — run on your machine, your cloud, your infrastructure.

### Non-dev path (tomorrow)
Outcome-driven. Pick a template, connect tools via UI, set limits in plain language, deploy. No code. No Docker. No permission schemas. Built for ops managers, CX leads, finance directors — the people who need agents in production but don't know what a container is.

Both paths run on the same engine. The non-dev path is what turns AgenticBox from developer tool into infrastructure company.

## Target Customers

1. **Developers** building agent-powered products — deploy via CLI, full control ($49/mo cloud beta)
2. **Semi-technical ops leads** deploying agents from templates — support managers, IT leads ($199/mo)
3. **Enterprises** needing custom SLAs, RBAC, on-prem, multiple agent teams ($999/mo)

## Competitive Landscape

| Competitor | What They Do | What They Lack |
|------------|-------------|----------------|
| Docker / K8s | Mature container runtime | No agent governance, no permissions, no audit — DIY everything |
| OpenAI Assistants API | Simple agent hosting | Vendor lock-in, no local control, no bounded execution |
| LangGraph / LangChain | Rich agent framework | No execution governance, no sandbox, no deployment safety |
| Cloudflare Workers AI | Edge model serving | No agent lifecycle, no permission system, no audit trail |

**The gap nobody fills:** the governance + deployment layer between "agent built" and "agent deployed in production." Frameworks build agents. Cloud providers serve models. Nobody makes the agent safe enough to touch real business systems.

## Unique Value Propositions

1. **Production-safe by default** — scoped permissions, bounded execution, full audit trail built in. Not an add-on.
2. **Vertical templates** — deploy a support agent or finance agent in minutes, not weeks.
3. **Local-first** — run on your own machine, your cloud, your infrastructure. No vendor lock-in.
4. **Vendor-neutral** — works with OpenAI, Ollama, vLLM, any model provider.
5. **Rust-native** — fast, safe, low memory footprint.
6. **Two paths** — full CLI control for devs, template-driven deployment for non-devs.

## What We're NOT

- Not a model provider (we work with any model)
- Not an agent framework (we don't dictate how agents are built)
- Not just containers (we add scoped permissions, governance, audit, templates)
- Not cloud-only (local-first is core to our identity)
- Not a sandbox/security tool (we're an enablement platform — governance is the mechanism, not the pitch)

## Content Strategy

- "Deploy your first production agent" tutorial series (per vertical)
- Benchmark comparisons (AgenticBox vs custom Docker vs OpenAI Assistants)
- Case studies: "How [company] deployed a support agent that handles 40% of tickets"
- Vertical deep-dives: "What permissions a support agent needs and why"
- X/Twitter updates about templates and deployments

## Launch Plan

1. **Core platform** — CLI, runtime, permissions, audit (shipping)
2. **First vertical template** — customer support agent (Zendesk/Intercom connector + permission profile)
3. **Community** — Discord, GitHub discussions, template contributions
4. **Paid tiers** — Starter ($49), Pro ($199), Enterprise ($999)
5. **Non-dev path** — template-driven UI deployment for ops leads
6. **Marketplace** — community-contributed vertical templates and tool connectors

## Passive Marketing via Repository

The repository itself is a marketing asset:

- Clean README with clear production-deployment positioning
- Well-documented architecture and governance model
- Vertical templates that are immediately usable
- Active development visible through commits
- Open source license (MIT OR Apache-2.0)
- Professional structure that signals quality

## Success Condition

When a company says "we deployed an agent on AgenticBox" and it means:

- The agent is doing real work in production — touching real systems, taking real actions
- It's bounded — it can only do what was authorized
- It's auditable — every action is logged and attributable
- It was fast — picked a template, connected tools, deployed
- Nobody had to build custom guardrails

Then we've won.
