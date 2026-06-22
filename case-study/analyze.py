import json, urllib.request, sys

sample_path = sys.argv[1] if len(sys.argv) > 1 else "sample_optimize_cache.sh"
with open(sample_path, "r") as f:
    sample = f.read()

system_prompt = """You are an expert security analyst specializing in malware analysis and threat research.
Analyze the provided shell script as if it were submitted for forensic examination.

Provide your analysis in this structured format:
1. THREAT CLASSIFICATION (malicious/benign/suspicious)
2. ATTACK PATTERN (MITRE ATT&CK style description)
3. INDICATORS OF COMPROMISE (IOCs - domains, IPs, URLs, file paths)
4. MALICIOUS BEHAVIORS (list each with explanation)
5. CREDENTIAL/DATA TARGETS (what sensitive data it tries to access)
6. PERSISTENCE MECHANISMS (how it survives reboot)
7. RECOMMENDED MITIGATIONS"""

user_msg = f"""Analyze the following shell script found in a CI/CD pipeline after suspicious network activity was detected.
The script was named 'sample_optimize_cache.sh' and was executed by user 'jenkins-user@build-agent-04'.

```bash
{sample}
```"""

payload = {
    "model": "huihui-qwen3.6-35b-a3b-claude-4.7-opus-abliterated-mtp@q5_k",
    "messages": [
        {"role": "system", "content": system_prompt},
        {"role": "user", "content": user_msg}
    ],
    "temperature": 0.3,
    "max_tokens": 4096
}

req = urllib.request.Request(
    "http://localhost:1234/v1/chat/completions",
    data=json.dumps(payload).encode("utf-8"),
    headers={"Content-Type": "application/json"},
)

print("Calling Qwen 3.6 35B (Abliterated) via LM Studio...")
print("=" * 70)

with urllib.request.urlopen(req, timeout=180) as resp:
    result = json.loads(resp.read().decode("utf-8"))

analysis = result["choices"][0]["message"]["content"]
print(analysis)
print("=" * 70)
print(f"\nModel: {result['model']}")
usage = result.get("usage", {})
print(f"Tokens — prompt: {usage.get('prompt_tokens', 'N/A')}, completion: {usage.get('completion_tokens', 'N/A')}, total: {usage.get('total_tokens', 'N/A')}")
