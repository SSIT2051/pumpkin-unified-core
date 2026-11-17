use pumpkin_plugin_api::{
    plugin::{Plugin, PluginMetadata},
    command::{Command, CommandNode, CommandHandler},
    event::EventHandler,
    context::Context,
    permission::{Permission, PermissionDefault},
    server::Server,
};
use std::sync::Arc;

mod config;
mod storage;
mod economy;
mod warps;
mod homes;
mod tpa;
mod kits;
mod moderation;
mod gamemode;
mod permissions;
mod spawn;
mod utility;
mod chat;
mod events;
mod utils;

use crate::{
    config::Config,
    storage::StorageManager,
    economy::EconomyModule,
    warps::WarpsModule,
    homes::HomesModule,
    tpa::TpaModule,
    kits::KitsModule,
    moderation::ModerationModule,
    gamemode::GamemodeModule,
    permissions::PermissionsModule,
    spawn::SpawnModule,
    utility::UtilityModule,
    chat::ChatModule,
    events::EventModule,
};

pub struct UnifiedPlugin {
    config: Arc<Config>,
    storage: Arc<StorageManager>,
    economy: Arc<EconomyModule>,
    warps: Arc<WarpsModule>,
    homes: Arc<HomesModule>,
    tpa: Arc<TpaModule>,
    kits: Arc<KitsModule>,
    moderation: Arc<ModerationModule>,
    gamemode: Arc<GamemodeModule>,
    permissions: Arc<PermissionsModule>,
    spawn: Arc<SpawnModule>,
    utility: Arc<UtilityModule>,
    chat: Arc<ChatModule>,
    events: Arc<EventModule>,
}

impl Plugin for UnifiedPlugin {
    fn new() -> Self {
        let config = Arc::new(Config::load());
        let storage = Arc::new(StorageManager::new(&config));
        
        let economy = Arc::new(EconomyModule::with_config(storage.clone(), config.economy.clone()));
        let warps = Arc::new(WarpsModule::with_config(storage.clone(), config.warps.clone()));
        let homes = Arc::new(HomesModule::with_config(storage.clone(), config.homes.clone()));
        let tpa = Arc::new(TpaModule::with_config(config.tpa.clone()));
        let kits = Arc::new(KitsModule::with_config(storage.clone(), config.kits.clone()));
        let moderation = Arc::new(ModerationModule::with_config(storage.clone(), config.moderation.clone()));
        let gamemode = Arc::new(GamemodeModule::new());
        let permissions = Arc::new(PermissionsModule::with_config(storage.clone(), config.permissions.clone()));
        let spawn = Arc::new(SpawnModule::with_config(storage.clone(), config.spawn.clone()));
        let utility = Arc::new(UtilityModule::with_config(config.utility.clone()));
        let chat = Arc::new(ChatModule::new().with_permissions(permissions.clone()));
        let events = Arc::new(EventModule::new(
            economy.clone(),
            warps.clone(),
            homes.clone(),
            tpa.clone(),
            kits.clone(),
            moderation.clone(),
            gamemode.clone(),
            permissions.clone(),
            spawn.clone(),
            utility.clone(),
            chat.clone(),
        ));

        Self {
            config,
            storage,
            economy,
            warps,
            homes,
            tpa,
            kits,
            moderation,
            gamemode,
            permissions,
            spawn,
            utility,
            chat,
            events,
        }
    }

    fn metadata() -> PluginMetadata {
        PluginMetadata {
            name: "pumpkin-unified-core",
            version: "0.1.0",
            authors: vec!["SSIT2051"],
            description: "Unified EssentialsX + LuckPerms native Rust plugin for PumpkinMC",
            api_version: "0.1.0-dev",
        }
    }

    fn on_load(&mut self, context: &mut Context) {
        tracing::info!("Loading pumpkin-unified-core...");

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.storage.init().expect("Failed to initialize storage");
            })
        });

        self.economy.register_commands(context);
        self.warps.register_commands(context);
        self.homes.register_commands(context);
        self.tpa.register_commands(context);
        self.kits.register_commands(context);
        self.moderation.register_commands(context);
        self.gamemode.register_commands(context);
        self.permissions.register_commands(context);
        self.spawn.register_commands(context);
        self.utility.register_commands(context);
        self.chat.register_commands(context);

        self.register_permissions(context);

        self.events.register_handlers(context);

        tracing::info!("pumpkin-unified-core loaded successfully!");
    }

    fn on_unload(&mut self, _context: &mut Context) {
        tracing::info!("Unloading pumpkin-unified-core...");
        self.storage.shutdown();
    }
}

