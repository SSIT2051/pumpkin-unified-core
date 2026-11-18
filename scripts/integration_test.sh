#!/bin/bash
# Integration test suite for pumpkin-unified-core
# Runs against a live PumpkinMC server via RCON
#
# Usage: ./integration_test.sh [RCON_HOST] [RCON_PORT] [RCON_PASSWORD]
# Example: ./integration_test.sh localhost 25575 changeme_secure_password

set -euo pipefail

RCON_HOST="${1:-127.0.0.1}"
RCON_PORT="${2:-25575}"
RCON_PASSWORD="${3:-changeme_secure_password}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PASS=0
FAIL=0
FAILED_TESTS=()

rcon() {
    rcon-cli --host "$RCON_HOST" --port "$RCON_PORT" --password "$RCON_PASSWORD" "$1" 2>/dev/null
}

pass() { PASS=$((PASS + 1)); echo -e "  ${GREEN}PASS${NC}  $1"; }
fail() { FAIL=$((FAIL + 1)); FAILED_TESTS+=("$1"); echo -e "  ${RED}FAIL${NC}  $1 :: $2"; }

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
echo " Pumpkin Unified Core - Integration Test Suite"
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
assert_cmd_contains "eco give" "eco give TestPlayer 1000" "1000"
assert_cmd_contains "balance" "balance TestPlayer" "1000"
assert_cmd_contains "eco take" "eco take TestPlayer 200" "800"
assert_cmd_contains "eco set" "eco set TestPlayer 2000" "2000"
assert_known "pay" "pay TestPlayer 100"
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
assert_known "tpa" "tpa TestPlayer"
assert_known "tpahere" "tpahere TestPlayer"
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
assert_cmd_contains "ban" "ban TestPlayer automated-verify" "ban"
assert_cmd_contains "tempban" "tempban TestPlayer 1m automated-verify" "tempban"
assert_cmd_contains "unban" "unban TestPlayer" "unban"
assert_cmd_contains "mute" "mute TestPlayer automated-verify" "mute"
assert_cmd_contains "unmute" "unmute TestPlayer" "unmute"
assert_known "kick" "kick TestPlayer automated-verify"
assert_known "fly" "fly TestPlayer"
assert_known "god" "god TestPlayer"
assert_known "heal" "heal TestPlayer"
assert_known "feed" "feed TestPlayer"
assert_known "vanish" "vanish TestPlayer"

# 8. Gamemode
echo ""
echo "[8] Gamemode"
assert_known "gamemode" "gamemode creative TestPlayer"
assert_known "gmc" "gmc TestPlayer"
assert_known "gms" "gms TestPlayer"
assert_known "gma" "gma TestPlayer"
assert_known "gmsp" "gmsp TestPlayer"

# 9. Spawn
echo ""
echo "[9] Spawn"
assert_known "spawn" "spawn TestPlayer"
assert_known "setspawn" "setspawn"

# 10. Utility
echo ""
echo "[10] Utility"
assert_known "afk" "afk TestPlayer"
assert_cmd_contains "rules" "rules" "rule"
assert_cmd_contains "motd" "motd" "motd"
assert_cmd_contains "list" "list" "player"
assert_cmd_contains "whois" "whois TestPlayer" "TestPlayer"
assert_known "speed" "speed TestPlayer 5"
assert_known "compass" "compass TestPlayer"
assert_known "top" "top TestPlayer"
assert_cmd_contains "seen" "seen TestPlayer" "TestPlayer"
assert_known "weather" "weather clear"
assert_known "time" "time set day"
assert_known "butcher" "butcher"
assert_known "extinguish" "extinguish"

# 11. Chat
echo ""
echo "[11] Chat"
assert_known "msg" "msg TestPlayer hello"
assert_known "r" "r TestPlayer reply"
assert_known "mail" "mail read"
assert_cmd_contains "broadcast" "broadcast automated verify" "broadcast"
assert_known "broadcastworld" "broadcastworld automated verify"

# 12. LuckPerms (/lp)
echo ""
echo "[12] LuckPerms (/lp)"
assert_cmd_contains "lp sync" "lp sync" "sync"
assert_cmd_contains "lp info" "lp info" "LuckPerms"
assert_cmd_contains "lp user info" "lp user TestPlayer info" "TestPlayer"
assert_cmd_contains "lp user permission set" "lp user TestPlayer permission set essentials.home true" "Set"
assert_cmd_contains "lp user permission unset" "lp user TestPlayer permission unset essentials.home" "Unset"
assert_cmd_contains "lp user parent add" "lp user TestPlayer parent add admin" "Added"
assert_cmd_contains "lp user parent remove" "lp user TestPlayer parent remove admin" "Removed"
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

# 13. New commands (teleportation, etc.)
echo ""
echo "[13] Extended commands"
assert_known "back" "back"
assert_known "tp" "tp TestPlayer"
assert_known "tphere" "tphere TestPlayer"
assert_known "tppos" "tppos 100 64 100"
assert_known "worth" "worth"
assert_known "sell" "sell 10"
assert_known "clearinventory" "clearinventory"
assert_known "depth" "depth"
assert_known "enchant" "enchant"
assert_known "exp" "exp 100"
assert_known "firework" "firework"
assert_known "fix" "fix"
assert_known "getpos" "getpos"
assert_known "hat" "hat"
assert_known "help" "help"
assert_known "helpop" "helpop"
assert_known "ignore" "ignore TestPlayer"
assert_known "invsee" "invsee TestPlayer"
assert_known "jump" "jump"
assert_known "near" "near"
assert_known "nick" "nick TestNick"
assert_known "ping" "ping"
assert_known "powertool" "powertool"
assert_known "ptime" "ptime day"
assert_known "pweather" "pweather clear"
assert_known "realname" "realname TestPlayer"
assert_known "recipe" "recipe"
assert_known "repair" "repair"
assert_known "sign" "sign"
assert_known "sit" "sit"
assert_known "skull" "skull"
assert_known "sudo" "sudo TestPlayer say hello"
assert_known "sun" "sun"
assert_known "superpickaxe" "superpickaxe"
assert_known "thunder" "thunder"
assert_known "toggledownfall" "toggledownfall"
assert_known "tree" "tree"
assert_known "unlimit" "unlimit"
assert_known "version" "version"
assert_known "wb" "wb set 1000"
assert_known "xp" "xp 100"
assert_known "me" "me waves hello"
assert_known "defaultgamemode" "defaultgamemode survival"
assert_known "spawnmob" "spawnmob zombie"

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
echo -e "${GREEN}All integration tests passed${NC}"
exit 0