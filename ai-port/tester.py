#!/usr/bin/env python3
"""
AI Testing Agent: Automated test generation and execution for PumpkinMC plugins.
Generates RCON command assertions and runs them against a test server.

Usage:
    python tester.py --test-file tests.json --server-host localhost --server-port 25575
"""

import argparse
import json
import re
import subprocess
import sys
import time
from pathlib import Path
from typing import Dict, List, Optional, Tuple


class RCONClient:
    """Simple RCON client for Minecraft/PumpkinMC servers."""

    def __init__(self, host: str, port: int, password: str):
        self.host = host
        self.port = port
        self.password = password
        self.request_id = 0

    def send_command(self, command: str) -> Tuple[int, str]:
        """Send a command via RCON and return (id, response)."""
        # This is a simplified RCON implementation
        # In production, use a proper RCON library like 'mcrcon' or 'rcon'
        try:
            result = subprocess.run(
                ['rcon-cli', '--host', self.host, '--port', str(self.port),
                 '--password', self.password, command],
                capture_output=True,
                text=True,
                timeout=10
            )
            return (0, result.stdout.strip())
        except Exception as e:
            return (-1, str(e))

    def is_connected(self) -> bool:
        """Check if server is responding."""
        _, response = self.send_command('list')
        return response.startswith('There are') or 'players' in response.lower()


