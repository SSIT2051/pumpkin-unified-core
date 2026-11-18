//! Spawn module: shared world spawn teleport point.

use crate::config::SpawnConfig;
use crate::storage::Storage;
use std::sync::Arc;

pub mod commands;

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Spawn {
    pub world: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

pub struct SpawnModule {
    #[allow(dead_code)]
    storage: Arc<Storage>,
    config: SpawnConfig,
}

impl SpawnModule {
    pub fn new(storage: Arc<Storage>, config: SpawnConfig) -> Self {
        Self { storage, config }
    }

    pub fn respawn_at_home(&self) -> bool {
        self.config.respawn_at_home
    }

    pub fn bed_priority(&self) -> bool {
        self.config.bed_priority
    }

    pub fn spawn_location(&self) -> Option<&crate::config::LocationConfig> {
        self.config.spawn_location.as_ref()
    }

    pub fn config(&self) -> &SpawnConfig {
        &self.config
    }
}