//! Warps module: named teleport locations.

use crate::storage::Storage;
use std::sync::Arc;

pub mod commands;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Warp {
    pub name: String,
    pub world: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub created_by: String,
    pub created_at: i64,
}

pub struct Warps {
    storage: Arc<Storage>,
}

impl Warps {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    pub fn get(&self, name: &str) -> Option<Warp> {
        self.storage.get("warps", name)
    }

    pub fn set(&self, warp: &Warp) {
        self.storage.set("warps", &warp.name, warp);
    }

    pub fn delete(&self, name: &str) -> bool {
        if self.get(name).is_some() {
            self.storage.delete("warps", name);
            true
        } else {
            false
        }
    }

    pub fn list(&self) -> Vec<Warp> {
        self.storage.values("warps")
    }
}