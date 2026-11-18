#!/bin/bash
# WASM signing script for pumpkin-unified-core
# Uses cosign for keyless signing or key-based signing
#
# Usage: ./sign_wasm.sh [--keyless] [--key <path>] [--cert <path>]
#
# Options:
#   --keyless       Use keyless signing (OIDC via cosign)
#   --key <path>    Path to private key (PEM format)
#   --cert <path>   Path to certificate (PEM format)
#   --help          Show this help

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
WASM_PATH="$PROJECT_ROOT/target/wasm32-wasip2/release/pumpkin_unified_core.wasm"
SIGNED_PATH="$PROJECT_ROOT/target/wasm32-wasip2/release/pumpkin_unified_core.wasm.signed"

USE_KEYLESS=false
KEY_PATH=""
CERT_PATH=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --keyless)
            USE_KEYLESS=true
            shift
            ;;
        --key)
            KEY_PATH="$2"
            shift 2
            ;;
        --cert)
            CERT_PATH="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [--keyless] [--key <path>] [--cert <path>]"
            echo ""
            echo "Options:"
            echo "  --keyless       Use keyless signing (OIDC via cosign)"
            echo "  --key <path>    Path to private key (PEM format)"
            echo "  --cert <path>   Path to certificate (PEM format)"
            echo "  --help          Show this help"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

if [ ! -f "$WASM_PATH" ]; then
    echo "ERROR: WASM file not found at $WASM_PATH"
    echo "Run 'cargo build --release --target wasm32-wasip2' first"
    exit 1
fi

# Check for cosign
if ! command -v cosign &> /dev/null; then
    echo "ERROR: cosign not found. Install from https://github.com/sigstore/cosign"
    exit 1
fi

echo "Signing $WASM_PATH..."

if [ "$USE_KEYLESS" = true ]; then
    echo "Using keyless signing (OIDC)..."
    cosign sign-blob \
        --yes \
        --output-signature "$SIGNED_PATH.sig" \
        --output-certificate "$SIGNED_PATH.pem" \
        "$WASM_PATH"
else
    if [ -z "$KEY_PATH" ] || [ -z "$CERT_PATH" ]; then
        echo "ERROR: --key and --cert required for key-based signing"
        echo "Use --keyless for keyless signing"
        exit 1
    fi
    
    if [ ! -f "$KEY_PATH" ] || [ ! -f "$CERT_PATH" ]; then
        echo "ERROR: Key or certificate file not found"
        exit 1
    fi
    
    echo "Using key-based signing..."
    cosign sign-blob \
        --key "$KEY_PATH" \
        --cert "$CERT_PATH" \
        --output-signature "$SIGNED_PATH.sig" \
        --output-certificate "$SIGNED_PATH.pem" \
        "$WASM_PATH"
fi

# Verify signature
echo "Verifying signature..."
if [ "$USE_KEYLESS" = true ]; then
    cosign verify-blob \
        --certificate "$SIGNED_PATH.pem" \
        --signature "$SIGNED_PATH.sig" \
        "$WASM_PATH"
else
    cosign verify-blob \
        --key "$KEY_PATH" \
        --certificate "$SIGNED_PATH.pem" \
        --signature "$SIGNED_PATH.sig" \
        "$WASM_PATH"
fi

echo "Signature verified successfully!"
echo "Signed artifact: $SIGNED_PATH"
echo "Signature: $SIGNED_PATH.sig"
echo "Certificate: $SIGNED_PATH.pem"

# Create signed bundle (WASM + sig + cert)
BUNDLE_PATH="$PROJECT_ROOT/target/wasm32-wasip2/release/pumpkin_unified_core.wasm.bundle"
tar -czf "$BUNDLE_PATH" -C "$(dirname "$WASM_PATH")" \
    "$(basename "$WASM_PATH")" \
    "$(basename "$SIGNED_PATH.sig")" \
    "$(basename "$SIGNED_PATH.pem")"

echo "Bundle created: $BUNDLE_PATH"