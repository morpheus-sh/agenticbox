#!/usr/bin/env python3
"""
Spike 002: Test if Unix sockets can be mounted into Docker containers
on Docker Desktop for Windows.

Strategy: 
1. Create a Unix socket in a shared volume
2. Start a server listening on that socket (in a container)
3. Start a client connecting to that socket (in another container)
4. Verify message round-trip
"""
import socket
import os
import threading
import time
import json

SOCKET_PATH = "/tmp/spike.sock"

def server():
    """Listen on the Unix socket and echo messages back."""
    if os.path.exists(SOCKET_PATH):
        os.remove(SOCKET_PATH)
    
    server_sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    server_sock.bind(SOCKET_PATH)
    server_sock.listen(1)
    server_sock.settimeout(10)
    
    print(f"[server] Listening on {SOCKET_PATH}")
    
    try:
        conn, _ = server_sock.accept()
        print("[server] Client connected!")
        
        data = conn.recv(4096)
        msg = json.loads(data.decode())
        print(f"[server] Received: {msg}")
        
        response = {"jsonrpc": "2.0", "id": msg.get("id"), "result": {"echo": msg.get("method")}}
        conn.sendall(json.dumps(response).encode())
        print(f"[server] Sent: {response}")
        
        conn.close()
    except socket.timeout:
        print("[server] Timeout — no client connected")
    finally:
        server_sock.close()
        if os.path.exists(SOCKET_PATH):
            os.remove(SOCKET_PATH)

if __name__ == "__main__":
    import sys
    mode = sys.argv[1] if len(sys.argv) > 1 else "server"
    
    if mode == "server":
        server()
    elif mode == "client":
        # Client mode — connect and send a message
        time.sleep(2)  # Wait for server to start
        client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        try:
            client.connect(SOCKET_PATH)
            msg = {"jsonrpc": "2.0", "id": 1, "method": "test"}
            client.sendall(json.dumps(msg).encode())
            response = client.recv(4096)
            print(f"[client] Response: {json.loads(response.decode())}")
        except Exception as e:
            print(f"[client] Error: {e}")
        finally:
            client.close()
