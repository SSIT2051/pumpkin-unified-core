# pumpkin-unified-core Implementation Plan

## Project Overview

Create a single native Rust `cdylib` plugin for PumpkinMC that unifies **EssentialsX** (160+ commands: economy, warps, homes, TPA, kits, moderation) and **LuckPerms** (permissions, groups, inheritance, contexts, tracks) into one zero-overhead plugin. Built entirely via GitHub Actions, deployed to `pumpkin-vm`, with automated upstream sync, resource guardrails, and in-game verification.

---

## Architecture Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Plugin Target** | `x86_64-unknown-linux-gnu` native `cdylib` (`.so`) | Pumpkin's `NativePluginLoader` loads `.so` files directly for zero-overhead execution |
| **API Crate** | `pumpkin-plugin-api` (crates.io, version `0.1.0-dev`) | Official Pumpkin plugin API with `Plugin` trait, `Command` builder, `EventHandler` |
| **Data Storage** | Multi-backend abstraction: SQLite (default), Redis, MySQL/PostgreSQL | Matches LuckPerms flexibility; `sqlx` with feature flags |
| **Config Format** | Dual TOML/YAML via `config-rs` + `serde` | Compatibility with existing EssentialsX/LuckPerms configs |
| **Concurrency** | `tokio` + `dashmap` for lock-free in-memory caches | Pumpkin is async-first; native Rust performance |
| **Entry Point** | `register_plugin!(UnifiedPlugin)` macro | Generates `pumpkin_plugin_init` extern "C" for `NativePluginLoader` |

---

## Repository Structure

```
pumpkin-unified-core/
├── .github/
│   ├── workflows/
│   │   ├── build.yml              # Cargo build + test + release .so artifact
│   │   ├── auto-sync.yml          # Monitor EssentialsX/LuckPerms main branches
│   │   ├── deploy.yml             # Download artifact → pumpkin-vm → restart → verify
│   │   └── cleanup.yml            # Scheduled + threshold-based cache cleanup
│   └── dependabot.yml
├── .cargo/
│   └── config.toml                # Target: x86_64-unknown-linux-gnu
├── src/
│   ├── lib.rs                     # Plugin entry point + register_plugin!
│   ├── config.rs                  # Dual TOML/YAML config loading
│   ├── storage/
│   │   ├── mod.rs                 # Storage trait + backend selection
│   │   ├── sqlite.rs              # SQLite backend (sqlx)
│   │   ├── redis.rs               # Redis backend (redis-rs)
│   │   └── mysql.rs               # MySQL/PostgreSQL backend (sqlx)
│   ├── economy/
│   │   ├── mod.rs                 # Balance, pay, baltop, eco give/take/set
│   │   └── commands.rs            # /balance, /pay, /baltop, /eco
│   ├── warps/
│   │   ├── mod.rs                 # Warp CRUD, per-warp permissions
│   │   └── commands.rs            # /warp, /setwarp, /delwarp, /warps
│   ├── homes/
│   │   ├── mod.rs                 # Multiple homes per player, per-world
│   │   └── commands.rs            # /sethome, /home, /delhome, /homes
│   ├── tpa/
│   │   ├── mod.rs                 # Request/accept/deny/expire logic
│   │   └── commands.rs            # /tpa, /tpahere, /tpaccept, /tpdeny
│   ├── kits/
│   │   ├── mod.rs                 # Kit definitions, cooldowns, permissions
│   │   └── commands.rs            # /kit, /kits, /createkit, /showkit
│   ├── moderation/
│   │   ├── mod.rs                 # Kick, ban, tempban, mute, jail, fly, god, heal, feed
│   │   └── commands.rs            # /kick, /ban, /tempban, /mute, /jail, /fly, /god, /heal, /feed
│   ├── gamemode/
│   │   └── commands.rs            # /gamemode, /gmc, /gms, /gma, /gmsp
│   ├── permissions/
│   │   ├── mod.rs                 # LuckPerms core: users, groups, inheritance, tracks
│   │   ├── context.rs             # Context calculator (world, gamemode, server, custom)
│   │   ├── storage.rs             # Permission data persistence
│   │   ├── commands.rs            # /lp user/group/permission/parent/meta/track/log
│   │   └── api.rs                 # Permission checking integration with Pumpkin
│   ├── spawn/
│   │   └── commands.rs            # /spawn, /setspawn
│   ├── utility/
│   │   ├── mod.rs                 # AFK, rules, motd, list, whois, speed, compass, etc.
│   │   └── commands.rs            # 40+ utility commands
│   ├── chat/
│   │   └── commands.rs            # /msg, /r, /mail, /broadcast, chat formatting
│   ├── events/
│   │   ├── mod.rs                 # Event handlers for all modules
│   │   ├── player.rs              # Join, leave, death, respawn, chat
│   │   ├── block.rs               # Break, place, interact
│   │   └── command.rs             # Pre-command permission checks
│   └── utils/
│       ├── text.rs                # TextComponent helpers, color codes
│       ├── time.rs                # Duration parsing (1h30m, 2d, etc.)
│       └── macros.rs              # Command registration macros
├── Cargo.toml
├── README.md
├── LICENSE
├── config.default.toml            # Default TOML config
├── config.default.yml             # Default YAML config
├── scripts/
│   ├── deploy.sh                  # SSH deploy to pumpkin-vm
│   ├── verify.sh                  # In-game verification via RCON/console
│   └── cleanup.sh                 # Local + VM safe cleanup (caches, logs, old .so)
├── pumpkin-vm/
│   ├── cleanup.service            # systemd timer for VM cleanup
│   └── cleanup.timer
└── plans/
    └── 1794938957220-pumpkin-unified-core-plan.md
```

