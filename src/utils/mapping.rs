//! API Mapping Layer: Bukkit/Spigot → PumpkinMC
//!
//! This module provides type mappings and conversion utilities for translating
//! Bukkit/Spigot API calls to PumpkinMC equivalents.

use pumpkin_plugin_api::{
    world::{Position, World, Gamemode},
    player::Player,
    inventory::ItemStack,
    event::EventHandler,
    command::{Command, CommandHandler},
    permission::{Permission, PermissionDefault},
};

/// Map Bukkit Location to Pumpkin Position
/// org.bukkit.Location → pumpkin_plugin_api::world::Position
pub fn location_to_position(loc: (String, f64, f64, f64, f32, f32)) -> Position {
    Position {
        world: loc.0,
        x: loc.1,
        y: loc.2,
        z: loc.3,
        yaw: loc.4,
        pitch: loc.5,
    }
}

/// Map Bukkit World to Pumpkin World
/// org.bukkit.World → pumpkin_plugin_api::world::World
pub fn world_from_string(world_name: &str) -> Option<World> {
    // World resolution would be done by server context
    Some(World {
        name: world_name.to_string(),
    })
}

/// Map Bukkit Player to Pumpkin Player
/// org.bukkit.entity.Player → pumpkin_plugin_api::player::Player
pub fn player_to_pumpkin(player: &Player) -> Player {
    player.clone()
}

/// Map Bukkit ItemStack to Pumpkin ItemStack
/// org.bukkit.inventory.ItemStack → pumpkin_plugin_api::inventory::ItemStack
pub fn itemstack_to_pumpkin(item: &ItemStack) -> ItemStack {
    item.clone()
}

/// Map Bukkit Gamemode to Pumpkin Gamemode
/// org.bukkit.GameMode → pumpkin_plugin_api::world::Gamemode
pub fn gamemode_from_string(mode: &str) -> Option<Gamemode> {
    match mode.to_lowercase().as_str() {
        "survival" | "0" | "s" => Some(Gamemode::Survival),
        "creative" | "1" | "c" => Some(Gamemode::Creative),
        "adventure" | "2" | "a" => Some(Gamemode::Adventure),
        "spectator" | "3" | "sp" => Some(Gamemode::Spectator),
        _ => None,
    }
}

/// Map Bukkit permission to Pumpkin Permission
/// org.bukkit.permissions.Permission → pumpkin_plugin_api::permission::Permission
pub fn permission_from_bukkit(name: &str, default: &str) -> Permission {
    let default_enum = match default.to_lowercase().as_str() {
        "true" | "allow" => PermissionDefault::Allow,
        "false" | "deny" => PermissionDefault::Deny,
        "op" => PermissionDefault::Op,
        "notop" => PermissionDefault::NotOp,
        _ => PermissionDefault::Deny,
    };
    Permission::new(name, default_enum)
}

/// Helper for converting Bukkit event handlers to Pumpkin
/// org.bukkit.event.Listener → pumpkin_plugin_api::event::EventHandler
pub fn create_event_handler<F>(handler: F) -> EventHandler
where
    F: Fn() + Send + Sync + 'static,
{
    EventHandler::new(handler)
}

/// Helper for creating Pumpkin commands from Bukkit command patterns
/// org.bukkit.command.CommandExecutor → pumpkin_plugin_api::command::CommandHandler
pub fn create_command_builder(name: &str, permission: &str, description: &str) -> Command {
    Command::new(name)
        .permission(permission)
        .description(description)
}

/// Configuration mapping utilities
/// org.bukkit.configuration.file.FileConfiguration → config-rs
pub struct ConfigMapping;

impl ConfigMapping {
    /// Convert Bukkit config section to our Config struct
    pub fn map_config_section<T>(_section: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        // This would use config-rs to deserialize
        None
    }

    /// Get string value from config
    pub fn get_string(config: &config::Config, key: &str, default: &str) -> String {
        config.get_string(key).unwrap_or_else(|_| default.to_string())
    }

    /// Get i64 value from config
    pub fn get_i64(config: &config::Config, key: &str, default: i64) -> i64 {
        config.get_int(key).unwrap_or(default)
    }

    /// Get f64 value from config
    pub fn get_f64(config: &config::Config, key: &str, default: f64) -> f64 {
        config.get_float(key).unwrap_or(default)
    }

