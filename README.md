# pumpkin-unified-core

Unified **EssentialsX** + **LuckPerms** plugin for PumpkinMC, written in Rust as a
native **WebAssembly component** (`.wasm`) using the official
[`pumpkin-plugin-api`](https://crates.io/crates/pumpkin-plugin-api).

Pumpkin plugins are WASM components (target `wasm32-wasip3`) loaded by the server's
plugin host — there is no native `.so` plugin API. This project follows the official
Pumpkin plugin documentation exactly.

## Status

| Module | Commands | Status |
|--------|----------|--------|
| Economy | `/balance`, `/bal`, `/money`, `/pay`, `/baltop`, `/eco give\|take\|set` | ✅ ported |
| LuckPerms (core) | `/lp user/group/track ...`, `/lp sync`, `/lp info` | ✅ ported |
| Warps | `/warp`, `/setwarp`, `/delwarp`, `/warps` | ⏳ pending |
| Homes | `/sethome`, `/home`, `/delhome`, `/homes` | ⏳ pending |
| TPA | `/tpa`, `/tpahere`, `/tpaccept`, `/tpdeny`, `/tptoggle` | ⏳ pending |
| Kits | `/kit`, `/kits`, `/createkit`, `/showkit` | ⏳ pending |
| Moderation | `/kick`, `/ban`, `/tempban`, `/unban`, `/mute`, `/unmute`, ... | ⏳ pending |
| Gamemode | `/gamemode`, `/gmc`, `/gms`, `/gma`, `/gmsp` | ⏳ pending |
| Utility | `/afk`, `/rules`, `/motd`, `/list`, `/whois`, ... | ⏳ pending |
| Chat | `/msg`, `/r`, `/mail`, `/broadcast` | ⏳ pending |
| Spawn | `/spawn`, `/setspawn` | ⏳ pending |

The remaining modules are ported incrementally following the same pattern as the
economy and permissions modules (see `src/economy/` and `src/permissions/`).

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

Requires the `wasm32-wasip3` target:

```bash
rustup target add wasm32-wasip3
cargo build --release --target wasm32-wasip3
```

The plugin is produced at `target/wasm32-wasip3/release/pumpkin_unified_core.wasm`.
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