---

## Phase 1: Repository Setup & Core Infrastructure (Week 1)

### 1.1 Create GitHub Repository
```bash
gh repo create SSIT2051/pumpkin-unified-core --public --description "Unified EssentialsX + LuckPerms native Rust plugin for PumpkinMC" --clone
```

### 1.2 Initialize Rust Project
- `cargo new --lib pumpkin-unified-core`
- Configure `Cargo.toml`:
  ```toml
  [package]
  name = "pumpkin-unified-core"
  version = "0.1.0"
  edition = "2024"
  license = "MIT OR Apache-2.0"
  authors = ["SSIT2051"]
  description = "Unified EssentialsX + LuckPerms plugin for PumpkinMC"

  [lib]
  crate-type = ["cdylib"]

  [dependencies]
  pumpkin-plugin-api = "0.1.0-dev"
  tracing = "0.1"
  serde = { version = "1.0", features = ["derive"] }
  serde_json = "1.0"
  toml = "0.8"
  yaml-rust = "0.4"
  config = { version = "0.14", features = ["toml", "yaml"] }
  sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "mysql", "postgres", "chrono", "uuid"] }
  redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }
  tokio = { version = "1.0", features = ["full"] }
  dashmap = "5.0"
  parking_lot = "0.12"
  thiserror = "1.0"
  anyhow = "1.0"
  chrono = { version = "0.4", features = ["serde"] }
  uuid = { version = "1.0", features = ["serde", "v4"] }
  regex = "1.0"
  base64 = "0.22"

  [profile.release]
  lto = true
  codegen-units = 1
  panic = "abort"
  strip = true
  ```

### 1.3 Core Plugin Skeleton (`src/lib.rs`)
- Implement `Plugin` trait with metadata
- Register all module commands in `on_load`
- Register event handlers
- Register permissions (EssentialsX + LuckPerms permission nodes)
- `register_plugin!(UnifiedPlugin)`

### 1.4 Configuration System (`src/config.rs`)
- Dual TOML/YAML loading via `config-rs`
- Merge with defaults, validate
- Hot-reload support via `/puc reload` command

### 1.5 Storage Abstraction (`src/storage/`)
- Define `Storage` trait with async CRUD
- Implement SQLite, Redis, MySQL backends
- Connection pooling, migrations
- Backend selection via config

---

## Phase 2: EssentialsX Core Modules (Weeks 2-4)

### 2.1 Economy Module
- Tables: `economy_balance`, `economy_transactions`
- Commands: `/balance`, `/pay`, `/baltop`, `/eco give/take/set`
- Permissions: `essentials.balance`, `essentials.pay`, `essentials.eco`