impl UnifiedPlugin {
    fn register_permissions(&self, context: &mut Context) {
        let perms = vec![
            ("essentials.balance", PermissionDefault::Allow),
            ("essentials.balance.others", PermissionDefault::Op),
            ("essentials.pay", PermissionDefault::Allow),
            ("essentials.eco", PermissionDefault::Op),
            ("essentials.eco.give", PermissionDefault::Op),
            ("essentials.eco.take", PermissionDefault::Op),
            ("essentials.eco.set", PermissionDefault::Op),
            ("essentials.warp", PermissionDefault::Allow),
            ("essentials.warp.others", PermissionDefault::Op),
            ("essentials.setwarp", PermissionDefault::Op),
            ("essentials.delwarp", PermissionDefault::Op),
            ("essentials.warps", PermissionDefault::Allow),
            ("essentials.warpinfo", PermissionDefault::Allow),
            ("essentials.home", PermissionDefault::Allow),
            ("essentials.home.others", PermissionDefault::Op),
            ("essentials.sethome", PermissionDefault::Allow),
            ("essentials.delhome", PermissionDefault::Allow),
            ("essentials.homes", PermissionDefault::Allow),
            ("essentials.homes.limit.*", PermissionDefault::Op),
            ("essentials.tpa", PermissionDefault::Allow),
            ("essentials.tpahere", PermissionDefault::Allow),
            ("essentials.tpaccept", PermissionDefault::Allow),
            ("essentials.tpdeny", PermissionDefault::Allow),
            ("essentials.tptoggle", PermissionDefault::Allow),
            ("essentials.kit", PermissionDefault::Allow),
            ("essentials.kit.*", PermissionDefault::Op),
            ("essentials.kits", PermissionDefault::Allow),
            ("essentials.createkit", PermissionDefault::Op),
            ("essentials.showkit", PermissionDefault::Op),
            ("essentials.kick", PermissionDefault::Op),
            ("essentials.ban", PermissionDefault::Op),
            ("essentials.tempban", PermissionDefault::Op),
            ("essentials.unban", PermissionDefault::Op),
            ("essentials.mute", PermissionDefault::Op),
            ("essentials.unmute", PermissionDefault::Op),
            ("essentials.jail", PermissionDefault::Op),
            ("essentials.unjail", PermissionDefault::Op),
            ("essentials.setjail", PermissionDefault::Op),
            ("essentials.deljail", PermissionDefault::Op),
            ("essentials.jails", PermissionDefault::Op),
            ("essentials.fly", PermissionDefault::Op),
            ("essentials.fly.others", PermissionDefault::Op),
            ("essentials.god", PermissionDefault::Op),
            ("essentials.god.others", PermissionDefault::Op),
            ("essentials.heal", PermissionDefault::Op),
            ("essentials.heal.others", PermissionDefault::Op),
            ("essentials.feed", PermissionDefault::Op),
            ("essentials.feed.others", PermissionDefault::Op),
            ("essentials.vanish", PermissionDefault::Op),
            ("essentials.gamemode", PermissionDefault::Op),
            ("essentials.gamemode.survival", PermissionDefault::Op),
            ("essentials.gamemode.creative", PermissionDefault::Op),
            ("essentials.gamemode.adventure", PermissionDefault::Op),
            ("essentials.gamemode.spectator", PermissionDefault::Op),
            ("essentials.spawn", PermissionDefault::Allow),
            ("essentials.setspawn", PermissionDefault::Op),
            ("essentials.afk", PermissionDefault::Allow),
            ("essentials.afk.others", PermissionDefault::Op),
            ("essentials.rules", PermissionDefault::Allow),
            ("essentials.motd", PermissionDefault::Allow),
            ("essentials.list", PermissionDefault::Allow),
            ("essentials.whois", PermissionDefault::Op),
            ("essentials.speed", PermissionDefault::Op),
            ("essentials.speed.others", PermissionDefault::Op),
            ("essentials.compass", PermissionDefault::Allow),
            ("essentials.top", PermissionDefault::Allow),
            ("essentials.back", PermissionDefault::Allow),
            ("essentials.back.ondeath", PermissionDefault::Allow),
            ("essentials.seen", PermissionDefault::Allow),
            ("essentials.repair", PermissionDefault::Op),
            ("essentials.repair.others", PermissionDefault::Op),
            ("essentials.disposal", PermissionDefault::Op),
            ("essentials.workbench", PermissionDefault::Allow),
            ("essentials.anvil", PermissionDefault::Allow),
            ("essentials.enderchest", PermissionDefault::Allow),
            ("essentials.enderchest.others", PermissionDefault::Op),
            ("essentials.weather", PermissionDefault::Op),
            ("essentials.time", PermissionDefault::Op),
            ("essentials.lightning", PermissionDefault::Op),
            ("essentials.butcher", PermissionDefault::Op),
            ("essentials.extinguish", PermissionDefault::Op),
            ("essentials.msg", PermissionDefault::Allow),
            ("essentials.reply", PermissionDefault::Allow),
            ("essentials.mail", PermissionDefault::Allow),
            ("essentials.mail.send", PermissionDefault::Allow),
            ("essentials.broadcast", PermissionDefault::Op),
            ("essentials.broadcastworld", PermissionDefault::Op),
            ("luckperms.user.info", PermissionDefault::Op),
            ("luckperms.user.editor", PermissionDefault::Op),
            ("luckperms.user.promote", PermissionDefault::Op),
            ("luckperms.user.demote", PermissionDefault::Op),
            ("luckperms.user.showtracks", PermissionDefault::Op),
            ("luckperms.user.clear", PermissionDefault::Op),
            ("luckperms.user.clone", PermissionDefault::Op),
            ("luckperms.user.permission", PermissionDefault::Op),
            ("luckperms.user.parent", PermissionDefault::Op),
            ("luckperms.user.meta", PermissionDefault::Op),
            ("luckperms.group.info", PermissionDefault::Op),
            ("luckperms.group.editor", PermissionDefault::Op),
            ("luckperms.group.listmembers", PermissionDefault::Op),
            ("luckperms.group.setweight", PermissionDefault::Op),
            ("luckperms.group.setdisplayname", PermissionDefault::Op),
            ("luckperms.group.showtracks", PermissionDefault::Op),
            ("luckperms.group.clear", PermissionDefault::Op),
            ("luckperms.group.rename", PermissionDefault::Op),
            ("luckperms.group.clone", PermissionDefault::Op),
            ("luckperms.group.permission", PermissionDefault::Op),
            ("luckperms.group.parent", PermissionDefault::Op),
            ("luckperms.group.meta", PermissionDefault::Op),
            ("luckperms.track.createtrack", PermissionDefault::Op),
            ("luckperms.track.deletetrack", PermissionDefault::Op),
            ("luckperms.track.listtracks", PermissionDefault::Op),
            ("luckperms.log.recent", PermissionDefault::Op),
            ("luckperms.log.search", PermissionDefault::Op),
            ("luckperms.log.notify", PermissionDefault::Op),
            ("luckperms.sync", PermissionDefault::Op),
            ("luckperms.info", PermissionDefault::Op),
            ("luckperms.editor", PermissionDefault::Op),
            ("luckperms.applyedits", PermissionDefault::Op),
            ("luckperms.verbose", PermissionDefault::Op),
            ("luckperms.tree", PermissionDefault::Op),
            ("luckperms.search", PermissionDefault::Op),
            ("luckperms.import", PermissionDefault::Op),
            ("luckperms.export", PermissionDefault::Op),
            ("luckperms.reloadconfig", PermissionDefault::Op),
            ("luckperms.translations", PermissionDefault::Op),
        ];

        for (name, default) in perms {
            context.register_permission(Permission::new(name, default));
        }
    }
}

pumpkin_plugin_api::register_plugin!(UnifiedPlugin);