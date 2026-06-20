# Spike 001: ACP over stdio pipe

## Question
Can we use bollard to exec into a Docker container and relay bidirectional
stdin/stdout, enabling ACP-style JSON-RPC communication between the host
and an agent running inside the container?

## Why this matters
The entire architecture depends on the host being able to talk to an agent
CLI running inside a container. If bollard can't do interactive exec with
stdin/stdout piping, we need a different transport (socket, HTTP, etc.).

## Approach
1. Start a container with python:3.12-slim
2. Use bollard's `exec_container` with `attach_stdin: true, attach_stdout: true`
3. Inside the container, run a simple JSON-RPC echo server that reads from stdin, writes to stdout
4. From the host, send JSON-RPC requests and verify responses come back

## Verdict: VALIDATED

### What worked
- `docker exec -i` pipes stdin/stdout bidirectionally — confirmed from CLI
- bollard `start_exec` with `attach_stdin + attach_stdout` — full bidirectional pipe in Rust
- 3/3 JSON-RPC messages round-tripped: host sends → container echoes → host parses
- Concurrent send + receive via `tokio::spawn` works cleanly with 10s timeout
- stderr arrives on a separate channel (ECHO_SERVER_READY signal seen before stdout)

### What didn't
- First attempt was sequential (send all, then read) — timed out. Fixed with concurrent sender/receiver tasks.

### Surprises
- bollard's `StartExecResults::Attached` gives you `output` (stream) and `input` (AsyncWrite) directly — no extra plumbing needed
- stderr is multiplexed into the same stream, distinguished by `LogOutput::StdErr` — useful for agent status signals

### Recommendation for the real build
**Use stdio as the ACP transport.** It's the simplest, most portable approach:
1. `agenticbox` creates container, installs agent, then `exec`s the agent CLI
2. Host relays ACP JSON-RPC over the exec's stdin/stdout
3. Permission decisions are intercepted in the relay layer (host-side)
4. No sockets, no ports, no network config needed