### 2.2 Warps Module
- Tables: `warps` (name, world, x, y, z, yaw, pitch, creator, perms)
- Commands: `/warp`, `/setwarp`, `/delwarp`, `/warps`, `/warpinfo`
- Per-warp permissions via `essentials.warps.<name>`

### 2.3 Homes Module
- Tables: `homes` (player_uuid, name, world, x, y, z, yaw, pitch)
- Commands: `/sethome`, `/home`, `/delhome`, `/homes`
- Multi-home support with limits per permission group

### 2.4 TPA Module
- In-memory request cache with TTL (DashMap + background cleanup)
- Commands: `/tpa`, `/tpahere`, `/tpaccept`, `/tpdeny`, `/tptoggle`
- Cross-world teleportation safety checks

### 2.5 Kits Module
- Tables: `kits` (name, delay, items_json, permissions)
- Commands: `/kit`, `/kits`, `/createkit`, `/showkit`
- Item serialization (enchantments, lore, custom NBT)

### 2.6 Moderation Module
- Tables: `bans`, `mutes`, `jails` (with expiry timestamps)
- Commands: `/kick`, `/ban`, `/tempban`, `/unban`, `/mute`, `/unmute`, `/jail`, `/unjail`, `/fly`, `/god`, `/heal`, `/feed`, `/vanish`
- Jail system: `/setjail`, `/deljail`, `/jails`

### 2.7 Gamemode Module
- Commands: `/gamemode`, `/gmc`, `/gms`, `/gma`, `/gmsp`
- Permissions: `essentials.gamemode`, `essentials.gamemode.<mode>`

### 2.8 Utility Commands (40+ commands)
- AFK, spawn, rules, motd, list, whois, speed, compass, top, back, seen, etc.
- Sign editing, repair, disposal, workbench, anvil, enderchest
- Weather, time, gamemode, lightning, butcher, extinguish

### 2.9 Chat Module
- `/msg`, `/r`, `/mail`, `/broadcast`, `/broadcastworld`
- Chat formatting with placeholders (group, prefix, suffix)

---

## Phase 3: LuckPerms Permission System (Weeks 5-7)

### 3.1 Core Data Model
- Tables: `users`, `groups`, `group_parents`, `group_members`, `permissions`, `inheritance`, `tracks`, `logs`
- UUID-based identity (players + groups)
- Context-aware permission nodes

### 3.2 Permission Resolution Engine
- Inheritance traversal (depth-first-pre-order configurable)
- Context calculation (server, world, gamemode, dimension, custom)
- Temporary permissions with expiry
- Priority: server-specific > world-specific > global; temp > permanent

### 3.3 Context Calculator
- Built-in contexts: `server`, `world`, `gamemode`, `dimension-type`
- Static contexts from config
- Dynamic contexts via event-driven calculators
- API for other plugins to register contexts

### 3.4 LuckPerms Commands (`/lp`)
- User: info, editor, promote, demote, showtracks, clear, clone, permission, parent, meta
- Group: info, editor, listmembers, setweight, setdisplayname, showtracks, clear, rename, clone, permission, parent, meta
- Track: createtrack, deletetrack, listtracks
- Log: recent, search, notify
- Sync, info, editor, applyedits, verbose, tree, search, import, export, reloadconfig, translations

### 3.5 Integration with Pumpkin Permissions
- Implement `PermissionService` trait for Pumpkin's permission checks
- Register all EssentialsX permission nodes dynamically
- Vault-compatible API for economy/chat plugins

---

## Phase 4: Events & Cross-Module Integration (Week 8)

### 4.1 Player Events
- Join: load permissions, economy, homes; send welcome; apply AFK timeout
- Leave: save dirty caches; cleanup TPA requests
- Death: keep inventory (config), drop items
- Respawn: handle bed/spawn/anchor logic
- Chat: format with LuckPerms prefixes/suffixes; spy

### 4.2 Block Events
- Break/Place: AntiBuild integration (permissions per block/item)
- Interact: Sign shops, protection

### 4.3 Command Pre-Processing
- Permission check via LuckPerms engine
- Cooldown enforcement (kits, TPA, etc.)
- Cost deduction (economy command costs)

---

## Phase 5: GitHub Actions Workflows (Week 9)

