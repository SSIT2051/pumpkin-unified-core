use crate::utils::macros::*;
use pumpkin_plugin_api::{
    event::{EventHandler, PlayerJoinEvent, PlayerLeaveEvent, PlayerChatEvent},
    context::Context,
};
use std::sync::Arc;

pub struct EventModule {
    economy: Arc<crate::economy::EconomyModule>,
    warps: Arc<crate::warps::WarpsModule>,
    homes: Arc<crate::homes::HomesModule>,
    tpa: Arc<crate::tpa::TpaModule>,
    kits: Arc<crate::kits::KitsModule>,
    moderation: Arc<crate::moderation::ModerationModule>,
    gamemode: Arc<crate::gamemode::GamemodeModule>,
    permissions: Arc<crate::permissions::PermissionsModule>,
    spawn: Arc<crate::spawn::SpawnModule>,
    utility: Arc<crate::utility::UtilityModule>,
    chat: Arc<crate::chat::ChatModule>,
}

impl EventModule {
    pub fn new(
        economy: Arc<crate::economy::EconomyModule>,
        warps: Arc<crate::warps::WarpsModule>,
        homes: Arc<crate::homes::HomesModule>,
        tpa: Arc<crate::tpa::TpaModule>,
        kits: Arc<crate::kits::KitsModule>,
        moderation: Arc<crate::moderation::ModerationModule>,
        gamemode: Arc<crate::gamemode::GamemodeModule>,
        permissions: Arc<crate::permissions::PermissionsModule>,
        spawn: Arc<crate::spawn::SpawnModule>,
        utility: Arc<crate::utility::UtilityModule>,
        chat: Arc<crate::chat::ChatModule>,
    ) -> Self {
        Self {
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
        }
    }

    pub fn register_handlers(&self, context: &mut Context) {
        let module = self.clone();
        context.register_event_handler(EventHandler::new(move |event: &PlayerJoinEvent| {
            let module = module.clone();
            let player = event.get_player();
            let uuid = player.get_uuid();
            
            if let Some(ban) = module.moderation.is_banned(uuid) {
                player.kick(&format!("&cYou are banned: {}", ban.reason)).ok();
                return;
            }
            
            if let Some(mute) = module.moderation.is_muted(uuid) {
                player.set_muted(true).ok();
            }
            
            if let Some(jail) = module.moderation.is_jailed(uuid) {
                if let Some(jail_loc) = &module.moderation.config.jail_location {
                    player.teleport(jail_loc.world.parse().unwrap(), jail_loc.x, jail_loc.y, jail_loc.z, jail_loc.yaw, jail_loc.pitch).ok();
                }
            }
            
            if let Some(join_msg) = &module.utility.config.events.player_join_message {
                let msg = join_msg.replace("{player}", player.get_name());
                crate::utils::text::format_text(&msg);
            }
        }));
        
        let module = self.clone();
        context.register_event_handler(EventHandler::new(move |event: &PlayerLeaveEvent| {
            let module = module.clone();
            let player = event.get_player();
            let uuid = player.get_uuid();
            
            module.tpa.remove_request(uuid);
            
            if let Some(leave_msg) = &module.utility.config.events.player_leave_message {
                let msg = leave_msg.replace("{player}", player.get_name());
                crate::utils::text::format_text(&msg);
            }
        }));
        
        let module = self.clone();
        context.register_event_handler(EventHandler::new(move |event: &PlayerChatEvent| {
            let module = module.clone();
            let player = event.get_player();
            let message = event.get_message();
            
            if module.moderation.is_muted(player.get_uuid()).is_some() {
                event.set_cancelled(true);
                player.send_message(&crate::utils::text::format_text("&cYou are muted"));
                return;
            }
            
            let formatted = module.chat.format_chat(player, message);
            event.set_message(&formatted);
        }));
    }
}

impl Clone for EventModule {
    fn clone(&self) -> Self {
        Self {
            economy: self.economy.clone(),
            warps: self.warps.clone(),
            homes: self.homes.clone(),
            tpa: self.tpa.clone(),
            kits: self.kits.clone(),
            moderation: self.moderation.clone(),
            gamemode: self.gamemode.clone(),
            permissions: self.permissions.clone(),
            spawn: self.spawn.clone(),
            utility: self.utility.clone(),
            chat: self.chat.clone(),
        }
    }
}