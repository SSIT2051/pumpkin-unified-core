#!/bin/bash
# Verification suite for pumpkin-unified-core (WASM plugin).
# Runs via RCON against the pumpkin-vm (or any PumpkinMC server).
#
# Currently covers the ported modules: economy + LuckPerms (/lp).
# Additional module sections are added as they are ported.
#
# Environment:
#   RCON_HOST      RCON host (default 127.0.0.1)
#   RCON_PORT      RCON port (default 25575)
#   RCON_PASSWORD  RCON password (default pumpkin)
#
# Exit code: 0 if all tests pass, 1 otherwise.
set -uo pipefail

RCON_HOST="${RCON_HOST:-127.0.0.1}"
RCON_PORT="${RCON_PORT:-25575}"
RCON_PASSWORD="${RCON_PASSWORD:-pumpkin}"

PASS=0
FAIL=0
FAILED_TESTS=()

rcon() {
    rcon-cli --host "$RCON_HOST" --port "$RCON_PORT" --password "$RCON_PASSWORD" "$1" 2>/dev/null
}

pass() { PASS=$((PASS + 1)); echo "  PASS  $1"; }
fail() { FAIL=$((FAIL + 1)); FAILED_TESTS+=("$1"); echo "  FAIL  $1 :: $2"; }

assert_contains() {
    local name="$1" expected="$2" actual="$3"
    if printf '%s' "$actual" | grep -qiF -- "$expected"; then
        pass "$name"
    else
        fail "$name" "expected '$expected', got: $(printf '%s' "$actual" | head -n1)"
    fi
}

assert_known() {
    local name="$1" command="$2" out
    out="$(rcon "$command")"
    if printf '%s' "$out" | grep -qiE "unknown command|unknown console command|no such command|unknown plugin command"; then
        fail "$name" "command not recognized"
    else
        pass "$name"
    fi
}

assert_cmd_contains() {
    local name="$1" command="$2" expected="$3" out
    out="$(rcon "$command")"
    assert_contains "$name" "$expected" "$out"
}

echo "==================================================================="
echo " Pumpkin Unified Core - Verification Suite (WASM)"
echo " Target: ${RCON_HOST}:${RCON_PORT}"
echo "==================================================================="

# 0. Wait for server
echo ""
echo "[0] Waiting for server..."
ready=0
for _ in $(seq 1 30); do
    if rcon "list" 2>/dev/null | grep -qiE "there are|players"; then
        ready=1
        break
    fi
    sleep 2
done
if [ "$ready" -ne 1 ]; then
    echo "ERROR: server not ready via RCON after 60s"
    exit 1
fi
echo "  Server is ready"

# 1. Plugin loaded
echo ""
echo "[1] Plugin load"
plugins_out="$(rcon "plugins")"
assert_contains "Plugin loaded" "pumpkin-unified-core" "$plugins_out"

# 2. Economy (functional - works for offline players)
echo ""
echo "[2] Economy"
assert_cmd_contains "eco give" "eco give testuser 1000" "1000"
assert_cmd_contains "balance" "balance testuser" "1000"
assert_cmd_contains "eco take" "eco take testuser 200" "800"
assert_cmd_contains "eco set" "eco set testuser 2000" "2000"
assert_known "pay" "pay testuser 100"
assert_known "baltop" "baltop"

# 3. LuckPerms (/lp)
echo ""
echo "[3] LuckPerms (/lp)"
assert_cmd_contains "lp sync" "lp sync" "sync"
assert_cmd_contains "lp info" "lp info" "LuckPerms"
assert_cmd_contains "lp user info" "lp user testuser info" "testuser"
assert_cmd_contains "lp user permission set" "lp user testuser permission set essentials.home true" "Set"
assert_cmd_contains "lp user permission unset" "lp user testuser permission unset essentials.home" "Unset"
assert_cmd_contains "lp user parent add" "lp user testuser parent add admin" "Added"
assert_cmd_contains "lp user parent remove" "lp user testuser parent remove admin" "Removed"
assert_cmd_contains "lp group list" "lp group list" "Groups"
assert_cmd_contains "lp group info" "lp group default info" "default"
assert_cmd_contains "lp group permission set" "lp group default permission set essentials.home true" "Set"
assert_cmd_contains "lp group permission unset" "lp group default permission unset essentials.home" "Unset"
assert_cmd_contains "lp group parent add" "lp group default parent add admin" "Added"
assert_cmd_contains "lp group parent remove" "lp group default parent remove admin" "Removed"
assert_cmd_contains "lp group setweight" "lp group default setweight 10" "weight"
assert_cmd_contains "lp track list" "lp track list" "Tracks"
assert_cmd_contains "lp track createtrack" "lp track createtrack verifytrack" "Created"
assert_cmd_contains "lp track deletetrack" "lp track deletetrack verifytrack" "Deleted"

# Summary
TOTAL=$((PASS + FAIL))
echo ""
echo "==================================================================="
echo " Results: ${PASS}/${TOTAL} passed, ${FAIL} failed"
echo "==================================================================="
if [ "$FAIL" -ne 0 ]; then
    echo "Failed tests:"
    for t in "${FAILED_TESTS[@]}"; do
        echo "  - $t"
    done
    exit 1
fi
echo "All verification checks passed"
exit 0