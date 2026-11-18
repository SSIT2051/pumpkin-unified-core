#!/bin/bash
set -e

SSH_KEY="${1:-$HOME/.ssh/id_ed25519}"
VM_HOST="${2:-pumpkin-vm}"
VM_USER="${3:-root}"
PLUGIN_PATH="target/wasm32-wasip2/release/pumpkin_unified_core.wasm"

if [ ! -f "$PLUGIN_PATH" ]; then
    echo "Plugin not found at $PLUGIN_PATH. Run 'cargo build --release --target wasm32-wasip2' first."
    exit 1
fi

echo "Deploying to $VM_USER@$VM_HOST..."

scp -i "$SSH_KEY" -o StrictHostKeyChecking=no "$PLUGIN_PATH" "$VM_USER@$VM_HOST:/root/pumpkin-server/plugins/pumpkin-unified-core.wasm.new"

ssh -i "$SSH_KEY" -o StrictHostKeyChecking=no "$VM_USER@$VM_HOST" << 'EOF'
    cd /root/pumpkin-server
    mv plugins/pumpkin-unified-core.wasm plugins/pumpkin-unified-core.wasm.$(date +%s).old 2>/dev/null || true
    mv plugins/pumpkin-unified-core.wasm.new plugins/pumpkin-unified-core.wasm
    sudo systemctl restart pumpkin
    sleep 10
    ./verify-plugin.sh
EOF

echo "Deployment complete!"