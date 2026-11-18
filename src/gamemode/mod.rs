//! Gamemode module.

use crate::config::GamemodeConfig;

pub mod commands;

pub struct Gamemode {
    config: GamemodeConfig,
}

impl Gamemode {
    pub fn new(config: GamemodeConfig) -> Self {
        Self { config }
    }

    pub fn default_gamemode(&self) -> &str {
        &self.config.default
    }
}