    /// Get bool value from config
    pub fn get_bool(config: &config::Config, key: &str, default: bool) -> bool {
        config.get_bool(key).unwrap_or(default)
    }

    /// Get string array from config
    pub fn get_string_array(config: &config::Config, key: &str) -> Vec<String> {
        config.get_array(key).ok().map(|arr| {
            arr.into_iter()
                .filter_map(|v| v.into_str().ok())
                .collect()
        }).unwrap_or_default()
    }
}

/// Scheduler mapping utilities
/// org.bukkit.scheduler.BukkitScheduler → tokio
pub struct SchedulerMapping;

impl SchedulerMapping {
    /// Run task asynchronously (Bukkit scheduler.runTaskAsynchronously)
    pub fn run_async<F>(task: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(task);
    }

    /// Run task later (Bukkit scheduler.runTaskLater)
    pub fn run_later<F>(task: F, delay_ms: u64)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
            task.await;
        });
    }

    /// Run task timer (Bukkit scheduler.runTaskTimer)
    pub fn run_timer<F>(task: F, delay_ms: u64, period_ms: u64)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(period_ms));
            loop {
                interval.tick().await;
                task.await;
            }
        });
    }
}

/// Metadata store mapping
/// org.bukkit.metadata.MetadataStore → DashMap
use dashmap::DashMap;
use std::sync::Arc;

pub struct MetadataStore {
    data: Arc<DashMap<String, Vec<(String, String)>>>,
}

impl MetadataStore {
    pub fn new() -> Self {
        Self {
            data: Arc::new(DashMap::new()),
        }
    }

    pub fn set(&self, subject: &str, key: &str, value: &str) {
        self.data
            .entry(subject.to_string())
            .or_insert_with(Vec::new)
            .push((key.to_string(), value.to_string()));
    }

    pub fn get(&self, subject: &str, key: &str) -> Option<String> {
        self.data
            .get(subject)
            .and_then(|vec| {
                vec.iter()
                    .find(|(k, _)| k == key)
                    .map(|(_, v)| v.clone())
            })
    }

    pub fn remove(&self, subject: &str, key: &str) -> Option<String> {
        if let Some(mut vec) = self.data.get_mut(subject) {
            if let Some(pos) = vec.iter().position(|(k, _)| k == key) {
                return Some(vec.remove(pos).1);
            }
        }
        None
    }

    pub fn get_all(&self, subject: &str) -> Vec<(String, String)> {
        self.data
            .get(subject)
            .map(|vec| vec.clone())
            .unwrap_or_default()
    }
}

/// Utility for converting Bukkit ChatColor to Pumpkin TextComponent
/// org.bukkit.ChatColor → pumpkin_plugin_api::text::TextComponent
pub fn chat_color_to_text_component(text: &str) -> pumpkin_plugin_api::text::TextComponent {
    crate::utils::text::format_text(text)
}

/// Utility for parsing Bukkit command arguments
/// String[] args → Vec<String>
pub fn parse_args(args: &[String]) -> Vec<String> {
    args.to_vec()
}

/// Utility for converting Bukkit ItemStack array to Pumpkin inventory
/// ItemStack[] → Vec<ItemStack>
pub fn item_array_to_inventory(items: &[ItemStack]) -> Vec<ItemStack> {
    items.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gamemode_mapping() {
        assert_eq!(gamemode_from_string("survival"), Some(Gamemode::Survival));
        assert_eq!(gamemode_from_string("creative"), Some(Gamemode::Creative));
        assert_eq!(gamemode_from_string("0"), Some(Gamemode::Survival));
        assert_eq!(gamemode_from_string("1"), Some(Gamemode::Creative));
        assert_eq!(gamemode_from_string("invalid"), None);
    }

    #[test]
    fn test_permission_mapping() {
        let perm = permission_from_bukkit("test.permission", "op");
        assert_eq!(perm.name, "test.permission");
    }

    #[test]
    fn test_location_mapping() {
        let pos = location_to_position(("world".to_string(), 10.0, 64.0, 20.0, 90.0, 0.0));
        assert_eq!(pos.world, "world");
        assert_eq!(pos.x, 10.0);
        assert_eq!(pos.y, 64.0);
        assert_eq!(pos.z, 20.0);
    }
}