### 5.1 Build Workflow (`.github/workflows/build.yml`)
```yaml
name: Build & Test
on: [push, pull_request]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: target: x86_64-unknown-linux-gnu
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --release --target x86_64-unknown-linux-gnu
      - run: cargo test --target x86_64-unknown-linux-gnu
      - uses: actions/upload-artifact@v4
        with:
          name: pumpkin-unified-core.so
          path: target/x86_64-unknown-linux-gnu/release/libpumpkin_unified_core.so
```

### 5.2 Auto-Sync Workflow (`.github/workflows/auto-sync.yml`)
- **Schedule**: Every 6 hours + on new releases
- **Sources**: `EssentialsX/Essentials@main`, `LuckPerms/LuckPerms@main`
- **Process**:
  1. Fetch latest commits since last sync
  2. Parse relevant changes (commands, permissions, config options)
  3. Generate PR with ported changes
  4. Run build + test workflow on PR
  5. **AI-assisted verification**: Deploy to test VM, run command smoke tests via RCON
  6. Add label `🤖-verified` if tests pass
  7. Require manual review + `✅-approved` label for merge

### 5.3 Deploy Workflow (`.github/workflows/deploy.yml`)
```yaml
name: Deploy to pumpkin-vm
on:
  workflow_dispatch:
  release:
    types: [published]
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
        with: name: pumpkin-unified-core.so
      - name: Deploy via SSH
        run: |
          scp libpumpkin_unified_core.so pumpkin-vm:/root/pumpkin-server/plugins/pumpkin-unified-core.so.new
          ssh pumpkin-vm "sudo systemctl restart pumpkin && sleep 5 && ./verify-plugin.sh"
```

### 5.4 Cleanup Workflow (`.github/workflows/cleanup.yml`)
- **Schedule**: Every 20 minutes (`*/20 * * * *`)
- **Trigger**: Disk > 85% OR RAM > 85% (via `df`/`free` checks)
- **Scope**: `target/`, `~/.cargo/registry/cache/`, `~/.cargo/git/checkouts/`, old `.so` backups (`*.so.old*`), build logs
- **Never touch**: `world/`, `config/`, `plugins/*.so` (active), `logs/latest.log`

---

## Phase 6: VM & Local Resource Guardrails (Week 10)

### 6.1 Local Cleanup Script (`scripts/cleanup.sh`)
```bash
#!/bin/bash
# Trigger: cron every 20 min + disk/RAM > 85%
# Safe targets only
cleanup_targets=(
  "$HOME/.cargo/registry/cache"
  "$HOME/.cargo/git/checkouts"
  "/root/projects/pumpkin-unified/target"
  "/tmp/cargo-*"
  "/tmp/rustc-*"
)
for target in "${cleanup_targets[@]}"; do
  find "$target" -type f -mtime +7 -delete 2>/dev/null
done
# Keep last 3 .so backups
ls -t /root/pumpkin-server/plugins/pumpkin-unified-core.so.* 2>/dev/null | tail -n +4 | xargs rm -f
```

### 6.2 VM Cleanup (systemd)
- `/etc/systemd/system/pumpkin-cleanup.service` + `.timer`
- Same logic, targets VM paths
- Runs every 20 min + on threshold breach

### 6.3 Resource Monitoring
- `df -h /` and `free -m` in cleanup script
- Alert via GitHub Actions workflow dispatch if thresholds exceeded

---

## Phase 7: In-Game Verification (Week 10)

### 7.1 Verification Script (`scripts/verify.sh`)
```bash
#!/bin/bash
# Run via RCON or console log parsing
ssh pumpkin-vm << 'EOF'
  # Wait for server ready
  sleep 10
  # Check plugin loaded
  rcon "plugins" | grep -q "pumpkin-unified-core" || exit 1
  # Test core commands
  rcon "lp user test permission set essentials.home true"
  rcon "lp group default parent add admin"
  rcon "eco give test 1000"
  rcon "balance test" | grep -q "1000" || exit 1
  rcon "sethome testhome"
  rcon "home testhome" | grep -q "Teleported" || exit 1
  rcon "setwarp testwarp"
  rcon "warp testwarp" | grep -q "Teleported" || exit 1
  rcon "kit starter" | grep -q "Received" || exit 1
  rcon "tpa test" # requires 2nd player
  echo "✅ All verification checks passed"
EOF
```

