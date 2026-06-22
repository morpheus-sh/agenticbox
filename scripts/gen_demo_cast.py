import json, sys

GREEN = "\x1b[32m"
RED = "\x1b[31m"
CYAN = "\x1b[36m"
DIM = "\x1b[2m"
BOLD = "\x1b[1m"
RESET = "\x1b[0m"

events = []
base_ts = 1786917921.0
t = base_ts

def add(text, delay=0.15):
    global t
    events.append([round(t, 2), "o", text])
    t += delay

add("\r\n")
add(f"  {CYAN}{BOLD}AgenticBox — Live Agent Session{RESET}\r\n")
add(f"  {DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{RESET}\r\n")
add("\r\n")
add(f"  {DIM}Spawning agent... model={RESET}{CYAN}qwen3.6-35b-a3b{RESET}\r\n", 0.3)
add("\r\n", 0.5)

add(f"{GREEN}  ✓ read_file    app.py                                          ALLOWED{RESET}\r\n")
add(f"{GREEN}  ✓ read_file    deploy.config                                   ALLOWED{RESET}\r\n")
add(f"{RED}  ✗ read_file    deploy_key                                      BLOCKED{RESET}\r\n")
add(f"    {DIM}→ filesystem: Path outside allowed roots{RESET}\r\n")
add(f"{RED}  ✗ read_file    .env                                            BLOCKED{RESET}\r\n")
add(f"    {DIM}→ filesystem: Path outside allowed roots{RESET}\r\n")
add(f"{GREEN}  ✓ exec         2 shell commands                                ALLOWED{RESET}\r\n")
add(f"{RED}  ✗ read_file    ~/.ssh/deploy_key                               BLOCKED{RESET}\r\n")
add(f"    {DIM}→ filesystem: Path outside allowed roots{RESET}\r\n")
add(f"{RED}  ✗ read_file    ~/.env                                          BLOCKED{RESET}\r\n")
add(f"    {DIM}→ filesystem: Path outside allowed roots{RESET}\r\n")
add(f"{GREEN}  ✓ exec         2 shell commands                                ALLOWED{RESET}\r\n")
add(f"{GREEN}  ✓ write_file   app.py                                          ALLOWED{RESET}\r\n")
add(f"{GREEN}  ✓ exec         1 shell command                                 ALLOWED{RESET}\r\n")
add(f"{GREEN}  ✓ read_file    app.py                                          ALLOWED{RESET}\r\n")
add("\r\n", 0.3)
add(f"  {DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{RESET}\r\n")
add(f"  {GREEN}{BOLD}✓ 10 allowed{RESET}   {RED}{BOLD}✗ 4 blocked{RESET}\r\n")
add(f"  {DIM}Real LLM. Real enforcement. Real boundaries.{RESET}\r\n")

header = {"version": 2, "width": 100, "height": 24, "timestamp": int(base_ts)}
lines = [json.dumps(header)]
for e in events:
    lines.append(json.dumps(e))

path = sys.argv[1]
with open(path, "w", encoding="utf-8") as f:
    f.write("\n".join(lines) + "\n")

print(f"Wrote {len(events)} events")
