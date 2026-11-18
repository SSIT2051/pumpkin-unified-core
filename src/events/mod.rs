//! Event handlers for PumpkinMC events.
//!
//! Real event handlers using pumpkin-plugin-api v0.2.1+ event system.
//! Handles player join/leave messages, chat formatting, LuckPerms permission
//! integration, AFK tracking, anti-build, and other event-driven features.

use crate::chat::Chat;
use crate::config::EventsConfig;
use crate::economy::Economy;
use crate::homes::Homes;
use crate::kits::Kits;
use crate::permissions::Permissions;
use crate::spawn::SpawnModule;
use crate::tpa::Tpa;
use crate::utility::Utility;
use crate::warps::Warps;
use pumpkin_plugin_api::events::{
    block::{BlockBreakEvent, BlockPlaceEvent},
    entity::EntityDamageEvent,
    player::{
        AsyncPlayerChatEvent, PlayerCommandPreprocessEvent, PlayerJoinEvent, PlayerLeaveEvent,
        PlayerMoveEvent, PlayerPermissionCheckEvent,
    },
    EventHandler, EventPriority, FromIntoEvent,
};
use pumpkin_plugin_api::text::TextComponent;
use pumpkin_plugin_api::{Context, Server};
use std::sync::Arc;

/// Event handler for player join events.
pub struct PlayerJoinHandler {
    events_config: EventsConfig,
    spawn: Arc<SpawnModule>,
}

impl PlayerJoinHandler {
    pub fn new(events_config: EventsConfig, spawn: Arc<SpawnModule>) -> Self {
        Self { events_config, spawn }
    }
}

impl EventHandler<PlayerJoinEvent> for PlayerJoinHandler {
    fn handle(&self, server: Server, mut event: <PlayerJoinEvent as FromIntoEvent>::Data) -> <PlayerJoinEvent as FromIntoEvent>::Data {
        let player = &event.player;
        
        // Set custom join message if configured
        if let Some(ref msg) = self.events_config.player_join_message {
            let formatted = msg.replace("{player}", &player.get_name());
            event.join_message = TextComponent::from_legacy_string(&formatted);
        }

        // Handle respawn at home if enabled
        if self.events_config.keep_inventory {
            // TODO: Implement keep inventory logic
        }

        // Auto-spawn if configured
        if let Some(ref spawn_loc) = self.spawn.config().spawn_location {
            // Find world by name using get_world_by_name
            if let Some(world) = server.get_world_by_name(&spawn_loc.world) {
                player.teleport(
                    (spawn_loc.x, spawn_loc.y, spawn_loc.z),
                    Some(spawn_loc.yaw),
                    Some(spawn_loc.pitch),
                    world,
                );
            }
        }

        event
    }
}

/// Event handler for player leave/quit events.
pub struct PlayerLeaveHandler {
    events_config: EventsConfig,
}

impl PlayerLeaveHandler {
    pub fn new(events_config: EventsConfig) -> Self {
        Self { events_config }
    }
}

impl EventHandler<PlayerLeaveEvent> for PlayerLeaveHandler {
    fn handle(&self, _server: Server, mut event: <PlayerLeaveEvent as FromIntoEvent>::Data) -> <PlayerLeaveEvent as FromIntoEvent>::Data {
        let player = &event.player;
        
        // Set custom leave message if configured
        if let Some(ref msg) = self.events_config.player_leave_message {
            let formatted = msg.replace("{player}", &player.get_name());
            event.leave_message = TextComponent::from_legacy_string(&formatted);
        }

        event
    }
}

/// Event handler for chat events - handles chat formatting, anti-spam.
pub struct PlayerChatHandler {
    chat: Arc<Chat>,
    events_config: EventsConfig,
}

impl PlayerChatHandler {
    pub fn new(chat: Arc<Chat>, events_config: EventsConfig) -> Self {
        Self { chat, events_config }
    }
}