class TestGenerator:
    """Generate test cases from command definitions."""

    def __init__(self):
        self.test_cases: List[Dict] = []

    def generate_command_tests(self, commands: List[str]) -> List[Dict]:
        """Generate basic command tests."""
        tests = []
        for cmd in commands:
            tests.append({
                'name': f'test_{cmd.replace(" ", "_")}',
                'command': cmd,
                'expected': 'success',
                'timeout': 5
            })
        return tests

    def generate_economy_tests(self) -> List[Dict]:
        """Generate economy module tests."""
        return [
            {'name': 'economy_give', 'command': 'eco give testuser 1000', 'expected': 'Set balance', 'timeout': 5},
            {'name': 'economy_balance', 'command': 'balance testuser', 'expected': '1000', 'timeout': 5},
            {'name': 'economy_pay', 'command': 'pay testuser 500', 'expected': 'paid', 'timeout': 5},
            {'name': 'economy_take', 'command': 'eco take testuser 200', 'expected': 'Took', 'timeout': 5},
            {'name': 'economy_set', 'command': 'eco set testuser 2000', 'expected': 'Set', 'timeout': 5},
            {'name': 'economy_baltop', 'command': 'baltop', 'expected': 'Balance Top', 'timeout': 5},
        ]

    def generate_warp_tests(self) -> List[Dict]:
        """Generate warp module tests."""
        return [
            {'name': 'warp_set', 'command': 'setwarp testwarp', 'expected': 'created', 'timeout': 5},
            {'name': 'warp_list', 'command': 'warps', 'expected': 'Warps', 'timeout': 5},
            {'name': 'warp_teleport', 'command': 'warp testwarp', 'expected': 'Teleported', 'timeout': 5},
            {'name': 'warp_delete', 'command': 'delwarp testwarp', 'expected': 'deleted', 'timeout': 5},
        ]

    def generate_home_tests(self) -> List[Dict]:
        """Generate home module tests."""
        return [
            {'name': 'home_set', 'command': 'sethome testhome', 'expected': 'Home', 'timeout': 5},
            {'name': 'home_list', 'command': 'homes', 'expected': 'Your Homes', 'timeout': 5},
            {'name': 'home_teleport', 'command': 'home testhome', 'expected': 'Teleported', 'timeout': 5},
            {'name': 'home_delete', 'command': 'delhome testhome', 'expected': 'deleted', 'timeout': 5},
        ]

    def generate_tpa_tests(self) -> List[Dict]:
        """Generate TPA module tests."""
        return [
            {'name': 'tpa_toggle', 'command': 'tptoggle', 'expected': 'Teleport requests', 'timeout': 5},
        ]

    def generate_kit_tests(self) -> List[Dict]:
        """Generate kit module tests."""
        return [
            {'name': 'kit_list', 'command': 'kits', 'expected': 'Available Kits', 'timeout': 5},
            {'name': 'kit_use', 'command': 'kit starter', 'expected': 'Received kit', 'timeout': 5},
        ]

    def generate_moderation_tests(self) -> List[Dict]:
        """Generate moderation module tests."""
        return [
            {'name': 'kick_test', 'command': 'kick testuser reason', 'expected': 'Kicked', 'timeout': 5},
            {'name': 'ban_test', 'command': 'ban testuser reason', 'expected': 'Banned', 'timeout': 5},
            {'name': 'tempban_test', 'command': 'tempban testuser 1m reason', 'expected': 'Temporarily banned', 'timeout': 5},
            {'name': 'mute_test', 'command': 'mute testuser reason', 'expected': 'Muted', 'timeout': 5},
            {'name': 'unban_test', 'command': 'unban testuser', 'expected': 'Unbanned', 'timeout': 5},
            {'name': 'unmute_test', 'command': 'unmute testuser', 'expected': 'Unmuted', 'timeout': 5},
            {'name': 'fly_test', 'command': 'fly testuser', 'expected': 'Flight', 'timeout': 5},
            {'name': 'god_test', 'command': 'god testuser', 'expected': 'God mode', 'timeout': 5},
            {'name': 'heal_test', 'command': 'heal testuser', 'expected': 'Healed', 'timeout': 5},
            {'name': 'feed_test', 'command': 'feed testuser', 'expected': 'Fed', 'timeout': 5},
            {'name': 'vanish_test', 'command': 'vanish testuser', 'expected': 'Vanish', 'timeout': 5},
        ]

    def generate_gamemode_tests(self) -> List[Dict]:
        """Generate gamemode module tests."""
        return [
            {'name': 'gamemode_test', 'command': 'gamemode creative testuser', 'expected': 'gamemode', 'timeout': 5},
            {'name': 'gmc_test', 'command': 'gmc testuser', 'expected': 'gamemode', 'timeout': 5},
            {'name': 'gms_test', 'command': 'gms testuser', 'expected': 'gamemode', 'timeout': 5},
            {'name': 'gma_test', 'command': 'gma testuser', 'expected': 'gamemode', 'timeout': 5},
            {'name': 'gmsp_test', 'command': 'gmsp testuser', 'expected': 'gamemode', 'timeout': 5},
        ]

    def generate_utility_tests(self) -> List[Dict]:
        """Generate utility module tests."""
        return [
            {'name': 'afk_test', 'command': 'afk testuser', 'expected': 'AFK', 'timeout': 5},
            {'name': 'rules_test', 'command': 'rules', 'expected': 'Server Rules', 'timeout': 5},
            {'name': 'motd_test', 'command': 'motd', 'expected': 'MOTD', 'timeout': 5},
            {'name': 'list_test', 'command': 'list', 'expected': 'Online players', 'timeout': 5},
            {'name': 'whois_test', 'command': 'whois testuser', 'expected': 'Player Info', 'timeout': 5},
            {'name': 'speed_test', 'command': 'speed testuser 5', 'expected': 'speed', 'timeout': 5},
            {'name': 'compass_test', 'command': 'compass testuser', 'expected': 'compass', 'timeout': 5},
            {'name': 'top_test', 'command': 'top testuser', 'expected': 'top block', 'timeout': 5},
            {'name': 'back_test', 'command': 'back testuser', 'expected': 'previous location', 'timeout': 5},
            {'name': 'seen_test', 'command': 'seen testuser', 'expected': 'online', 'timeout': 5},
            {'name': 'repair_test', 'command': 'repair testuser', 'expected': 'repaired', 'timeout': 5},
        ]

    def generate_chat_tests(self) -> List[Dict]:
        """Generate chat module tests."""
        return [
            {'name': 'msg_test', 'command': 'msg testuser hello', 'expected': 'PM', 'timeout': 5},
            {'name': 'broadcast_test', 'command': 'broadcast test', 'expected': 'Broadcast', 'timeout': 5},
        ]

    def generate_permission_tests(self) -> List[Dict]:
        """Generate LuckPerms command tests."""
        return [
            {'name': 'lp_user_info', 'command': 'lp user testuser info', 'expected': 'User', 'timeout': 5},
            {'name': 'lp_user_perm_set', 'command': 'lp user testuser permission set essentials.home true', 'expected': 'Set permission', 'timeout': 5},
            {'name': 'lp_user_perm_unset', 'command': 'lp user testuser permission unset essentials.home', 'expected': 'Unset permission', 'timeout': 5},
            {'name': 'lp_user_parent_add', 'command': 'lp user testuser parent add admin', 'expected': 'Added group', 'timeout': 5},
            {'name': 'lp_user_parent_remove', 'command': 'lp user testuser parent remove admin', 'expected': 'Removed group', 'timeout': 5},
            {'name': 'lp_group_list', 'command': 'lp group list', 'expected': 'Groups', 'timeout': 5},
            {'name': 'lp_group_info', 'command': 'lp group default info', 'expected': 'Group', 'timeout': 5},
            {'name': 'lp_group_setweight', 'command': 'lp group default setweight 10', 'expected': 'Set weight', 'timeout': 5},
            {'name': 'lp_group_setdisplayname', 'command': 'lp group default setdisplayname Default', 'expected': 'Set display name', 'timeout': 5},
            {'name': 'lp_group_perm_set', 'command': 'lp group default permission set essentials.home true', 'expected': 'Set permission', 'timeout': 5},
            {'name': 'lp_group_perm_unset', 'command': 'lp group default permission unset essentials.home', 'expected': 'Unset permission', 'timeout': 5},
            {'name': 'lp_group_parent_add', 'command': 'lp group default parent add admin', 'expected': 'Added parent', 'timeout': 5},
            {'name': 'lp_group_parent_remove', 'command': 'lp group default parent remove admin', 'expected': 'Removed parent', 'timeout': 5},
            {'name': 'lp_track_create', 'command': 'lp track createtrack testtrack', 'expected': 'Created track', 'timeout': 5},
            {'name': 'lp_track_delete', 'command': 'lp track deletetrack testtrack', 'expected': 'Deleted track', 'timeout': 5},
            {'name': 'lp_track_list', 'command': 'lp track list', 'expected': 'Tracks', 'timeout': 5},
            {'name': 'lp_track_append', 'command': 'lp track testtrack append default', 'expected': 'Appended', 'timeout': 5},
            {'name': 'lp_track_insert', 'command': 'lp track testtrack insert admin 0', 'expected': 'Inserted', 'timeout': 5},
            {'name': 'lp_track_remove', 'command': 'lp track testtrack remove admin', 'expected': 'Removed', 'timeout': 5},
            {'name': 'lp_track_clear', 'command': 'lp track testtrack clear', 'expected': 'Cleared', 'timeout': 5},
            {'name': 'lp_track_rename', 'command': 'lp track testtrack rename newtrack', 'expected': 'Renamed', 'timeout': 5},
            {'name': 'lp_track_clone', 'command': 'lp track testtrack clone testtrack2', 'expected': 'Cloned', 'timeout': 5},
            {'name': 'lp_sync', 'command': 'lp sync', 'expected': 'Syncing', 'timeout': 5},
            {'name': 'lp_info', 'command': 'lp info', 'expected': 'LuckPerms Info', 'timeout': 5},
            {'name': 'lp_reload', 'command': 'lp reload', 'expected': 'Reloading', 'timeout': 5},
        ]

    def generate_all_tests(self) -> List[Dict]:
        """Generate all test cases."""
        tests = []
        tests.extend(self.generate_economy_tests())
        tests.extend(self.generate_warp_tests())
        tests.extend(self.generate_home_tests())
        tests.extend(self.generate_tpa_tests())
        tests.extend(self.generate_kit_tests())
        tests.extend(self.generate_moderation_tests())
        tests.extend(self.generate_gamemode_tests())
        tests.extend(self.generate_utility_tests())
        tests.extend(self.generate_chat_tests())
        tests.extend(self.generate_permission_tests())
        return tests


