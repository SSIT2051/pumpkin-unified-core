# pumpkin-unified-core

Unified **EssentialsX** + **LuckPerms** native Rust plugin for PumpkinMC.

A single high-performance `cdylib` plugin (`.so`) that combines 160+ EssentialsX commands with full LuckPerms permission system, built for zero-overhead execution via Pumpkin's `NativePluginLoader`.

## Features

### EssentialsX Commands (160+)
| Module | Commands | Status |
|--------|----------|--------|
| Economy | `/balance`, `/pay`, `/baltop`, `/eco give/take/set` | ✅ |
| Warps | `/warp`, `/setwarp`, `/delwarp`, `/warps` | ✅ |
| Homes | `/sethome`, `/home`, `/delhome`, `/homes` | ✅ |
| TPA | `/tpa`, `/tpahere`, `/tpaccept`, `/tpdeny`, `/tptoggle` | ✅ |
| Kits | `/kit`, `/kits`, `/createkit`, `/showkit` | ✅ |
| Moderation | `/kick`, `/ban`, `/tempban`, `/unban`, `/mute`, `/unmute`, `/jail`, `/unjail`, `/fly`, `/god`, `/heal`, `/feed`, `/vanish` | ✅ |
| Gamemode | `/gamemode`, `/gmc`, `/gms`, `/gma`, `/gmsp` | ✅ |
| Utility | `/afk`, `/rules`, `/motd`, `/list`, `/whois`, `/speed`, `/compass`, `/top`, `/back`, `/seen`, `/repair`, `/disposal`, `/workbench`, `/anvil`, `/enderchest`, `/weather`, `/time`, `/butcher`, `/extinguish` | ✅ |
| Chat | `/msg`, `/r`, `/mail`, `/broadcast`, `/broadcastworld` | ✅ |
| Spawn | `/spawn`, `/setspawn` | ✅ |

### LuckPerms Features
- Full permission system with users, groups, inheritance
- Context-aware permissions (server, world, gamemode, dimension, custom)
- Tracks for promotion/demotion
- `/lp` command suite (user, group, track, log, sync, editor, etc.)
- Permission priority: server-specific > world-specific > global; temp > permanent
- Vault-compatible API for economy/chat integration

### Technical
- **Native `.so` plugin** - Zero overhead via `NativePluginLoader`
- **Multi-backend storage** - SQLite (default), Redis, MySQL/PostgreSQL
- **Dual config format** - TOML and YAML support
- **Async-first** - Built on Tokio with lock-free DashMap caches
- **Hot-reload** - `/puc reload` for config changes

## Installation