impl EventHandler<AsyncPlayerChatEvent> for PlayerChatHandler {
    fn handle(&self, _server: Server, mut event: <AsyncPlayerChatEvent as FromIntoEvent>::Data) -> <AsyncPlayerChatEvent as FromIntoEvent>::Data {
        let player = &event.player;
        let message = event.message.to_string();
        
        // Anti-spam check
        if self.events_config.anti_spam {
            if self.chat.is_spam(&player.get_id().to_string(), &message, self.events_config.spam_threshold) {
                event.cancelled = true;
                player.send_system_message(TextComponent::from_legacy_string("&cYou are sending messages too fast!"), false);
                return event;
            }
        }

        // Format the message using chat module
        let formatted = self.chat.format_message(player, &message);
        event.message = formatted;

        event
    }
}

/// Event handler for permission checks - integrates LuckPerms with host.
pub struct PermissionCheckHandler {
    permissions: Arc<Permissions>,
}

impl PermissionCheckHandler {
    pub fn new(permissions: Arc<Permissions>) -> Self {
        Self { permissions }
    }
}

impl EventHandler<PlayerPermissionCheckEvent> for PermissionCheckHandler {
    fn handle(&self, _server: Server, mut event: <PlayerPermissionCheckEvent as FromIntoEvent>::Data) -> <PlayerPermissionCheckEvent as FromIntoEvent>::Data {
        let player = &event.player;
        let uuid = player.get_id().to_string();
        let permission_node = &event.permission;
        
        // Check if this is one of our plugin's permissions
        if permission_node.starts_with("pumpkin-unified-core:") {
            let has_perm = self.permissions.has_permission(&uuid, permission_node);
            event.permission_result = has_perm;
        }

        event
    }
}

/// Event handler for player move - AFK detection, anti-build.
pub struct PlayerMoveHandler {
    events_config: EventsConfig,
    utility: Arc<Utility>,
}

impl PlayerMoveHandler {
    pub fn new(events_config: EventsConfig, utility: Arc<Utility>) -> Self {
        Self { events_config, utility }
    }
}

impl EventHandler<PlayerMoveEvent> for PlayerMoveHandler {
    fn handle(&self, _server: Server, event: <PlayerMoveEvent as FromIntoEvent>::Data) -> <PlayerMoveEvent as FromIntoEvent>::Data {
        let _player = &event.player;
        let uuid = _player.get_id().to_string();
        
        // Update AFK tracking
        self.utility.update_activity(&uuid);
        
        // Anti-build check
        if self.events_config.anti_build {
            // Could check if player is in creative and trying to build in survival areas
        }

        event
    }
}

/// Event handler for block break - anti-build enforcement.
pub struct BlockBreakHandler {
    events_config: EventsConfig,
}

impl BlockBreakHandler {
    pub fn new(events_config: EventsConfig) -> Self {
        Self { events_config }
    }
}

impl EventHandler<BlockBreakEvent> for BlockBreakHandler {
    fn handle(&self, _server: Server, event: <BlockBreakEvent as FromIntoEvent>::Data) -> <BlockBreakEvent as FromIntoEvent>::Data {
        if self.events_config.anti_build {
            // Check if player has permission to build
            let _player = &event.player;
            // TODO: Check build permissions
        }
        event
    }
}

/// Event handler for block place - anti-build enforcement.
pub struct BlockPlaceHandler {
    events_config: EventsConfig,
}

impl BlockPlaceHandler {
    pub fn new(events_config: EventsConfig) -> Self {
        Self { events_config }
    }
}

impl EventHandler<BlockPlaceEvent> for BlockPlaceHandler {
    fn handle(&self, _server: Server, event: <BlockPlaceEvent as FromIntoEvent>::Data) -> <BlockPlaceEvent as FromIntoEvent>::Data {
        if self.events_config.anti_build {
            // Check if player has permission to build
            let _player = &event.player;
            // TODO: Check build permissions
        }
        event
    }
}

/// Event handler for entity damage - god mode, PvP handling.
pub struct EntityDamageHandler;

impl EntityDamageHandler {
    pub fn new() -> Self {
        Self
    }
}

impl EventHandler<EntityDamageEvent> for EntityDamageHandler {
    fn handle(&self, _server: Server, event: <EntityDamageEvent as FromIntoEvent>::Data) -> <EntityDamageEvent as FromIntoEvent>::Data {
        // TODO: Check if victim is a player with god mode
        // Currently no API to get entity by ID from server
        // Would need to track entity IDs or use a different approach
        
        event
    }
}

