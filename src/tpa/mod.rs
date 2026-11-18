//! TPA module: teleport request (tpa) system.

use crate::config::TpaConfig;
use crate::storage::Storage;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

pub mod commands;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TpaRequest {
    pub requester: String,
    pub requester_uuid: String,
    pub target: String,
    pub target_uuid: String,
    pub here: bool,
    pub created_at: i64,
    pub expires_at: i64,
}

pub struct Tpa {
    #[allow(dead_code)]
    storage: Arc<Storage>,
    requests: Mutex<HashMap<String, TpaRequest>>,
    enabled: Mutex<HashMap<String, bool>>,
    config: TpaConfig,
}

impl Tpa {
    pub fn new(storage: Arc<Storage>, config: TpaConfig) -> Self {
        Self {
            storage,
            requests: Mutex::new(HashMap::new()),
            enabled: Mutex::new(HashMap::new()),
            config,
        }
    }

    pub fn is_enabled(&self, uuid: &str) -> bool {
        *self.enabled.lock().unwrap().get(uuid).unwrap_or(&true)
    }

    pub fn toggle(&self, uuid: &str) -> bool {
        let mut enabled = self.enabled.lock().unwrap();
        let current = *enabled.get(uuid).unwrap_or(&true);
        let new_value = !current;
        enabled.insert(uuid.to_string(), new_value);
        new_value
    }

    pub fn send_request(
        &self,
        requester: String,
        requester_uuid: String,
        target: String,
        target_uuid: String,
        here: bool,
    ) {
        let now = crate::utils::time::now_timestamp();
        let request = TpaRequest {
            requester,
            requester_uuid,
            target: target.clone(),
            target_uuid: target_uuid.clone(),
            here,
            created_at: now,
            expires_at: now + self.config.request_timeout as i64,
        };
        self.requests
            .lock()
            .unwrap()
            .insert(target_uuid, request);
    }

    pub fn accept(&self, target_uuid: &str) -> Option<TpaRequest> {
        self.requests.lock().unwrap().remove(target_uuid)
    }

    pub fn deny(&self, target_uuid: &str) -> Option<TpaRequest> {
        self.requests.lock().unwrap().remove(target_uuid)
    }

    pub fn request_timeout(&self) -> u64 {
        self.config.request_timeout
    }

    pub fn teleport_cooldown(&self) -> u64 {
        self.config.teleport_cooldown
    }

    pub fn get_cooldown_remaining(&self, _uuid: &str) -> Option<u64> {
        // Check if player has recent TPA teleport
        // This would need a cooldown tracking system
        None
    }

    pub fn cleanup_player_requests(&self, uuid: &str) {
        let mut requests = self.requests.lock().unwrap();
        requests.retain(|_, req| req.target_uuid != uuid && req.requester_uuid != uuid);
    }
}