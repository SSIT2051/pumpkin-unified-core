use pumpkin_plugin_api::{
    Context, Plugin, PluginMetadata, Result,
    permission::{Permission, PermissionDefault, PermissionLevel},
};
use std::sync::Arc;

mod chat;
mod config;
mod economy;
mod events;
mod gamemode;
mod homes;
mod kits;
mod moderation;
mod permissions;
mod spawn;
mod storage;
mod tpa;
mod utility;
mod utils;
mod warps;

use chat::Chat;
use config::Config;
use economy::Economy;
use events::EventModule;
use homes::Homes;
use kits::Kits;
use moderation::Moderation;
use permissions::Permissions;
use spawn::SpawnModule;
use storage::Storage;
use tpa::Tpa;
use utility::Utility;
use warps::Warps;

pub struct UnifiedPlugin;

impl Plugin for UnifiedPlugin {
    fn new() -> Self {
        UnifiedPlugin
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "pumpkin-unified-core".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: vec!["SSIT2051".into()],
            description: "Unified EssentialsX + LuckPerms plugin for PumpkinMC".into(),
            dependencies: vec![],
            permissions: vec![],
        }
    }

    fn on_load(&self, context: Context) -> Result<()> {
        tracing::info!("Loading pumpkin-unified-core...");

        // Load configuration from file (config.yml/toml) and environment variables (PUC_ prefix).
        let data_folder = context.get_data_folder();
        let config = Arc::new(Config::load(Some(&data_folder)));
        
        // Initialize storage with configured backend
        let storage_backend = match config.storage.backend {
            crate::config::StorageBackendConfig::Json => crate::storage::StorageBackendType::Json,
            crate::config::StorageBackendConfig::Sqlite => crate::storage::StorageBackendType::Sqlite,
            crate::config::StorageBackendConfig::Redis => {
                tracing::warn!("Redis backend not yet implemented, falling back to JSON");
                crate::storage::StorageBackendType::Json
            }
            crate::config::StorageBackendConfig::Mysql => {
                tracing::warn!("MySQL backend not yet implemented, falling back to JSON");
                crate::storage::StorageBackendType::Json
            }
        };
        
        let storage = Arc::new(crate::storage::Storage::new(&data_folder, storage_backend).map_err(|e| e.to_string())?);

        let economy = Arc::new(Economy::new(storage.clone(), config.economy.clone()));
        let warps = Arc::new(Warps::new(storage.clone(), config.warps.clone()));
        let homes = Arc::new(Homes::new(storage.clone(), config.homes.clone()));
        let tpa = Arc::new(Tpa::new(storage.clone(), config.tpa.clone()));
        let kits = Arc::new(Kits::new(storage.clone(), config.kits.clone()));
        let moderation = Arc::new(Moderation::new(storage.clone(), config.moderation.clone()));
        let permissions = Arc::new(Permissions::new(storage.clone(), config.permissions.clone()));
        let spawn = Arc::new(SpawnModule::new(storage.clone(), config.spawn.clone()));
        let utility = Arc::new(Utility::new(storage.clone(), config.utility.clone()));
        let chat = Arc::new(Chat::new(storage.clone(), config.chat.clone(), permissions.clone()));

        // Register event handlers.
        let event_module = EventModule::new(
            economy.clone(),
            warps.clone(),
            homes.clone(),
            tpa.clone(),
            kits.clone(),
            permissions.clone(),
            spawn.clone(),
            utility.clone(),
            chat.clone(),
            config.events.clone(),
        );
        event_module.register_handlers(&context);

        // Register commands.
        permissions::commands::register(permissions.clone(), &context);
        economy::commands::register(economy.clone(), &context);
        warps::commands::register(warps.clone(), &context);
        homes::commands::register(homes.clone(), &context);
        tpa::commands::register(tpa.clone(), &context);
        kits::commands::register(kits.clone(), &context);
        moderation::commands::register(moderation.clone(), &context);
        gamemode::commands::register(&context);
        spawn::commands::register(spawn.clone(), &context);
        utility::commands::register(utility.clone(), &context);
        chat::commands::register(chat.clone(), &context);

        register_permissions(&context)?;

        tracing::info!("pumpkin-unified-core loaded successfully");
        Ok(())
    }

    fn on_unload(&self, _context: Context) -> Result<()> {
        tracing::info!("Unloading pumpkin-unified-core...");
        Ok(())
    }
}

