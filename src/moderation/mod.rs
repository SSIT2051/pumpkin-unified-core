//! Moderation module: bans, mutes, kicks, fly, god, heal, feed, vanish.

use crate::config::ModerationConfig;
use crate::storage::Storage;
use crate::utils::uuid_resolver::UuidResolver;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

pub mod commands;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Punishment {
    pub uuid: String,
    pub name: String,
    pub kind: String,
    pub reason: String,
    pub created_at: i64,
    pub expires_at: Option<i64>,
}

impl Punishment {
    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|e| crate::utils::time::now_timestamp() >= e).unwrap_or(false)
    }
}

pub struct Moderation {
    storage: Arc<Storage>,
    config: ModerationConfig,
    state: Mutex<HashMap<String, ModerationState>>,
    uuid_resolver: Arc<UuidResolver>,
}

#[derive(Debug, Clone, Default)]
pub struct ModerationState {
    pub flying: bool,
    pub god: bool,
    pub vanished: bool,
    pub jailed: bool,
    pub social_spy: bool,
    pub no_pain: bool,
    pub keep_inventory: bool,
}

impl Moderation {
    pub fn new(storage: Arc<Storage>, config: ModerationConfig) -> Self {
        let uuid_resolver = Arc::new(UuidResolver::new(storage.clone()));
        uuid_resolver.load_cache();
        Self {
            storage,
            config,
            state: Mutex::new(HashMap::new()),
            uuid_resolver,
        }
    }

    pub fn uuid_resolver(&self) -> Arc<UuidResolver> {
        self.uuid_resolver.clone()
    }

    fn key(uuid: &str) -> String {
        uuid.to_string()
    }

    pub fn set_punishment(&self, punishment: &Punishment) {
        self.storage.set(
            "punishments",
            &Self::key(&punishment.uuid),
            punishment,
        );
    }

    pub fn get_punishment(&self, uuid: &str) -> Option<Punishment> {
        self.storage.get("punishments", &Self::key(uuid))
    }

    /// Finds a punishment by player name (scanning stored records).
    pub fn get_punishment_by_name(&self, name: &str) -> Option<Punishment> {
        for key in self.storage.keys("punishments") {
            if let Some(p) = self.storage.get::<Punishment>("punishments", &key) {
                if p.name.eq_ignore_ascii_case(name) {
                    return Some(p);
                }
            }
        }
        None
    }

    pub fn remove_punishment(&self, uuid: &str) {
        self.storage.delete("punishments", &Self::key(uuid));
    }

    pub fn is_muted(&self, uuid: &str) -> bool {
        match self.get_punishment(uuid) {
            Some(p) => p.kind == "mute" && !p.is_expired(),
            None => false,
        }
    }

    pub fn state(&self, uuid: &str) -> ModerationState {
        self.state
            .lock()
            .unwrap()
            .get(uuid)
            .cloned()
            .unwrap_or_default()
    }

    pub fn set_flying(&self, uuid: &str, flying: bool) {
        self.state.lock().unwrap().entry(uuid.to_string()).or_default().flying = flying;
    }

    pub fn set_god(&self, uuid: &str, god: bool) {
        self.state.lock().unwrap().entry(uuid.to_string()).or_default().god = god;
    }

    pub fn set_vanished(&self, uuid: &str, vanished: bool) {
        self.state.lock().unwrap().entry(uuid.to_string()).or_default().vanished = vanished;
    }

    pub fn set_jailed(&self, uuid: &str, jailed: bool) {
        self.state.lock().unwrap().entry(uuid.to_string()).or_default().jailed = jailed;
    }

    pub fn set_social_spy(&self, uuid: &str, social_spy: bool) {
        self.state.lock().unwrap().entry(uuid.to_string()).or_default().social_spy = social_spy;
    }

    pub fn set_no_pain(&self, uuid: &str, no_pain: bool) {
        self.state.lock().unwrap().entry(uuid.to_string()).or_default().no_pain = no_pain;
    }

    pub fn set_keep_inventory(&self, uuid: &str, keep_inventory: bool) {
        self.state.lock().unwrap().entry(uuid.to_string()).or_default().keep_inventory = keep_inventory;
    }

    pub fn ban_message(&self) -> &str {
        &self.config.ban_message
    }

    pub fn mute_message(&self) -> &str {
        &self.config.mute_message
    }

    pub fn jail_message(&self) -> &str {
        &self.config.jail_message
    }

    pub fn max_tempban_duration(&self) -> u64 {
        self.config.max_tempban_duration
    }

    pub fn max_tempmute_duration(&self) -> u64 {
        self.config.max_tempmute_duration
    }

    pub fn is_god(&self, uuid: &str) -> bool {
        self.state(uuid).god
    }

    pub fn is_jailed(&self, uuid: &str) -> bool {
        self.state(uuid).jailed
    }

    pub fn is_social_spy(&self, uuid: &str) -> bool {
        self.state(uuid).social_spy
    }

    pub fn is_no_pain(&self, uuid: &str) -> bool {
        self.state(uuid).no_pain
    }

    pub fn is_keep_inventory(&self, uuid: &str) -> bool {
        self.state(uuid).keep_inventory
    }
}