//! Spawn module: shared world spawn teleport point.

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
}

impl SpawnModule {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }
}