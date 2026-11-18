//! Utility module: assorted EssentialsX-style convenience commands.

use crate::storage::Storage;
use std::sync::Arc;

pub mod commands;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerData {
    pub uuid: String,
    pub name: String,
    pub last_seen: i64,
    pub afk: bool,
}

pub struct Utility {
    storage: Arc<Storage>,
    rules: String,
    motd: String,
}

impl Utility {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self {
            storage,
            rules: "Be respectful. No griefing. Have fun.".to_string(),
            motd: "Welcome to the server!".to_string(),
        }
    }

    pub fn rules(&self) -> &str {
        &self.rules
    }

    pub fn motd(&self) -> &str {
        &self.motd
    }

    pub fn set_afk(&self, uuid: &str, name: &str, afk: bool) {
        let data = PlayerData {
            uuid: uuid.to_string(),
            name: name.to_string(),
            last_seen: crate::utils::time::now_timestamp(),
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
}