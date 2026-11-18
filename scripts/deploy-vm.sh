#!/bin/bash
# Deploy pumpkin-unified-core.wasm to the pumpkin-vm.
#
# Uses the pre-configured `pumpkin-vm` SSH alias (Ed25519 key auth, SSH port
# 25726). NEVER use port 25491 (that is the Minecraft game port) or password
# auth.
#
# Procedure:
#   1. Upload the .wasm to the plugins folder.
#   2. Upload server.properties with RCON config.
#   3. Stop the running server with SIGINT (graceful).
#   4. Start a fresh background instance in /root/pumpkin-server.
#   5. Verify /tmp/pumpkin.log shows the plugin initializing without errors.
set -uo pipefail

WASM_PATH="${1:-/tmp/artifact/pumpkin_unified_core.wasm}"
SSH_HOST="${SSH_HOST:-pumpkin-vm}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [ ! -f "$WASM_PATH" ]; then
    echo "ERROR: wasm artifact not found at $WASM_PATH"
    exit 1
fi

echo "=== pumpkin-vm deploy ==="
echo "Target: $SSH_HOST"
echo "Plugin: $WASM_PATH ($(stat -c%s "$WASM_PATH") bytes)"

echo "=== Step 1: upload plugin ==="
scp -o StrictHostKeyChecking=no "$WASM_PATH" "$SSH_HOST:/root/pumpkin-server/plugins/pumpkin-unified-core.wasm" || exit 1

echo "=== Step 2: upload server.properties with RCON ==="
scp -o StrictHostKeyChecking=no "$SCRIPT_DIR/../config/server.properties" "$SSH_HOST:/root/pumpkin-server/server.properties" || exit 1

echo "=== Step 3: graceful restart ==="
ssh -o StrictHostKeyChecking=no "$SSH_HOST" 'bash -s' <<'RESTART'
set -uo pipefail
pkill -INT -f "./pumpkin" || true
sleep 4
cd /root/pumpkin-server
export PUMPKIN_CONFIG_DIR=/home/container/config
nohup ./pumpkin > /tmp/pumpkin.log 2>&1 </dev/null &
sleep 4
RESTART

echo "=== Step 4: verify plugin load ==="
ssh -o StrictHostKeyChecking=no "$SSH_HOST" 'bash -s' <<'VERIFY'
echo "--- pumpkin process ---"
ps aux | grep '[.]/pumpkin' | head -2 || echo "NOT RUNNING"
echo "--- plugin lines ---"
grep -i 'pumpkin-unified' /tmp/pumpkin.log | head -10
echo "--- errors (if any) ---"
grep -iE 'error|failed|panic' /tmp/pumpkin.log | grep -iv 'error_|no errors' | head -10 || echo "none"
VERIFY

echo "=== Step 5: verify RCON ==="
ssh -o StrictHostKeyChecking=no "$SSH_HOST" 'bash -s' <<'RCON'
sleep 2
echo "Testing RCON connection..."
if rcon-cli --host 127.0.0.1 --port 25575 --password changeme_secure_password "list" 2>/dev/null | grep -qiE "there are|players"; then
    echo "RCON: OK"
else
    echo "RCON: FAILED (may need server to fully start)"
fi
RCON

echo "=== Deploy complete ==="