class TestRunner:
    """Run tests against a PumpkinMC server."""

    def __init__(self, rcon_client: RCONClient):
        self.rcon = rcon_client
        self.results: List[Dict] = []

    def run_test(self, test: Dict) -> Dict:
        """Run a single test case."""
        name = test.get('name', 'unknown')
        command = test.get('command', '')
        expected = test.get('expected', '')
        timeout = test.get('timeout', 5)

        result = {
            'name': name,
            'command': command,
            'expected': expected,
            'status': 'unknown',
            'response': '',
            'duration': 0
        }

        start_time = time.time()

        try:
            # Send command
            _, response = self.rcon.send_command(command)

            result['response'] = response
            result['duration'] = time.time() - start_time

            # Check expected result
            if expected.lower() in response.lower() or 'success' in expected.lower():
                result['status'] = 'passed'
            else:
                result['status'] = 'failed'
                result['error'] = f'Expected "{expected}" in response, got: {response[:200]}'

        except Exception as e:
            result['duration'] = time.time() - start_time
            result['status'] = 'error'
            result['error'] = str(e)

        return result

    def run_all_tests(self, tests: List[Dict]) -> List[Dict]:
        """Run all test cases."""
        results = []
        total = len(tests)

        for i, test in enumerate(tests):
            print(f"[{i+1}/{total}] Running: {test['name']}...", end=' ', flush=True)
            result = self.run_test(test)
            results.append(result)
            print(f"{result['status'].upper()}")

            # Small delay between tests
            time.sleep(0.5)

        return results


