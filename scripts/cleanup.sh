#!/bin/bash
# Trigger: cron every 20 min + disk/RAM > 85%
# Safe targets only

DISK_THRESHOLD=85
RAM_THRESHOLD=85

check_and_cleanup() {
    local disk_usage=$(df -h / | awk 'NR==2 {print $5}' | sed 's/%//')
    local ram_usage=$(free | awk 'NR==2 {printf "%.0f", $3/$2 * 100}')
    
    echo "Disk usage: ${disk_usage}%, RAM usage: ${ram_usage}%"
    
    if [ "$disk_usage" -le "$DISK_THRESHOLD" ] && [ "$ram_usage" -le "$RAM_THRESHOLD" ]; then
        echo "Usage below thresholds, skipping cleanup"
        return 0
    fi
    
    echo "Threshold exceeded, running cleanup..."
    
    cleanup_targets=(
        "$HOME/.cargo/registry/cache"
        "$HOME/.cargo/git/checkouts"
        "/root/projects/pumpkin-unified/target"
        "/tmp/cargo-*"
        "/tmp/rustc-*"
    )
    
    for target in "${cleanup_targets[@]}"; do
        find "$target" -type f -mtime +7 -delete 2>/dev/null || true
    done
    
    # Keep last 3 .so backups
    ls -t /root/pumpkin-server/plugins/pumpkin-unified-core.so.* 2>/dev/null | tail -n +4 | xargs rm -f 2>/dev/null || true
    
    echo "Cleanup completed"
}

check_and_cleanup