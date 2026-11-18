//! Kits module: predefined item/kit bundles with cooldowns.

use crate::storage::Storage;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

pub mod commands;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Kit {
    pub name: String,
    pub items: Vec<String>,
    pub cooldown_seconds: i64,
    pub delay_seconds: i64,
    pub one_time: bool,
}

pub struct Kits {
    storage: Arc<Storage>,
    cooldowns: Mutex<HashMap<String, i64>>,
}

impl Kits {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self {
            storage,
            cooldowns: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, name: &str) -> Option<Kit> {
        self.storage.get("kits", name)
    }

    pub fn create(&self, kit: &Kit) {
        self.storage.set("kits", &kit.name, kit);
    }

    pub fn list(&self) -> Vec<Kit> {
        self.storage.values("kits")
    }

    fn cd_key(uuid: &str, kit: &str) -> String {
        format!("{uuid}:{kit}")
    }

    /// Returns remaining cooldown seconds (0 if ready).
    pub fn remaining_cooldown(&self, uuid: &str, kit: &str) -> i64 {
        let key = Self::cd_key(uuid, kit);
        if let Some(last) = self.cooldowns.lock().unwrap().get(&key) {
            let elapsed = crate::utils::time::now_timestamp() - last;
            let kit_data = self.get(kit);
            let cooldown = kit_data.map(|k| k.cooldown_seconds).unwrap_or(0);
            let remaining = cooldown - elapsed;
            if remaining > 0 { remaining } else { 0 }
        } else {
            0
        }
    }

    pub fn set_cooldown(&self, uuid: &str, kit: &str) {
        self.cooldowns
            .lock()
            .unwrap()
            .insert(Self::cd_key(uuid, kit), crate::utils::time::now_timestamp());
    }
}