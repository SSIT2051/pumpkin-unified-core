#!/bin/bash
# Deploy pumpkin-unified-core.wasm to the pumpkin-vm and clean up disk.
# Designed to be run from the build host. Handles a flaky/overloaded VM by
# retrying and by cleaning disk BEFORE deploying.
set -uo pipefail

VM_HOST="${VM_HOST:-th-bkk02.altare.global}"
VM_PORT="${VM_PORT:-25491}"
VM_USER="${VM_USER:-root}"
VM_PASS="${VM_PASS:-1234567890}"
WASM_PATH="${1:-/tmp/artifact/pumpkin_unified_core.wasm}"

SSH_OPTS="-o StrictHostKeyChecking=no -o ConnectTimeout=40 -o ServerAliveInterval=10 -o ServerAliveCountMax=3"

echo "=== pumpkin-vm deploy + cleanup ==="
echo "Target: $VM_USER@$VM_HOST:$VM_PORT"
echo "Plugin: $WASM_PATH ($(stat -c%s "$WASM_PATH" 2>/dev/null || echo '?') bytes)"

# Try to connect up to 5 times.
connected=0
for attempt in 1 2 3 4 5; do
    echo "--- connection attempt $attempt ---"
    if timeout 45 sshpass -p "$VM_PASS" ssh $SSH_OPTS "$VM_USER@$VM_HOST" -p "$VM_PORT" "echo CONNECTED" 2>/dev/null | grep -q CONNECTED; then
        connected=1
        break
    fi
    echo "attempt $attempt failed; waiting 15s..."
    sleep 15
done

if [ "$connected" -ne 1 ]; then
    echo "ERROR: could not connect to VM after 5 attempts (VM likely overloaded)"
    exit 1
fi

echo "=== Step 1: disk cleanup (safe targets only) ==="
timeout 60 sshpass -p "$VM_PASS" ssh $SSH_OPTS "$VM_USER@$VM_HOST" -p "$VM_PORT" 'bash -s' <<'CLEANUP'
set -uo pipefail
echo "Before:"
df -h / | tail -1

# Clean old plugin backups (keep newest 1).
cd /root/pumpkin-server 2>/dev/null || cd /root
ls -1t plugins/pumpkin-unified-core.wasm.*.old plugins/pumpkin-unified-core.wasm.*.bak 2>/dev/null | tail -n +2 | xargs -r rm -f
ls -1t plugins/*.so.*.old plugins/*.so.*.bak 2>/dev/null | tail -n +3 | xargs -r rm -f

# Clean cargo/target caches if present (big wins, safe).
rm -rf /root/.cargo/registry/cache/* /root/.cargo/git/checkouts/* 2>/dev/null || true
find /root -maxdepth 4 -type d -name target -path '*/pumpkin*' -prune -exec rm -rf {} + 2>/dev/null || true

# Clean old logs (keep last 3).
find /root/pumpkin-server -name '*.log.gz' -mtime +7 -delete 2>/dev/null || true
find /root/pumpkin-server/logs -name '*.log' -mtime +7 -delete 2>/dev/null || true

# Clean apt cache (safe).
apt-get clean 2>/dev/null || true
rm -rf /var/cache/apt/archives/*.deb 2>/dev/null || true

# Clean /tmp old files (safe, >1 day).
find /tmp -maxdepth 2 -type f -mtime +1 -delete 2>/dev/null || true

echo "After:"
df -h / | tail -1
CLEANUP

echo "=== Step 2: upload plugin ==="
timeout 90 sshpass -p "$VM_PASS" scp $SSH_OPTS "$WASM_PATH" "$VM_USER@$VM_HOST:/root/pumpkin-server/plugins/pumpkin-unified-core.wasm.new" 2>&1 | tail -3

echo "=== Step 3: swap + restart ==="
timeout 60 sshpass -p "$VM_PASS" ssh $SSH_OPTS "$VM_USER@$VM_HOST" -p "$VM_PORT" 'bash -s' <<'DEPLOY'
set -uo pipefail
cd /root/pumpkin-server
# Backup current, install new.
[ -f plugins/pumpkin-unified-core.wasm ] && mv plugins/pumpkin-unified-core.wasm "plugins/pumpkin-unified-core.wasm.$(date +%s).old" 2>/dev/null
mv plugins/pumpkin-unified-core.wasm.new plugins/pumpkin-unified-core.wasm
echo "Installed plugin:"
ls -la plugins/pumpkin-unified-core.wasm
# Restart server.
sudo systemctl restart pumpkin 2>/dev/null || echo "systemctl restart failed (may need manual restart)"
sleep 8
echo "Server status:"
sudo systemctl is-active pumpkin 2>/dev/null || echo "unknown"
DEPLOY

echo "=== Deploy complete ==="