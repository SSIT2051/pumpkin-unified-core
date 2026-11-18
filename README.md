# pumpkin-unified-core

Unified **EssentialsX** + **LuckPerms** plugin for PumpkinMC, written in Rust as a
native **WebAssembly component** (`.wasm`) using the official
[`pumpkin-plugin-api`](https://crates.io/crates/pumpkin-plugin-api).

Pumpkin plugins are WASM components (target `wasm32-wasip2`) loaded by the server's
plugin host — there is no native `.so` plugin API. This project follows the official
Pumpkin plugin documentation exactly.

## Status

| Module | Commands | Status |
|--------|----------|--------|
| Economy | `/balance`, `/bal`, `/money`, `/pay`, `/baltop`, `/eco give\|take\|set` | ✅ ported |
| Warps | `/warp`, `/setwarp`, `/delwarp`, `/warps` | ✅ ported |
| Homes | `/sethome`, `/home`, `/delhome`, `/homes` | ✅ ported |
| TPA | `/tpa`, `/tpahere`, `/tpaccept`, `/tpdeny`, `/tptoggle` | ✅ ported |
| Kits | `/kit`, `/kits`, `/createkit`, `/showkit` | ✅ ported |
| Moderation | `/kick`, `/ban`, `/tempban`, `/unban`, `/mute`, `/unmute`, `/fly`, `/god`, `/heal`, `/feed`, `/vanish` | ✅ ported |
| Gamemode | `/gamemode`, `/gmc`, `/gms`, `/gma`, `/gmsp` | ✅ ported |
| Utility | `/afk`, `/rules`, `/motd`, `/list`, `/whois`, `/speed`, `/compass`, `/top`, `/seen`, `/weather`, `/time`, `/butcher`, `/extinguish` | ✅ ported |
| Chat | `/msg`, `/r`, `/mail`, `/broadcast`, `/broadcastworld` | ✅ ported |
| Spawn | `/spawn`, `/setspawn` | ✅ ported |
| LuckPerms (core) | `/lp user/group/track ...`, `/lp sync`, `/lp info` | ✅ ported |

All modules are ported to the real Pumpkin WASM plugin API. `butcher`, `extinguish`,
and `top` are stubs that notify the player (they require entity/block APIs not yet
wired). Kits currently store item *names* and notify on use; actual item granting
will use the inventory API in a follow-up.

## Architecture

- **Plugin trait** — `impl Plugin` with `metadata`, `on_load`, `on_unload`, registered
  via `pumpkin_plugin_api::register_plugin!`.
- **Commands** — each command is a `CommandHandler` registered with
  `context.register_command(Command::new(...).execute(handler), "permission")`.
  Command trees use chainable `CommandNode` builders (the crate's `chainable_methods`).
- **Permissions** — `context.register_permission(&Permission { ... })`; the LuckPerms
  engine resolves nodes with group inheritance and wildcard matching.
- **Storage** — WASI file-based JSON store (`<data_folder>/data.json`) via `std::fs` +
  `serde_json`. No external database (sqlx/redis are not available in the WASM sandbox).
- **Text** — legacy `&` colour codes are parsed by `TextComponent::from_legacy_string_with_code`.

## Building

Requires the `wasm32-wasip2` target:

```bash
rustup target add wasm32-wasip2
cargo build --release --target wasm32-wasip2
```

The plugin is produced at `target/wasm32-wasip2/release/pumpkin_unified_core.wasm`.
`.cargo/config.toml` already pins the target.

## Installation

1. Copy `pumpkin_unified_core.wasm` into your PumpkinMC `plugins/` directory.
2. Restart the server.
3. The plugin stores data under its WASI data folder as `data.json`.

## Automated pipeline

- `build.yml` — builds the WASM plugin and runs host unit tests.
- `test-vm.yml` — deploys to the pumpkin-vm and runs `scripts/verify.sh` via RCON.
- `deploy.yml` — SSH deployment of the `.wasm` artifact.
- `ai-port/` — Java→Rust translation and RCON test tooling for upstream sync.

## Credits

Ports features from:
- **EssentialsX** (https://github.com/EssentialsX/Essentials, MIT License)
- **LuckPerms** (https://github.com/LuckPerms/LuckPerms, MIT License)

Original authors retain copyright. This is a clean-room Rust reimplementation for
PumpkinMC, not a fork or wrapper.

## License

Dual licensed under **MIT OR Apache-2.0** (matches `pumpkin-plugin-api`).