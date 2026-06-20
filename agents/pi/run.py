#!/usr/bin/env python3
"""Pi Agent — minimal runner for ACP transport test.

Reads JSON-RPC messages from stdin, responds on stdout.
This proves the bidirectional stdio pipe works for a real agent.
"""
import sys
import json

def main():
    # Signal readiness on stderr (separate channel)
    sys.stderr.write("PI_AGENT_READY\n")
    sys.stderr.flush()

    # Simple ACP-like loop: read requests, respond
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            msg = json.loads(line)
            method = msg.get("method", "")
            params = msg.get("params", {})

            if method == "initialize":
                response = {
                    "jsonrpc": "2.0",
                    "id": msg.get("id"),
                    "result": {
                        "agent": "pi",
                        "version": "0.1.0",
                        "capabilities": ["terminal", "filesystem", "network"],
                    },
                }
            elif method == "task":
                task = params.get("prompt", "")
                response = {
                    "jsonrpc": "2.0",
                    "id": msg.get("id"),
                    "result": {
                        "status": "accepted",
                        "message": f"Pi agent received task: {task}",
                    },
                }
            elif method == "shutdown":
                response = {
                    "jsonrpc": "2.0",
                    "id": msg.get("id"),
                    "result": {"status": "shutdown"},
                }
                sys.stdout.write(json.dumps(response) + "\n")
                sys.stdout.flush()
                break
            else:
                response = {
                    "jsonrpc": "2.0",
                    "id": msg.get("id"),
                    "error": {"code": -32601, "message": f"Unknown method: {method}"},
                }

            sys.stdout.write(json.dumps(response) + "\n")
            sys.stdout.flush()
        except json.JSONDecodeError:
            sys.stdout.write(json.dumps({
                "jsonrpc": "2.0",
                "error": {"code": -32700, "message": "Parse error"},
            }) + "\n")
            sys.stdout.flush()

if __name__ == "__main__":
    main()