fn permission(node: &str, description: &str, default: PermissionDefault) -> Permission {
    Permission {
        // PumpkinMC requires the plugin namespace prefix on every registered
        // permission; call sites keep using short LuckPerms-style nodes.
        node: crate::utils::perm::namespaced(node),
        description: description.to_string(),
        default,
        children: Vec::new(),
    }
}

fn register_permissions(context: &Context) -> Result<()> {
    let allow = PermissionDefault::Allow;
    let op = || PermissionDefault::Op(PermissionLevel::Four);

    let perms = vec![
        // Economy
        permission("essentials.balance", "Check your balance", allow),
        permission("essentials.balance.others", "Check others' balances", allow),
        permission("essentials.pay", "Pay another player", allow),
        permission("essentials.balancetop", "Show top balances", allow),
        permission("essentials.eco", "Economy admin", op()),
        // Warps
        permission("essentials.warp", "Use warps", allow),
        permission("essentials.setwarp", "Create warps", op()),
        permission("essentials.delwarp", "Delete warps", op()),
        permission("essentials.warps", "List warps", allow),
        // Homes
        permission("essentials.home", "Teleport to home", allow),
        permission("essentials.sethome", "Set a home", allow),
        permission("essentials.delhome", "Delete a home", allow),
        permission("essentials.homes", "List homes", allow),
        // TPA
        permission("essentials.tpa", "Request teleport", allow),
        permission("essentials.tpahere", "Request teleport here", allow),
        permission("essentials.tpaccept", "Accept teleport request", allow),
        permission("essentials.tpdeny", "Deny teleport request", allow),
        permission("essentials.tptoggle", "Toggle teleport requests", allow),
        // Kits
        permission("essentials.kit", "Use kits", allow),
        permission("essentials.kits", "List kits", allow),
        permission("essentials.createkit", "Create kits", op()),
        permission("essentials.showkit", "Show kit contents", op()),
        // Moderation
        permission("essentials.kick", "Kick players", op()),
        permission("essentials.ban", "Ban players", op()),
        permission("essentials.tempban", "Temporarily ban players", op()),
        permission("essentials.unban", "Unban players", op()),
        permission("essentials.mute", "Mute players", op()),
        permission("essentials.unmute", "Unmute players", op()),
        permission("essentials.fly", "Toggle flight", op()),
        permission("essentials.god", "Toggle god mode", op()),
        permission("essentials.heal", "Heal players", op()),
        permission("essentials.feed", "Feed players", op()),
        permission("essentials.vanish", "Toggle vanish", op()),
        // Gamemode
        permission("essentials.gamemode", "Change gamemode", op()),
        // Spawn
        permission("essentials.spawn", "Teleport to spawn", allow),
        permission("essentials.setspawn", "Set spawn point", op()),
        // Utility
        permission("essentials.afk", "Toggle AFK", allow),
        permission("essentials.rules", "View rules", allow),
        permission("essentials.motd", "View MOTD", allow),
        permission("essentials.list", "List players", allow),
        permission("essentials.whois", "View player info", allow),
        permission("essentials.speed", "Set speed", op()),
        permission("essentials.compass", "Show position", allow),
        permission("essentials.top", "Teleport to top", allow),
        permission("essentials.seen", "Check last seen", allow),
        permission("essentials.weather", "Change weather", op()),
        permission("essentials.time", "Change time", op()),
        permission("essentials.butcher", "Kill mobs", op()),
        permission("essentials.extinguish", "Extinguish fires", op()),
        // Chat
        permission("essentials.msg", "Send private messages", allow),
        permission("essentials.reply", "Reply to messages", allow),
        permission("essentials.mail", "Manage mail", allow),
        permission("essentials.broadcast", "Broadcast messages", op()),
        permission("essentials.broadcastworld", "Broadcast to world", op()),
        // LuckPerms
        permission("luckperms.admin", "LuckPerms administration", op()),
        permission("luckperms.user.info", "View user info", op()),
        permission("luckperms.user.permission", "Manage user permissions", op()),
        permission("luckperms.user.parent", "Manage user groups", op()),
        permission("luckperms.group.info", "View group info", op()),
        permission("luckperms.group.permission", "Manage group permissions", op()),
        permission("luckperms.group.parent", "Manage group inheritance", op()),
        permission("luckperms.track", "Manage tracks", op()),
        permission("luckperms.sync", "Sync permissions", op()),
        permission("luckperms.info", "Plugin info", op()),
    ];

    for perm in &perms {
        context.register_permission(perm)?;
    }
    Ok(())
}

pumpkin_plugin_api::register_plugin!(UnifiedPlugin);