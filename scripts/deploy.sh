#!/bin/bash
# Build-and-deploy helper: builds the wasip2 plugin and deploys it to pumpkin-vm.
# Heavy compilation is normally offloaded to GitHub Actions; this local path is
# for quick iteration. Uses the `pumpkin-vm` SSH alias (Ed25519 key auth).
set -e

PLUGIN_PATH="target/wasm32-wasip2/release/pumpkin_unified_core.wasm"

if [ ! -f "$PLUGIN_PATH" ]; then
    echo "Building wasip2 plugin..."
    cargo build --release --target wasm32-wasip2
fi

echo "Deploying to pumpkin-vm..."
bash "$(dirname "$0")/deploy-vm.sh" "$PLUGIN_PATH"