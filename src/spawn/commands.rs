use crate::spawn::SpawnModule;
use crate::utils::text::format_text;
use pumpkin_plugin_api::{
    command::{
        ArgumentType, Command, CommandError, CommandNode, CommandSender, ConsumedArgs,
        StringType,
    },
    commands::CommandHandler,
    Context, Server,
};
use std::sync::Arc;

fn string_arg(name: &str) -> CommandNode {
    CommandNode::argument(name, &ArgumentType::String(StringType::SingleWord))
}

pub struct SpawnHandler {
    spawn: Arc<SpawnModule>,
}

impl CommandHandler for SpawnHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        // Teleport to the current world's spawn location.
        if let Some(player) = sender.as_player() {
            let world = player.get_world();
            let spawn = world.get_spawn_location();
            let pos = (spawn.pos.x as f64, spawn.pos.y as f64, spawn.pos.z as f64);
            player.teleport(pos, Some(spawn.yaw), Some(spawn.pitch), world);
            sender.send_message(format_text("&aTeleported to spawn"));
        } else {
            sender.send_message(format_text("&cOnly players can use /spawn"));
        }
        Ok(1)
    }
}

pub struct SetSpawnHandler {
    spawn: Arc<SpawnModule>,
}

impl CommandHandler for SetSpawnHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        sender.send_message(format_text(
            "&aSpawn point set &7(uses world spawn location)",
        ));
        Ok(1)
    }
}

// ---------------- spawnmob ----------------

pub struct SpawnMobHandler {
    spawn: Arc<SpawnModule>,
}

impl CommandHandler for SpawnMobHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /spawnmob"));
                return Ok(1);
            }
        };
        sender.send_message(format_text(
            "&eSpawnMob requires entity API integration",
        ));
        Ok(1)
    }
}

pub fn register(spawn: Arc<SpawnModule>, context: &Context) {
    let spawn_cmd = Command::new(&["spawn".to_string()], "Teleport to spawn")
        .execute(SpawnHandler {
            spawn: spawn.clone(),
        });
    context.register_command(spawn_cmd, "pumpkin-unified-core:essentials.spawn");

    let setspawn = Command::new(&["setspawn".to_string()], "Set the spawn point")
        .execute(SetSpawnHandler { spawn: spawn.clone() });
    context.register_command(setspawn, "pumpkin-unified-core:essentials.setspawn");

    // /spawnmob <type> [count]
    let spawnmob = Command::new(&["spawnmob".to_string()], "Spawn a mob")
        .then(string_arg("type"))
        .then(string_arg("count"))
        .execute(SpawnMobHandler { spawn: spawn.clone() });
    context.register_command(spawnmob, "pumpkin-unified-core:essentials.spawnmob");
}