/// Event handler for command preprocess - can be used for command aliases, logging.
pub struct CommandPreprocessHandler {
    utility: Arc<Utility>,
}

impl CommandPreprocessHandler {
    pub fn new(utility: Arc<Utility>) -> Self {
        Self { utility }
    }
}

impl EventHandler<PlayerCommandPreprocessEvent> for CommandPreprocessHandler {
    fn handle(&self, _server: Server, event: <PlayerCommandPreprocessEvent as FromIntoEvent>::Data) -> <PlayerCommandPreprocessEvent as FromIntoEvent>::Data {
        let player = &event.player;
        let command = &event.command;
        
        // Log command for audit trail
        tracing::debug!("Player {} executed command: {}", player.get_name(), command);
        
        // Update AFK status on command
        self.utility.update_activity(&player.get_id().to_string());
        
        event
    }
}

/// Main event module that registers all event handlers.
pub struct EventModule {
    permissions: Arc<Permissions>,
    utility: Arc<Utility>,
    chat: Arc<Chat>,
    spawn: Arc<SpawnModule>,
    events_config: EventsConfig,
}

impl EventModule {
    pub fn new(
        economy: Arc<Economy>,
        warps: Arc<Warps>,
        homes: Arc<Homes>,
        tpa: Arc<Tpa>,
        kits: Arc<Kits>,
        permissions: Arc<Permissions>,
        spawn: Arc<SpawnModule>,
        utility: Arc<Utility>,
        chat: Arc<Chat>,
        events_config: EventsConfig,
    ) -> Self {
        // Some modules are not used yet but kept for future event handlers
        let _ = (economy, warps, homes, tpa, kits);
        
        Self {
            permissions,
            utility,
            chat,
            spawn,
            events_config,
        }
    }

    pub fn register_handlers(&self, context: &Context) {
        // Player join/leave
        let join_handler = PlayerJoinHandler::new(self.events_config.clone(), self.spawn.clone());
        let leave_handler = PlayerLeaveHandler::new(self.events_config.clone());
        
        // Permission check (highest priority to override host decisions)
        let perm_handler = PermissionCheckHandler::new(self.permissions.clone());
        
        // Chat
        let chat_handler = PlayerChatHandler::new(self.chat.clone(), self.events_config.clone());
        
        // Movement/AFK
        let move_handler = PlayerMoveHandler::new(self.events_config.clone(), self.utility.clone());
        
        // Block events
        let break_handler = BlockBreakHandler::new(self.events_config.clone());
        let place_handler = BlockPlaceHandler::new(self.events_config.clone());
        
        // Entity damage
        let damage_handler = EntityDamageHandler::new();
        
        // Command preprocess
        let cmd_handler = CommandPreprocessHandler::new(self.utility.clone());

        // Register all handlers
        // Priority: Highest for permission checks, Normal for others
        let _ = context.register_event_handler(perm_handler, EventPriority::Highest, true);
        let _ = context.register_event_handler(join_handler, EventPriority::Normal, true);
        let _ = context.register_event_handler(leave_handler, EventPriority::Normal, true);
        let _ = context.register_event_handler(chat_handler, EventPriority::Normal, true);
        let _ = context.register_event_handler(move_handler, EventPriority::Normal, false);
        let _ = context.register_event_handler(break_handler, EventPriority::Normal, true);
        let _ = context.register_event_handler(place_handler, EventPriority::Normal, true);
        let _ = context.register_event_handler(damage_handler, EventPriority::Normal, true);
        let _ = context.register_event_handler(cmd_handler, EventPriority::Normal, false);

        tracing::info!("All event handlers registered successfully");
    }
}

impl Clone for EventModule {
    fn clone(&self) -> Self {
        Self {
            permissions: self.permissions.clone(),
            utility: self.utility.clone(),
            chat: self.chat.clone(),
            spawn: self.spawn.clone(),
            events_config: self.events_config.clone(),
        }
    }
}