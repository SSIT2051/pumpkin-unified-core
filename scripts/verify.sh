#!/bin/bash
# Verification suite for pumpkin-unified-core (WASM plugin).
# Runs via RCON against the pumpkin-vm (or any PumpkinMC server).
#
# Covers all ported modules: economy, warps, homes, tpa, kits, moderation,
# gamemode, spawn, utility, chat, and LuckPerms (/lp).
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

# 2. Economy
echo ""
echo "[2] Economy"
assert_cmd_contains "eco give" "eco give testuser 1000" "1000"
assert_cmd_contains "balance" "balance testuser" "1000"
assert_cmd_contains "eco take" "eco take testuser 200" "800"
assert_cmd_contains "eco set" "eco set testuser 2000" "2000"
assert_known "pay" "pay testuser 100"
assert_known "baltop" "baltop"

# 3. Warps
echo ""
echo "[3] Warps"
assert_known "warp" "warp testwarp"
assert_known "setwarp" "setwarp testwarp"
assert_known "delwarp" "delwarp testwarp"
assert_known "warps" "warps"

# 4. Homes
echo ""
echo "[4] Homes"
assert_known "sethome" "sethome testhome"
assert_known "home" "home testhome"
assert_known "delhome" "delhome testhome"
assert_known "homes" "homes"

# 5. TPA
echo ""
echo "[5] TPA"
assert_known "tpa" "tpa testuser"
assert_known "tpahere" "tpahere testuser"
assert_known "tpaccept" "tpaccept"
assert_known "tpdeny" "tpdeny"
assert_known "tptoggle" "tptoggle"

# 6. Kits
echo ""
echo "[6] Kits"
assert_known "kit" "kit starter"
assert_known "kits" "kits"
assert_known "createkit" "createkit testkit"
assert_known "showkit" "showkit starter"

# 7. Moderation
echo ""
echo "[7] Moderation"
assert_cmd_contains "ban" "ban testuser automated-verify" "ban"
assert_cmd_contains "tempban" "tempban testuser 1m automated-verify" "tempban"
assert_cmd_contains "unban" "unban testuser" "unban"
assert_cmd_contains "mute" "mute testuser automated-verify" "mute"
assert_cmd_contains "unmute" "unmute testuser" "unmute"
assert_known "kick" "kick testuser automated-verify"
assert_known "fly" "fly testuser"
assert_known "god" "god testuser"
assert_known "heal" "heal testuser"
assert_known "feed" "feed testuser"
assert_known "vanish" "vanish testuser"

# 8. Gamemode
echo ""
echo "[8] Gamemode"
assert_known "gamemode" "gamemode creative testuser"
assert_known "gmc" "gmc testuser"
assert_known "gms" "gms testuser"
assert_known "gma" "gma testuser"
assert_known "gmsp" "gmsp testuser"

# 9. Spawn
echo ""
echo "[9] Spawn"
assert_known "spawn" "spawn testuser"
assert_known "setspawn" "setspawn"

# 10. Utility
echo ""
echo "[10] Utility"
assert_known "afk" "afk testuser"
assert_cmd_contains "rules" "rules" "rule"
assert_cmd_contains "motd" "motd" "motd"
assert_cmd_contains "list" "list" "player"
assert_cmd_contains "whois" "whois testuser" "testuser"
assert_known "speed" "speed testuser 5"
assert_known "compass" "compass testuser"
assert_known "top" "top testuser"
assert_cmd_contains "seen" "seen testuser" "testuser"
assert_known "weather" "weather clear"
assert_known "time" "time set day"
assert_known "butcher" "butcher"
assert_known "extinguish" "extinguish"

# 11. Chat
echo ""
echo "[11] Chat"
assert_known "msg" "msg testuser hello"
assert_known "r" "r testuser reply"
assert_known "mail" "mail read"
assert_cmd_contains "broadcast" "broadcast automated verify" "broadcast"
assert_known "broadcastworld" "broadcastworld automated verify"

# 12. LuckPerms (/lp)
echo ""
echo "[12] LuckPerms (/lp)"
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
assert_cmd_contains "lp group setdisplayname" "lp group default setdisplayname Default" "display"
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