//! Teleportation module: tp, back, tphere, tppos, spawn

use crate::utils::uuid_resolver::ResolvedPlayer;
use crate::utils::Position;
use pumpkin_plugin_api::{
    command::{ArgumentType, Command, CommandError, CommandNode, CommandSender, ConsumedArgs, StringType},
    commands::CommandHandler,
    Context, Server,
};
use std::sync::Arc;

fn string_arg(name: &str) -> CommandNode {
    CommandNode::argument(name, &ArgumentType::String(StringType::SingleWord))
}

// ... teleportation command handlers would go here

pub fn register(utility: Arc<crate::utility::Utility>, context: &Context) {
    // Teleportation commands will be registered here
}