1. Download the latest `libpumpkin_unified_core.so` from [Releases](https://github.com/SSIT2051/pumpkin-unified-core/releases)
2. Place it in your PumpkinMC `plugins/` directory
3. Restart the server
4. Configure via `config/pumpkin-unified-core/config.toml` or `config.yml`

## Configuration

The plugin supports both TOML and YAML configuration files. See `config.default.toml` and `config.default.yml` for all options.

Key configuration sections:
- `storage` - Database backend selection
- `economy` - Currency, starting balance, limits
- `warps` - Limits, cooldowns, permissions
- `homes` - Home limits, cooldowns
- `tpa` - Request timeout, cross-world settings
- `kits` - Cooldowns, signs
- `moderation` - Messages, jail location, duration limits
- `permissions` - LuckPerms contexts, default group, inheritance
- `chat` - Formatting, anti-spam
- `events` - Join/leave messages, death messages, anti-build

## Permissions

### EssentialsX Permissions
```
essentials.balance          # Check own balance
essentials.balance.others   # Check other's balance
essentials.pay              # Pay other players
essentials.eco              # Economy admin commands
essentials.warp             # Use warps
essentials.setwarp          # Create warps
essentials.delwarp          # Delete warps
essentials.warps            # List warps
essentials.home             # Teleport to home
essentials.sethome          # Set home
essentials.delhome          # Delete home
essentials.homes            # List homes
essentials.tpa              # Request teleport
essentials.tpahere          # Request teleport here
essentials.tpaccept         # Accept teleport
essentials.tpdeny           # Deny teleport
essentials.tptoggle         # Toggle TPA requests
essentials.kit              # Use kits
essentials.kits             # List kits
essentials.createkit        # Create kits
essentials.kick             # Kick players
essentials.ban              # Ban players
essentials.tempban          # Temp ban players
essentials.unban            # Unban players
essentials.mute             # Mute players
essentials.unmute           # Unmute players
essentials.jail             # Jail players
essentials.unjail           # Unjail players
essentials.fly              # Toggle flight
essentials.god              # Toggle god mode
essentials.heal             # Heal players
essentials.feed             # Feed players
essentials.vanish           # Toggle vanish
essentials.gamemode         # Change gamemode
essentials.spawn            # Teleport to spawn
essentials.setspawn         # Set spawn
essentials.afk              # Toggle AFK
essentials.rules            # View rules
essentials.motd             # View MOTD
essentials.list             # List players
essentials.whois            # View player info
essentials.speed            # Set speed
essentials.compass          # Get compass
essentials.top              # Teleport to top
essentials.back             # Teleport back
essentials.seen             # Check last seen
essentials.repair           # Repair items
essentials.disposal         # Open disposal
essentials.workbench        # Open workbench
essentials.anvil            # Open anvil
essentials.enderchest       # Open ender chest
essentials.weather          # Change weather
essentials.time             # Change time
essentials.butcher          # Kill mobs
essentials.extinguish       # Extinguish fires
essentials.msg              # Send private messages
essentials.reply            # Reply to messages
essentials.mail             # Manage mail
essentials.broadcast        # Broadcast messages
essentials.broadcastworld   # Broadcast to world
```

### LuckPerms Permissions
```
luckperms.user.info         # View user info
luckperms.user.editor       # Use web editor for user
luckperms.user.promote      # Promote user
luckperms.user.demote       # Demote user
luckperms.user.showtracks   # Show user tracks
luckperms.user.clear        # Clear user data
luckperms.user.clone        # Clone user
luckperms.user.permission   # Manage user permissions
luckperms.user.parent       # Manage user groups
luckperms.user.meta         # Manage user metadata
luckperms.group.info        # View group info
luckperms.group.editor      # Use web editor for group
luckperms.group.listmembers # List group members
luckperms.group.setweight   # Set group weight
luckperms.group.setdisplayname # Set group display name
luckperms.group.showtracks  # Show group tracks
luckperms.group.clear       # Clear group data
luckperms.group.rename      # Rename group
luckperms.group.clone       # Clone group
luckperms.group.permission  # Manage group permissions
luckperms.group.parent      # Manage group inheritance
luckperms.group.meta        # Manage group metadata
luckperms.track.createtrack # Create track
luckperms.track.deletetrack # Delete track
luckperms.track.listtracks  # List tracks
luckperms.log.recent        # View recent logs
luckperms.log.search        # Search logs
luckperms.log.notify        # Log notifications
luckperms.sync              # Sync storage
luckperms.info              # Plugin info
luckperms.editor            # Open web editor
luckperms.applyedits        # Apply editor changes
luckperms.verbose           # Verbose permission check
luckperms.tree              # Permission tree
luckperms.search            # Search permissions
luckperms.import            # Import data
luckperms.export            # Export data
luckperms.reloadconfig      # Reload config
luckperms.translations      # Manage translations
```

## Building from Source

Requires Rust toolchain with `x86_64-unknown-linux-gnu` target:

```bash
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu
```

The compiled plugin will be at `target/x86_64-unknown-linux-gnu/release/libpumpkin_unified_core.so`.

## Deployment

The plugin is designed for GitHub Actions deployment to a PumpkinMC server:

1. Push to main or create a release
2. GitHub Actions builds the `.so` artifact
3. Deploy workflow copies via SSH and restarts Pumpkin
4. Verification script runs in-game command tests

## Credits

This plugin ports features from:
- **EssentialsX** (https://github.com/EssentialsX/Essentials, MIT License)
- **LuckPerms** (https://github.com/LuckPerms/LuckPerms, MIT License)

Original authors retain copyright. This is a clean-room Rust reimplementation for PumpkinMC, not a fork or wrapper.

## License

Dual licensed under **MIT OR Apache-2.0** (matches `pumpkin-plugin-api`).

See [LICENSE](LICENSE) for details.