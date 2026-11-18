//! Utility module: assorted EssentialsX-style convenience commands.

use crate::config::UtilityConfig;
use crate::storage::Storage;
use std::sync::Arc;

pub mod commands;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerData {
    pub uuid: String,
    pub name: String,
    pub last_seen: i64,
    pub last_death: Option<crate::utils::Position>,
    pub last_location: Option<crate::utils::Position>,
    pub afk: bool,
}

pub struct Utility {
    storage: Arc<Storage>,
    config: UtilityConfig,
}

impl Utility {
    pub fn new(storage: Arc<Storage>, config: UtilityConfig) -> Self {
        Self {
            storage,
            config,
        }
    }

    pub fn rules(&self) -> String {
        self.config.rules.join("\n")
    }

    pub fn motd(&self) -> &str {
        &self.config.motd
    }

    pub fn set_afk(&self, uuid: &str, name: &str, afk: bool) {
        let data = PlayerData {
            uuid: uuid.to_string(),
            name: name.to_string(),
            last_seen: crate::utils::time::now_timestamp(),
            last_death: None,
            last_location: None,
            afk,
        };
        self.storage.set("players", uuid, &data);
    }

    pub fn get_player(&self, uuid: &str) -> Option<PlayerData> {
        self.storage.get("players", uuid)
    }

    pub fn last_seen(&self, name: &str) -> Option<PlayerData> {
        for key in self.storage.keys("players") {
            if let Some(p) = self.storage.get::<PlayerData>("players", &key) {
                if p.name.eq_ignore_ascii_case(name) {
                    return Some(p);
                }
            }
        }
        None
    }

    pub fn set_last_seen(&self, uuid: &str, _name: &str) {
        if let Some(mut data) = self.get_player(uuid) {
            data.last_seen = crate::utils::time::now_timestamp();
            self.storage.set("players", uuid, &data);
        }
    }

    pub fn update_last_seen(&self, uuid: &str, name: &str) {
        if let Some(mut data) = self.get_player(uuid) {
            data.last_seen = crate::utils::time::now_timestamp();
            data.name = name.to_string();
            self.storage.set("players", uuid, &data);
        } else {
            let data = PlayerData {
                uuid: uuid.to_string(),
                name: name.to_string(),
                last_seen: crate::utils::time::now_timestamp(),
                last_death: None,
                last_location: None,
                afk: false,
            };
            self.storage.set("players", uuid, &data);
        }
    }

    pub fn set_last_death(&self, uuid: &str, pos: crate::utils::Position) {
        if let Some(mut data) = self.get_player(uuid) {
            data.last_death = Some(pos);
            self.storage.set("players", uuid, &data);
        }
    }

    pub fn set_last_location(&self, uuid: &str, pos: crate::utils::Position) {
        if let Some(mut data) = self.get_player(uuid) {
            data.last_location = Some(pos);
            self.storage.set("players", uuid, &data);
        }
    }

    pub fn afk_timeout(&self) -> u64 {
        self.config.afk_timeout
    }

    pub fn afk_kick_timeout(&self) -> Option<u64> {
        self.config.afk_kick_timeout
    }

    pub fn max_players(&self) -> u32 {
        self.config.max_players
    }

    pub fn update_activity(&self, uuid: &str) {
        if let Some(mut data) = self.get_player(uuid) {
            data.last_seen = crate::utils::time::now_timestamp();
            self.storage.set("players", uuid, &data);
        }
    }
}