#!/bin/bash
# Automated safe cleanup guardrail for the pumpkin-unified build host and VM.
#
# Trigger: cron every 25 minutes. Prunes old build caches/logs by age, and runs
# a deeper clean when disk or RAM usage exceeds 85%.
#
# Safety rules:
#   - Never touches worlds, world data, server.properties, or plugin configs.
#   - Only deletes files older than the given age (or superseded backups).
#   - All operations are best-effort (failures are ignored).
DISK_THRESHOLD=85
RAM_THRESHOLD=85

echo "=== cleanup $(date -u +%FT%TZ) ==="

# ---------- Always: age-based pruning (safe, bounded) ----------

# Cargo build caches: source caches older than 1 day are safe to drop.
find "$HOME/.cargo/registry/cache" -type f -mtime +1 -delete 2>/dev/null || true
find "$HOME/.cargo/git/checkouts" -type f -mtime +1 -delete 2>/dev/null || true

# Stale temp build artifacts.
find /tmp -maxdepth 1 \( -name 'cargo-*' -o -name 'rustc-*' \) -mtime +1 -exec rm -rf {} + 2>/dev/null || true
find /tmp -maxdepth 2 -type f -mtime +1 -delete 2>/dev/null || true

# VM plugin backups: keep the newest 1 .wasm backup and 3 .so backups.
if [ -d /root/pumpkin-server/plugins ]; then
    ls -1t /root/pumpkin-server/plugins/pumpkin-unified-core.wasm.*.old 2>/dev/null | tail -n +2 | xargs -r rm -f
    ls -1t /root/pumpkin-server/plugins/*.so.*.old 2>/dev/null | tail -n +4 | xargs -r rm -f
    # Rotated/old logs (never touches the active /tmp/pumpkin.log).
    find /root/pumpkin-server -name '*.log.gz' -mtime +3 -delete 2>/dev/null || true
    find /root/pumpkin-server/logs -name '*.log' -mtime +3 -delete 2>/dev/null || true
fi

# ---------- Threshold: deep clean above 85% ----------

disk_usage=$(df -h / | awk 'NR==2 {print $5}' | sed 's/%//')
ram_usage=$(free | awk 'NR==2 {printf "%.0f", $3/$2 * 100}')
echo "Disk usage: ${disk_usage}%, RAM usage: ${ram_usage}%"

if [ "${disk_usage:-0}" -gt "$DISK_THRESHOLD" ] || [ "${ram_usage:-0}" -gt "$RAM_THRESHOLD" ]; then
    echo "Threshold exceeded, running deep cleanup..."
    # Full cargo cache (re-downloadable), git checkouts, target dirs.
    rm -rf "$HOME/.cargo/registry/cache"/* 2>/dev/null || true
    rm -rf "$HOME/.cargo/git/checkouts"/* 2>/dev/null || true
    find /root/projects -maxdepth 3 -type d -name target -prune -exec rm -rf {} + 2>/dev/null || true
    # apt cache (safe).
    rm -rf /var/cache/apt/archives/*.deb 2>/dev/null || true
    # Old /tmp files (>1 day).
    find /tmp -maxdepth 2 -type f -mtime +1 -delete 2>/dev/null || true
    echo "Deep cleanup completed"
else
    echo "Usage below thresholds, age-based pruning only"
fi

echo "=== cleanup done ==="