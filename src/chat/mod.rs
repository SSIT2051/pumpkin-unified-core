use crate::utils::macros::*;
use crate::utils::text::format_text;
use pumpkin_plugin_api::{
    command::{Command, CommandSender},
    context::Context,
    server::Server,
};
use std::sync::Arc;

pub struct ChatModule {
    config: Arc<crate::config::ChatConfig>,
}

impl ChatModule {
    pub fn new() -> Self {
        let config = Arc::new(crate::config::Config::default().chat);
        Self { config }
    }

    pub fn format_chat(&self, player: &pumpkin_plugin_api::player::Player, message: &str) -> String {
        let group = "default";
        let prefix = "";
        let suffix = "";
        
        self.config.format
            .replace("{player}", player.get_name())
            .replace("{message}", message)
            .replace("{group}", group)
            .replace("{prefix}", prefix)
            .replace("{suffix}", suffix)
    }

    pub fn register_commands(&self, context: &mut Context) {
        crate::chat::commands::register(Arc::new(self.clone()), context);
    }
}

impl Clone for ChatModule {
    fn clone(&self) -> Self {
        Self { config: self.config.clone() }
    }
}