# Spike 002: Unix socket mount

## Question
Can we mount a Unix socket into a Docker container for agent-to-host
communication, as an alternative to stdio piping?

## Why this matters
If stdio doesn't work for some agents (e.g., agents that need a persistent
connection, or multi-agent scenarios), a Unix socket is the fallback transport.

## Risk
HIGH on Windows — Docker Desktop runs containers in a Linux VM. Unix sockets
on the host (Windows) can't be directly mounted into the Linux VM. This may
require a TCP socket instead, or may not be feasible on Windows at all.

## Approach
1. Create a Unix socket server on the host (or in a helper container)
2. Mount the socket file into a container
3. Have the container connect to the socket and send a message
4. Verify the host receives it

## Verdict: VALIDATED (container-to-container) / NOT NEEDED for MVP

### What worked
- Two containers sharing a Unix socket via a Docker named volume — full round-trip
- Server container creates `/tmp/spike.sock` in the shared volume
- Client container connects to the same socket path, sends JSON, gets response

### What didn't
- Mounting a host Unix socket into a container on Windows was not tested (blocked by user). On Docker Desktop for Windows, host Unix sockets likely don't work because the Docker daemon runs in a Linux VM. TCP sockets would be the cross-platform alternative.

### Surprises
- Container-to-container Unix sockets via shared volumes "just work" — no special config
- Could be useful for multi-agent scenarios where agents need to talk to each other

### Recommendation for the real build
**Not needed for MVP.** stdio (spike 001) is the right transport for single-agent scenarios.
Socket between containers is a viable option for multi-agent coordination later.
Host-to-container socket mounting on Windows is problematic — use stdio or TCP instead.
