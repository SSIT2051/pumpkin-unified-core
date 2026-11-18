//! Homes module: per-player personal teleport locations.

use crate::storage::Storage;
use std::sync::Arc;

pub mod commands;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Home {
    pub owner: String,
    pub name: String,
    pub world: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

pub struct Homes {
    storage: Arc<Storage>,
    max_homes: usize,
}

impl Homes {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self {
            storage,
            max_homes: 3,
        }
    }

    fn key(owner: &str, name: &str) -> String {
        format!("{owner}:{name}")
    }

    pub fn get(&self, owner: &str, name: &str) -> Option<Home> {
        self.storage.get("homes", &Self::key(owner, name))
    }

    pub fn set(&self, home: &Home) {
        self.storage.set("homes", &Self::key(&home.owner, &home.name), home);
    }

    pub fn delete(&self, owner: &str, name: &str) -> bool {
        let key = Self::key(owner, name);
        if self.storage.get::<Home>("homes", &key).is_some() {
            self.storage.delete("homes", &key);
            true
        } else {
            false
        }
    }

    pub fn list(&self, owner: &str) -> Vec<Home> {
        self.storage
            .values::<Home>("homes")
            .into_iter()
            .filter(|h| h.owner == owner)
            .collect()
    }

    pub fn count(&self, owner: &str) -> usize {
        self.list(owner).len()
    }

    pub fn max_homes(&self) -> usize {
        self.max_homes
    }
}