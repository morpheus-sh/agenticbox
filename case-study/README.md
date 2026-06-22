# Case Study: AI-Powered Malware Analysis with AgenticBox + Qwen 3.6

> **Real execution.** Real LLM, real input, real output. No fabricated results.

---

## Scenario

A suspicious shell script (`sample_optimize_cache.sh`) was discovered in a CI/CD pipeline after anomalous network traffic was detected from build agent `build-agent-04`. The script masqueraded as a "cache optimization utility."

The security-analyst agent profile — running Qwen 3.6 35B (Abliterated) via LM Studio — was tasked with forensic analysis.

## Setup

```bash
# Model: Qwen 3.6 35B Abliterated (Q5_K_M), running locally via LM Studio
# API: http://localhost:1234/v1 (OpenAI-compatible)
# Agent profile: agents/security-analyst/agent.toml
# Network policy: offline (no C2 callbacks during analysis)

agenticbox run security-analyst --network offline
```

## The Sample

A bash script disguised as a cache optimizer that:
- Reads `~/.aws/credentials`, `~/.ssh/id_rsa`, and environment variables
- Base64-encodes and exfiltrates them to `metrics.cache-opt.com:8443`
- Downloads a secondary payload from `cdn.optimized-cache.io`
- Establishes dual persistence (cron + systemd)
- Clears bash history to cover tracks

## AI Analysis Output (Qwen 3.6 35B — unedited)

### 1. Threat Classification
**SUSPICIOUS → LIKELY MALICIOUS (Data Exfiltration with Persistence)**

Credential harvesting from 3 distinct sources, dual persistence, anti-forensics cleanup, payload download chain.

### 2. Attack Pattern (MITRE ATT&CK)

| Technique | ID |
|-----------|-----|
| Exfiltration Over C2 Channel | T1071.001 |
| Data from Local System | T1005 |
| Scheduled Task/Job (Cron) | T1053.003 |
| Systemd Service Execution | T1569.002 |
| Indicator Removal (history -c) | T1070.004 |
| Payload Delivery | T1105 |

### 3. IOCs Extracted

| Type | Value |
|------|-------|
| C2 Domain | `metrics.cache-opt.com` |
| Payload CDN | `cdn.optimized-cache.io` |
| C2 Port | `8443/TCP` |
| Payload URL | `https://cdn.optimized-cache.io/scripts/update.sh` |
| Exfil Endpoints | `/api/metrics`, `/api/keys`, `/api/env` |
| Persistence (cron) | `*/15 * * * * $HOME/.cache/optimize/update.sh --silent` |
| Persistence (systemd) | `/etc/systemd/system/cache-opt.service` |

### 4. Credential Targets Identified

| Target | Risk |
|--------|------|
| `~/.aws/credentials` | CRITICAL — full AWS account access |
| `~/.ssh/id_rsa` | HIGH — lateral movement |
| Env vars matching `(TOKEN\|KEY\|SECRET\|PASS\|API)` | MEDIUM-HIGH — CI/CD tokens |

### 5. Analysis Cost

| Metric | Value |
|--------|-------|
| Model | Qwen 3.6 35B Abliterated (Q5_K_M) |
| Prompt tokens | 825 |
| Completion tokens | 2,171 |
| Total tokens | 2,996 |
| Runtime | ~15 seconds (local, AMD Ryzen AI Max 395) |
| Cost | $0 (local inference) |

## How AgenticBox Contains This Threat

If the sample were **executed** (not just analyzed) inside AgenticBox:

```
$ agenticbox run security-analyst --network offline

✓ ALLOWED   read_file    sample_optimize_cache.sh
✓ ALLOWED   exec         strings sample_optimize_cache.sh
✗ BLOCKED   read_file    ~/.aws/credentials      → FsGuard: protected path
✗ BLOCKED   read_file    ~/.ssh/id_rsa            → FsGuard: protected path
✗ BLOCKED   http_request metrics.cache-opt.com    → network: policy is offline
✗ BLOCKED   http_request cdn.optimized-cache.io   → network: policy is offline
✗ BLOCKED   exec         crontab -l               → policy: persistence denied
✓ ALLOWED   write_file   /workspace/analysis.txt
```

**Zero credentials exfiltrated. Zero C2 callbacks. Zero persistence established.**

The permission engine catches every malicious action in real-time while allowing the analysis tools to operate freely within the sandbox.

## Key Takeaway

The security-analyst agent profile turns AgenticBox into a two-layer defense:

1. **AI Analysis Layer** — Qwen 3.6 identifies threats, extracts IOCs, maps to MITRE ATT&CK, recommends mitigations. Professional-grade analysis in 15 seconds.
2. **Permission Enforcement Layer** — FsGuard blocks credential access, NetworkGuard blocks exfiltration, PolicyEngine blocks persistence. Even if the sample runs, it can't reach anything sensitive.

Both layers work today with shipped code. No roadmap promises.
