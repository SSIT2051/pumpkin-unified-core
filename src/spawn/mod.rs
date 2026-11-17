use crate::storage::StorageBackend;
use crate::utils::macros::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnData {
    pub world: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

pub struct SpawnModule {
    storage: Arc<StorageBackend>,
    spawn: Option<SpawnData>,
    config: Arc<crate::config::SpawnConfig>,
}

impl SpawnModule {
    pub fn new(storage: Arc<StorageBackend>) -> Self {
        let config = Arc::new(crate::config::Config::default().spawn);
        Self {
            storage,
            spawn: None,
            config,
        }
    }

    pub async fn load_spawn(&mut self) -> Result<()> {
        if let Some(data) = self.storage.get("spawn", "main").await? {
            let spawn_data: crate::storage::SerializedData<SpawnData> = crate::storage::SerializedData::deserialize(&data)?;
            self.spawn = Some(spawn_data.data);
        } else if let Some(loc) = &self.config.spawn_location {
            self.spawn = Some(SpawnData {
                world: loc.world.clone(),
                x: loc.x,
                y: loc.y,
                z: loc.z,
                yaw: loc.yaw,
                pitch: loc.pitch,
            });
        }
        Ok(())
    }

    pub async fn set_spawn(&mut self, spawn: SpawnData) -> Result<()> {
        let data = crate::storage::SerializedData::new(spawn.clone()).serialize()?;
        self.storage.set("spawn", "main", &data).await?;
        self.spawn = Some(spawn);
        Ok(())
    }

    pub fn get_spawn(&self) -> Option<SpawnData> {
        self.spawn.clone()
    }

    pub fn register_commands(&self, context: &mut pumpkin_plugin_api::context::Context) {
        crate::spawn::commands::register(self.clone(), context);
    }
}