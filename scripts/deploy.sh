#!/bin/bash
set -e

SSH_KEY="${1:-$HOME/.ssh/id_ed25519}"
VM_HOST="${2:-pumpkin-vm}"
VM_USER="${3:-root}"
PLUGIN_PATH="target/x86_64-unknown-linux-gnu/release/libpumpkin_unified_core.so"

if [ ! -f "$PLUGIN_PATH" ]; then
    echo "Plugin not found at $PLUGIN_PATH. Run 'cargo build --release --target x86_64-unknown-linux-gnu' first."
    exit 1
fi

echo "Deploying to $VM_USER@$VM_HOST..."

scp -i "$SSH_KEY" -o StrictHostKeyChecking=no "$PLUGIN_PATH" "$VM_USER@$VM_HOST:/root/pumpkin-server/plugins/pumpkin-unified-core.so.new"

ssh -i "$SSH_KEY" -o StrictHostKeyChecking=no "$VM_USER@$VM_HOST" << 'EOF'
    cd /root/pumpkin-server
    mv plugins/pumpkin-unified-core.so plugins/pumpkin-unified-core.so.$(date +%s).old 2>/dev/null || true
    mv plugins/pumpkin-unified-core.so.new plugins/pumpkin-unified-core.so
    sudo systemctl restart pumpkin
    sleep 10
    ./verify-plugin.sh
EOF

echo "Deployment complete!"