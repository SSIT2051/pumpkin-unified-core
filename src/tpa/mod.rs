//! TPA module: teleport request (tpa) system.

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
    timeout_seconds: i64,
    enabled: Mutex<HashMap<String, bool>>,
}

impl Tpa {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self {
            storage,
            requests: Mutex::new(HashMap::new()),
            timeout_seconds: 60,
            enabled: Mutex::new(HashMap::new()),
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
            expires_at: now + self.timeout_seconds,
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
}