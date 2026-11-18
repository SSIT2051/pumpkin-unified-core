//! Gamemode module: change player gamemodes.

use crate::utils::text::format_text;
use pumpkin_plugin_api::{
    command::{
        Arg, ArgumentType, Command, CommandError, CommandNode, CommandSender, ConsumedArgs,
        StringType,
    },
    commands::CommandHandler,
    common::GameMode,
    Context, Server,
};

fn arg_string(args: &ConsumedArgs, key: &str) -> Option<String> {
    match args.get_value(key) {
        Arg::Simple(s) | Arg::Msg(s) => Some(s),
        _ => None,
    }
}

fn string_arg(name: &str) -> CommandNode {
    CommandNode::argument(name, &ArgumentType::String(StringType::SingleWord))
}

fn mode_name(mode: GameMode) -> &'static str {
    match mode {
        GameMode::Survival => "survival",
        GameMode::Creative => "creative",
        GameMode::Adventure => "adventure",
        GameMode::Spectator => "spectator",
    }
}

/// Sets the gamemode of the sender (or `[target]`) to a fixed `mode`.
pub struct GamemodeHandler {
    mode: GameMode,
}

impl CommandHandler for GamemodeHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target_name = arg_string(&args, "target");
        let target = if let Some(name) = target_name {
            match server.get_player_by_name(&name) {
                Some(p) => p,
                None => {
                    sender.send_message(format_text(&format!(
                        "&cPlayer not found: {name}"
                    )));
                    return Ok(1);
                }
            }
        } else {
            match sender.as_player() {
                Some(p) => p,
                None => {
                    sender.send_message(format_text("&cSpecify a player from console"));
                    return Ok(1);
                }
            }
        };

        target.set_gamemode(self.mode);
        sender.send_message(format_text(&format!(
            "&aSet {}'s gamemode to &f{}",
            target.get_name(),
            mode_name(self.mode)
        )));
        Ok(1)
    }
}

/// Reports the sender's (or `[target]`'s) current gamemode.
pub struct GamemodeQueryHandler;

impl CommandHandler for GamemodeQueryHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target = if let Some(name) = arg_string(&args, "target") {
            match server.get_player_by_name(&name) {
                Some(p) => p,
                None => {
                    sender.send_message(format_text(&format!(
                        "&cPlayer not found: {name}"
                    )));
                    return Ok(1);
                }
            }
        } else {
            match sender.as_player() {
                Some(p) => p,
                None => {
                    sender.send_message(format_text("&cSpecify a player from console"));
                    return Ok(1);
                }
            }
        };
        sender.send_message(format_text(&format!(
            "&a{} 's gamemode is &f{}",
            target.get_name(),
            mode_name(target.get_gamemode())
        )));
        Ok(1)
    }
}

pub fn register(context: &Context) {
    let gamemode = Command::new(
        &["gamemode".to_string(), "gm".to_string()],
        "Change gamemode",
    )
    .then(
        CommandNode::literal("survival")
            .then(string_arg("target").execute(GamemodeHandler { mode: GameMode::Survival })),
    )
    .then(
        CommandNode::literal("creative")
            .then(string_arg("target").execute(GamemodeHandler { mode: GameMode::Creative })),
    )
    .then(
        CommandNode::literal("adventure")
            .then(string_arg("target").execute(GamemodeHandler { mode: GameMode::Adventure })),
    )
    .then(
        CommandNode::literal("spectator")
            .then(string_arg("target").execute(GamemodeHandler { mode: GameMode::Spectator })),
    )
    .then(string_arg("mode").execute(GamemodeQueryHandler));
    context.register_command(gamemode, "pumpkin-unified-core:essentials.gamemode");

    let alias = |name: &str, mode: GameMode| {
        let cmd = Command::new(&[name.to_string()], "Change gamemode")
            .then(string_arg("target").execute(GamemodeHandler { mode }));
        context.register_command(cmd, "pumpkin-unified-core:essentials.gamemode");
    };
    alias("gmc", GameMode::Creative);
    alias("gms", GameMode::Survival);
    alias("gma", GameMode::Adventure);
    alias("gmsp", GameMode::Spectator);
}