def generate_test_report(results: List[Dict]) -> str:
    """Generate a test report."""
    passed = sum(1 for r in results if r['status'] == 'passed')
    failed = sum(1 for r in results if r['status'] == 'failed')
    errors = sum(1 for r in results if r['status'] == 'error')
    total = len(results)

    report = f"""
# Pumpkin Unified Core Test Report

## Summary
- Total Tests: {total}
- Passed: {passed}
- Failed: {failed}
- Errors: {errors}
- Pass Rate: {passed/total*100:.1f}% (if total > 0 else 0)

## Test Results

| Test | Command | Status | Response |
|------|---------|--------|----------|
"""

    for r in results:
        response_preview = r.get('response', '')[:50].replace('\n', ' ')
        report += f"| {r['name']} | `{r['command']}` | {r['status'].upper()} | {response_preview} |\n"

    # Add failures section
    failures = [r for r in results if r['status'] in ('failed', 'error')]
    if failures:
        report += "\n## Failures\n\n"
        for r in failures:
            report += f"### {r['name']}\n"
            report += f"- Command: `{r['command']}`\n"
            report += f"- Expected: {r['expected']}\n"
            report += f"- Got: {r.get('response', 'No response')[:200]}\n"
            if 'error' in r:
                report += f"- Error: {r['error']}\n"
            report += "\n"

    return report


def main():
    parser = argparse.ArgumentParser(description='Test PumpkinMC plugin')
    parser.add_argument('--test-file', help='JSON file with test cases')
    parser.add_argument('--server-host', default='localhost', help='Server host')
    parser.add_argument('--server-port', type=int, default=25575, help='Server RCON port')
    parser.add_argument('--server-password', default='', help='Server RCON password')
    parser.add_argument('--output', help='Output report file')
    parser.add_argument('--generate-only', action='store_true', help='Only generate tests, don\'t run')
    args = parser.parse_args()

    generator = TestGenerator()

    # Generate tests
    if args.test_file:
        with open(args.test_file) as f:
            custom_tests = json.load(f)
        tests = custom_tests
    else:
        tests = generator.generate_all_tests()

    print(f"Generated {len(tests)} test cases")

    if args.generate_only:
        print(json.dumps(tests, indent=2))
        return 0

    # Run tests
    if not args.server_password:
        print("Error: --server-password is required for running tests", file=sys.stderr)
        return 1

    rcon = RCONClient(args.server_host, args.server_port, args.server_password)

    # Wait for server to be ready
    print("Waiting for server to be ready...")
    for _ in range(30):
        if rcon.is_connected():
            print("Server is ready!")
            break
        time.sleep(1)
    else:
        print("Error: Server not ready after 30 seconds", file=sys.stderr)
        return 1

    # Run tests
    runner = TestRunner(rcon)
    results = runner.run_all_tests(tests)

    # Generate report
    report = generate_test_report(results)
    print(report)

    if args.output:
        Path(args.output).write_text(report)
        print(f"Report saved to {args.output}")

    # Return non-zero if any tests failed
    failed = sum(1 for r in results if r['status'] in ('failed', 'error'))
    return 0 if failed == 0 else 1


if __name__ == '__main__':
    sys.exit(main())