use config::Config as ConfigRs;
use serde::{Deserialize, Serialize};
use crate::storage::StorageBackend;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub storage: StorageConfig,
    pub economy: EconomyConfig,
    pub warps: WarpsConfig,
    pub homes: HomesConfig,
    pub tpa: TpaConfig,
    pub kits: KitsConfig,
    pub moderation: ModerationConfig,
    pub gamemode: GamemodeConfig,
    pub permissions: PermissionsConfig,
    pub spawn: SpawnConfig,
    pub utility: UtilityConfig,
    pub chat: ChatConfig,
    pub events: EventsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackendConfig {
    Json,
    Sqlite,
    Redis,
    Mysql,
}

impl Default for StorageBackendConfig {
    fn default() -> Self {
        Self::Json
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub backend: StorageBackendConfig,
    pub sqlite: SqliteConfig,
    pub redis: RedisConfig,
    pub mysql: MysqlConfig,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackendConfig::default(),
            sqlite: SqliteConfig::default(),
            redis: RedisConfig::default(),
            mysql: MysqlConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqliteConfig {
    pub path: String,
    pub max_connections: u32,
}

impl Default for SqliteConfig {
    fn default() -> Self {
        Self {
            path: "plugins/pumpkin-unified-core/data.db".to_string(),
            max_connections: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            max_connections: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MysqlConfig {
    pub url: String,
    pub max_connections: u32,
}

impl Default for MysqlConfig {
    fn default() -> Self {
        Self {
            url: "mysql://user:pass@localhost/pumpkin".to_string(),
            max_connections: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyConfig {
    pub starting_balance: f64,
    pub currency_symbol: String,
    pub currency_format: String,
    pub max_money: f64,
    pub min_money: f64,
    pub log_transactions: bool,
}

impl Default for EconomyConfig {
    fn default() -> Self {
        Self {
            starting_balance: 100.0,
            currency_symbol: "$".to_string(),
            currency_format: "{symbol}{amount}".to_string(),
            max_money: 1_000_000_000.0,
            min_money: 0.0,
            log_transactions: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarpsConfig {
    pub max_warps_per_player: usize,
    pub warp_cooldown: u64,
    pub per_warp_permissions: bool,
}

impl Default for WarpsConfig {
    fn default() -> Self {
        Self {
            max_warps_per_player: 10,
            warp_cooldown: 5,
            per_warp_permissions: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomesConfig {
    pub max_homes_default: usize,
    pub max_homes_vip: usize,
    pub home_cooldown: u64,
    pub home_on_death: bool,
}

impl Default for HomesConfig {
    fn default() -> Self {
        Self {
            max_homes_default: 3,
            max_homes_vip: 10,
            home_cooldown: 5,
            home_on_death: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TpaConfig {
    pub request_timeout: u64,
    pub teleport_cooldown: u64,
    pub cross_world: bool,
}

impl Default for TpaConfig {
    fn default() -> Self {
        Self {
            request_timeout: 30,
            teleport_cooldown: 5,
            cross_world: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KitsConfig {
    pub default_cooldown: u64,
    pub kit_signs: bool,
}

impl Default for KitsConfig {
    fn default() -> Self {
        Self {
            default_cooldown: 86400,
            kit_signs: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationConfig {
    pub ban_message: String,
    pub mute_message: String,
    pub jail_message: String,
    pub jail_location: Option<LocationConfig>,
    pub max_tempban_duration: u64,
    pub max_tempmute_duration: u64,
}

impl Default for ModerationConfig {
    fn default() -> Self {
        Self {
            ban_message: "You have been banned from this server.".to_string(),
            mute_message: "You have been muted.".to_string(),
            jail_message: "You have been jailed.".to_string(),
            jail_location: None,
            max_tempban_duration: 2592000,
            max_tempmute_duration: 2592000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationConfig {
    pub world: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for LocationConfig {
    fn default() -> Self {
        Self {
            world: "world".to_string(),
            x: 0.0,
            y: 64.0,
            z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamemodeConfig {
    pub default: String,
}

impl Default for GamemodeConfig {
    fn default() -> Self {
        Self {
            default: "survival".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionsConfig {
    pub enable_contexts: bool,
    pub default_group: String,
    pub inheritance: InheritanceConfig,
    pub tracks: Vec<TrackConfig>,
}

impl Default for PermissionsConfig {
    fn default() -> Self {
        Self {
            enable_contexts: true,
            default_group: "default".to_string(),
            inheritance: InheritanceConfig::default(),
            tracks: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritanceConfig {
    pub max_depth: usize,
    pub order: String,
}

impl Default for InheritanceConfig {
    fn default() -> Self {
        Self {
            max_depth: 10,
            order: "depth-first".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackConfig {
    pub name: String,
    pub groups: Vec<String>,
}

impl Default for TrackConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            groups: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnConfig {
    pub spawn_location: Option<LocationConfig>,
    pub respawn_at_home: bool,
    pub bed_priority: bool,
}

impl Default for SpawnConfig {
    fn default() -> Self {
        Self {
            spawn_location: None,
            respawn_at_home: false,
            bed_priority: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilityConfig {
    pub afk_timeout: u64,
    pub afk_kick_timeout: Option<u64>,
    pub motd: String,
    pub rules: Vec<String>,
    pub max_players: u32,
}

impl Default for UtilityConfig {
    fn default() -> Self {
        Self {
            afk_timeout: 300,
            afk_kick_timeout: None,
            motd: "Welcome to the server!".to_string(),
            rules: vec!["Be respectful".to_string(), "No cheating".to_string()],
            max_players: 20,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConfig {
    pub format: String,
    pub broadcast_format: String,
    pub private_message_format: String,
    pub chat_radius: Option<f64>,
    pub anti_spam: bool,
    pub spam_threshold: u32,
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            format: "&8[&7{group}&8] &r{prefix}&r{player}&r: &f{message}".to_string(),
            broadcast_format: "&4[Broadcast] &r{message}".to_string(),
            private_message_format: "&8[&bPM&8] &r{from}&r -> &r{to}&r: &f{message}".to_string(),
            chat_radius: None,
            anti_spam: true,
            spam_threshold: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventsConfig {
    pub player_join_message: Option<String>,
    pub player_leave_message: Option<String>,
    pub death_messages: bool,
    pub keep_inventory: bool,
    pub anti_build: bool,
    pub anti_spam: bool,
    pub spam_threshold: u32,
}

impl Default for EventsConfig {
    fn default() -> Self {
        Self {
            player_join_message: Some("&a{player} joined the game".to_string()),
            player_leave_message: Some("&c{player} left the game".to_string()),
            death_messages: true,
            keep_inventory: false,
            anti_build: true,
            anti_spam: true,
            spam_threshold: 5,
        }
    }
}

impl Config {
    /// Load configuration from file (config.yml/toml) and environment variables.
    /// File takes precedence over env vars, which take precedence over defaults.
    pub fn load(data_folder: Option<&str>) -> Self {
        let mut builder = ConfigRs::builder();
        
        // 1. Add file source if data_folder provided
        if let Some(folder) = data_folder {
            let config_path = std::path::Path::new(folder).join("config");
            // Try YAML first, then TOML
            if config_path.with_extension("yml").exists() {
                builder = builder.add_source(config::File::from(config_path.with_extension("yml")));
            } else if config_path.with_extension("yaml").exists() {
                builder = builder.add_source(config::File::from(config_path.with_extension("yaml")));
            } else if config_path.with_extension("toml").exists() {
                builder = builder.add_source(config::File::from(config_path.with_extension("toml")));
            }
        }
        
        // 2. Add environment variable source (PUC_ prefix) - overrides file
        builder = builder.add_source(config::Environment::with_prefix("PUC").separator("__"));
        
        let config_rs = builder.build().expect("Failed to build configuration");
        
        // Try to deserialize; fall back to defaults if no config set
        config_rs.try_deserialize().unwrap_or_else(|e| {
            tracing::warn!("Failed to deserialize config: {}, using defaults", e);
            Self::default()
        })
    }

    pub fn reload(&self, data_folder: Option<&str>) -> Self {
        Self::load(data_folder)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage: StorageConfig {
                backend: StorageBackendConfig::default(),
                sqlite: SqliteConfig {
                    path: "plugins/pumpkin-unified-core/data.db".to_string(),
                    max_connections: 10,
                },
                redis: RedisConfig {
                    url: "redis://127.0.0.1:6379".to_string(),
                    max_connections: 10,
                },
                mysql: MysqlConfig {
                    url: "mysql://user:pass@localhost/pumpkin".to_string(),
                    max_connections: 10,
                },
            },
            economy: EconomyConfig {
                starting_balance: 100.0,
                currency_symbol: "$".to_string(),
                currency_format: "{symbol}{amount}".to_string(),
                max_money: 1_000_000_000.0,
                min_money: 0.0,
                log_transactions: true,
            },
            warps: WarpsConfig {
                max_warps_per_player: 10,
                warp_cooldown: 5,
                per_warp_permissions: true,
            },
            homes: HomesConfig {
                max_homes_default: 3,
                max_homes_vip: 10,
                home_cooldown: 5,
                home_on_death: false,
            },
            tpa: TpaConfig {
                request_timeout: 30,
                teleport_cooldown: 5,
                cross_world: true,
            },
            kits: KitsConfig {
                default_cooldown: 86400,
                kit_signs: true,
            },
            moderation: ModerationConfig {
                ban_message: "You have been banned from this server.".to_string(),
                mute_message: "You have been muted.".to_string(),
                jail_message: "You have been jailed.".to_string(),
                jail_location: None,
                max_tempban_duration: 2592000,
                max_tempmute_duration: 2592000,
            },
            gamemode: GamemodeConfig {
                default: "survival".to_string(),
            },
            permissions: PermissionsConfig {
                enable_contexts: true,
                default_group: "default".to_string(),
                inheritance: InheritanceConfig {
                    max_depth: 10,
                    order: "depth-first".to_string(),
                },
                tracks: vec![],
            },
            spawn: SpawnConfig {
                spawn_location: None,
                respawn_at_home: false,
                bed_priority: true,
            },
            utility: UtilityConfig {
                afk_timeout: 300,
                afk_kick_timeout: None,
                motd: "Welcome to the server!".to_string(),
                rules: vec!["Be respectful".to_string(), "No cheating".to_string()],
                max_players: 20,
            },
            chat: ChatConfig {
                format: "&8[&7{group}&8] &r{prefix}&r{player}&r: &f{message}".to_string(),
                broadcast_format: "&4[Broadcast] &r{message}".to_string(),
                private_message_format: "&8[&bPM&8] &r{from}&r -> &r{to}&r: &f{message}".to_string(),
                chat_radius: None,
                anti_spam: true,
                spam_threshold: 5,
            },
            events: EventsConfig {
                player_join_message: Some("&a{player} joined the game".to_string()),
                player_leave_message: Some("&c{player} left the game".to_string()),
                death_messages: true,
                keep_inventory: false,
                anti_build: true,
                anti_spam: true,
                spam_threshold: 5,
            },
        }
    }
}