### 7.2 Automated Test Suite
- Integration tests in `tests/integration.rs` using `pumpkin-test` crate (if available)
- Spawn test server, load plugin, execute command assertions

---

## Phase 8: Documentation & Licensing (Week 11)

### 8.1 README.md
- Feature matrix (EssentialsX commands ✅/❌, LuckPerms features ✅/❌)
- Installation: download `.so` → `plugins/` → restart
- Configuration: TOML/YAML examples for all modules
- Permissions reference table
- **Credits section**:
  > This plugin ports features from **EssentialsX** (https://github.com/EssentialsX/Essentials, MIT License) and **LuckPerms** (https://github.com/LuckPerms/LuckPerms, MIT License). Original authors retain copyright. This is a clean-room Rust reimplementation for PumpkinMC, not a fork or wrapper.

### 8.2 LICENSE
- Dual MIT OR Apache-2.0 (matches `pumpkin-plugin-api`)
- Include `NOTICE` file with upstream attributions

### 8.3 Config Documentation
- `config.default.toml` and `config.default.yml` with all options commented

---

## Risk Assessment & Mitigation

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Pumpkin API breaking changes | High | High | Pin `pumpkin-plugin-api` version; test on every PR |
| Scope creep (160+ commands) | High | High | Phase 2 delivers MVP; iterate in follow-up PRs |
| Data migration from existing servers | Medium | Medium | Import scripts for EssentialsX/LuckPerms YAML/SQL |
| Permission resolution performance | Medium | Medium | Cache resolved permissions; benchmarks in CI |
| Native plugin unloading unsupported | Low | High | Document: requires server restart for updates |
| VM SSH key rotation | Medium | Low | Use GitHub Environments with secrets; rotate quarterly |

---

## Success Criteria

1. **Repository**: `SSIT2051/pumpkin-unified-core` public, CI passing
2. **Build**: `cargo build --release --target x86_64-unknown-linux-gnu` produces `libpumpkin_unified_core.so` (< 10 MB)
3. **Load**: Plugin loads on PumpkinMC without errors; `/plugins` shows it
4. **Commands**: All 160+ EssentialsX commands + full `/lp` suite register and execute
5. **Permissions**: LuckPerms inheritance, contexts, tracks work identically
6. **Economy**: Balance/pay/baltop persist across restarts (SQLite/Redis/MySQL)
7. **Deploy**: GitHub Actions → pumpkin-vm → restart → verification passes
8. **Guardrails**: Cleanup runs on schedule + thresholds; never deletes world/config/active `.so`
9. **Sync**: Auto-sync detects upstream changes, creates PRs, AI-verified + manual review
10. **Credits**: README.md and LICENSE clearly attribute EssentialsX and LuckPerms authors

---

## Open Questions for Implementation

1. **Pumpkin version pinning**: Which PumpkinMC version/commit to target? (API unstable)
2. **Item serialization**: How to serialize Pumpkin items (enchantments, NBT) for kits?
3. **Sign shops**: EssentialsX sign shop format vs Pumpkin's block entity system
4. **WorldGuard integration**: EssentialsX Protect/AntiBuild vs separate region plugin
5. **Web editor**: LuckPerms web editor requires HTTP endpoint - implement in plugin?
6. **Metrics/Telemetry**: Opt-in usage stats?
7. **Migration tooling**: One-time import from EssentialsX `config.yml` + LuckPerms storage

---

## Next Steps

1. **Immediate**: Create GitHub repo, initialize Rust project, push Phase 1 skeleton
2. **Week 1**: Core infrastructure (config, storage, plugin skeleton)
3. **Week 2-4**: EssentialsX modules (parallelize across modules)
4. **Week 5-7**: LuckPerms permission engine (most complex)
5. **Week 8**: Events + integration
6. **Week 9**: GitHub Actions workflows
7. **Week 10**: VM guardrails + verification
8. **Week 11**: Documentation + licensing

---

*Plan generated: 2026-11-17 | Target: SSIT2051/pumpkin-unified-core | Agent: Kilo Planning Mode*