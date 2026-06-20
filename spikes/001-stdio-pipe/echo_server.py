#!/usr/bin/env python3
"""Echo server for ACP spike — reads JSON-RPC from stdin, echoes back with prefix."""
import sys
import json

sys.stderr.write("ECHO_SERVER_READY\n")
sys.stderr.flush()

for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    try:
        msg = json.loads(line)
        response = {
            "jsonrpc": "2.0",
            "id": msg.get("id"),
            "result": {
                "echo": msg.get("method", ""),
                "params": msg.get("params", {}),
            }
        }
        sys.stdout.write(json.dumps(response) + "\n")
        sys.stdout.flush()
    except json.JSONDecodeError:
        sys.stdout.write(json.dumps({
            "jsonrpc": "2.0",
            "error": {"code": -32700, "message": "Parse error"}
        }) + "\n")
        sys.stdout.flush()
