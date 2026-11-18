use crate::utils::text::format_text;
use crate::utility::Utility;
use pumpkin_plugin_api::{
    command::{
        Arg, ArgumentType, Command, CommandError, CommandNode, CommandSender, ConsumedArgs,
        StringType,
    },
    commands::CommandHandler,
    Context, Server,
};
use std::sync::Arc;

fn arg_string(args: &ConsumedArgs, key: &str) -> Option<String> {
    match args.get_value(key) {
        Arg::Simple(s) | Arg::Msg(s) => Some(s),
        _ => None,
    }
}

fn string_arg(name: &str) -> CommandNode {
    CommandNode::argument(name, &ArgumentType::String(StringType::SingleWord))
}

fn mode_name(mode: pumpkin_plugin_api::common::GameMode) -> &'static str {
    use pumpkin_plugin_api::common::GameMode;
    match mode {
        GameMode::Survival => "survival",
        GameMode::Creative => "creative",
        GameMode::Adventure => "adventure",
        GameMode::Spectator => "spectator",
    }
}

fn require_player(
    sender: &CommandSender,
) -> Option<pumpkin_plugin_api::player::Player> {
    match sender.as_player() {
        Some(p) => Some(p),
        None => {
            sender.send_message(format_text("&cOnly players can use this command"));
            None
        }
    }
}

// ---------------- afk ----------------

pub struct AfkHandler {
    utility: Arc<Utility>,
}

impl CommandHandler for AfkHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        if let Some(player) = require_player(&sender) {
            let uuid = player.get_id().to_string();
            let name = player.get_name();
            let current = self.utility.get_player(&uuid).map(|p| p.afk).unwrap_or(false);
            let new_state = !current;
            self.utility.set_afk(&uuid, &name, new_state);
            sender.send_message(format_text(if new_state {
                "&eYou are now AFK"
            } else {
                "&aYou are no longer AFK"
            }));
        }
        Ok(1)
    }
}

// ---------------- rules ----------------

pub struct RulesHandler {
    utility: Arc<Utility>,
}

impl CommandHandler for RulesHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        sender.send_message(format_text("&6&lServer Rules"));
        sender.send_message(format_text(&format!(
            "&e{}",
            self.utility.rules()
        )));
        Ok(1)
    }
}

// ---------------- motd ----------------

pub struct MotdHandler {
    utility: Arc<Utility>,
}

impl CommandHandler for MotdHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        sender.send_message(format_text(&format!(
            "&6MOTD: &e{}",
            self.utility.motd()
        )));
        Ok(1)
    }
}

// ---------------- list ----------------

pub struct ListHandler;

impl CommandHandler for ListHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let players = server.get_all_players();
        sender.send_message(format_text(&format!(
            "&6Online players ({}/{}):",
            players.len(),
            server.get_max_players()
        )));
        let names: Vec<String> = players.iter().map(|p| p.get_name()).collect();
        sender.send_message(format_text(&format!("&e{}", names.join(", "))));
        Ok(1)
    }
}

// ---------------- whois ----------------

pub struct WhoisHandler;

impl CommandHandler for WhoisHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "target").unwrap_or_default();
        match server.get_player_by_name(&name) {
            Some(player) => {
                sender.send_message(format_text(&format!(
                    "&6Player: &f{}",
                    player.get_name()
                )));
                sender.send_message(format_text(&format!(
                    "&7UUID: &f{}",
                    player.get_id().to_string()
                )));
                sender.send_message(format_text(&format!(
                    "&7Gamemode: &f{}",
                    mode_name(player.get_gamemode())
                )));
                sender.send_message(format_text(&format!(
                    "&7Ping: &f{}ms",
                    player.get_ping()
                )));
                sender.send_message(format_text(&format!(
                    "&7Health: &f{}/{}",
                    player.get_health(),
                    player.get_max_health()
                )));
            }
            None => sender.send_message(format_text(&format!(
                "&cPlayer not found: {name}"
            ))),
        }
        Ok(1)
    }
}

// ---------------- speed ----------------

pub struct SpeedHandler;

impl CommandHandler for SpeedHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match require_player(&sender) {
            Some(p) => p,
            None => return Ok(1),
        };
        let speed: f32 = arg_string(&args, "speed")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1.0);
        let speed = speed.clamp(0.1, 10.0);
        player.set_walk_speed(speed);
        sender.send_message(format_text(&format!(
            "&aSet walk speed to &f{speed}"
        )));
        Ok(1)
    }
}

// ---------------- compass ----------------

pub struct CompassHandler;

impl CommandHandler for CompassHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match require_player(&sender) {
            Some(p) => p,
            None => return Ok(1),
        };
        let pos = player.get_position();
        sender.send_message(format_text(&format!(
            "&aPosition: &f{:.1}, {:.1}, {:.1}",
            pos.0, pos.1, pos.2
        )));
        Ok(1)
    }
}

// ---------------- top ----------------

pub struct TopHandler;

impl CommandHandler for TopHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let _player = match require_player(&sender) {
            Some(p) => p,
            None => return Ok(1),
        };
        sender.send_message(format_text(
            "&eTop is not yet implemented (requires block API)",
        ));
        Ok(1)
    }
}

// ---------------- seen ----------------

pub struct SeenHandler {
    utility: Arc<Utility>,
}

impl CommandHandler for SeenHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "target").unwrap_or_default();
        if server.get_player_by_name(&name).is_some() {
            sender.send_message(format_text(&format!(
                "&a{} is online",
                name
            )));
        } else if let Some(data) = self.utility.last_seen(&name) {
            sender.send_message(format_text(&format!(
                "&e{} &7was last seen {}",
                data.name,
                crate::utils::time::format_duration(
                    crate::utils::time::now_timestamp() - data.last_seen
                )
            )));
        } else {
            sender.send_message(format_text(&format!(
                "&cPlayer not found: {name}"
            )));
        }
        Ok(1)
    }
}

// ---------------- weather ----------------

pub struct WeatherHandler;

impl CommandHandler for WeatherHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match require_player(&sender) {
            Some(p) => p,
            None => return Ok(1),
        };
        let world = player.get_world();
        let mode = arg_string(&args, "mode").unwrap_or_else(|| "clear".to_string());
        match mode.to_lowercase().as_str() {
            "clear" | "sun" => {
                world.set_raining(false);
                world.set_thundering(false);
                sender.send_message(format_text("&aSet weather to clear"));
            }
            "rain" => {
                world.set_raining(true);
                world.set_thundering(false);
                sender.send_message(format_text("&aSet weather to rain"));
            }
            "thunder" | "storm" => {
                world.set_raining(true);
                world.set_thundering(true);
                sender.send_message(format_text("&aSet weather to thunder"));
            }
            _ => sender.send_message(format_text("&cUsage: /weather <clear|rain|thunder>")),
        }
        Ok(1)
    }
}

// ---------------- time ----------------

pub struct TimeHandler;

impl CommandHandler for TimeHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match require_player(&sender) {
            Some(p) => p,
            None => return Ok(1),
        };
        let world = player.get_world();
        let mode = arg_string(&args, "mode").unwrap_or_else(|| "day".to_string());
        let ticks: u64 = match mode.to_lowercase().as_str() {
            "day" => 1000,
            "noon" => 6000,
            "night" => 13000,
            "midnight" => 18000,
            other => match other.parse() {
                Ok(t) => t,
                Err(_) => {
                    sender.send_message(format_text(
                        "&cUsage: /time <day|night|noon|midnight|<ticks>>",
                    ));
                    return Ok(1);
                }
            },
        };
        world.set_time_of_day(ticks);
        sender.send_message(format_text(&format!(
            "&aSet time to &f{mode}"
        )));
        Ok(1)
    }
}

// ---------------- butcher / extinguish (stubs pending entity API) ----------------

pub struct ButcherHandler;

impl CommandHandler for ButcherHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        sender.send_message(format_text(
            "&eButcher is not yet implemented (requires entity API)",
        ));
        Ok(1)
    }
}

pub struct ExtinguishHandler;

impl CommandHandler for ExtinguishHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        sender.send_message(format_text(
            "&eExtinguish is not yet implemented (requires block API)",
        ));
        Ok(1)
    }
}

// ---------------- registration ----------------

pub fn register(utility: Arc<Utility>, context: &Context) {
    let afk = Command::new(&["afk".to_string()], "Toggle AFK")
        .execute(AfkHandler {
            utility: utility.clone(),
        });
    context.register_command(afk, "pumpkin-unified-core:essentials.afk");

    let rules = Command::new(&["rules".to_string()], "View server rules")
        .execute(RulesHandler {
            utility: utility.clone(),
        });
    context.register_command(rules, "pumpkin-unified-core:essentials.rules");

    let motd = Command::new(&["motd".to_string()], "View message of the day")
        .execute(MotdHandler {
            utility: utility.clone(),
        });
    context.register_command(motd, "pumpkin-unified-core:essentials.motd");

    let list = Command::new(&["list".to_string()], "List online players")
        .execute(ListHandler);
    context.register_command(list, "pumpkin-unified-core:essentials.list");

    let whois = Command::new(&["whois".to_string()], "View player info")
        .then(string_arg("target").execute(WhoisHandler));
    context.register_command(whois, "pumpkin-unified-core:essentials.whois");

    let speed = Command::new(&["speed".to_string()], "Set walk speed")
        .then(string_arg("speed").execute(SpeedHandler));
    context.register_command(speed, "pumpkin-unified-core:essentials.speed");

    let compass = Command::new(&["compass".to_string()], "Show position")
        .execute(CompassHandler);
    context.register_command(compass, "pumpkin-unified-core:essentials.compass");

    let top = Command::new(&["top".to_string()], "Teleport to top")
        .execute(TopHandler);
    context.register_command(top, "pumpkin-unified-core:essentials.top");

    let seen = Command::new(&["seen".to_string()], "Check last seen")
        .then(string_arg("target").execute(SeenHandler {
            utility: utility.clone(),
        }));
    context.register_command(seen, "pumpkin-unified-core:essentials.seen");

    let weather = Command::new(&["weather".to_string()], "Change weather")
        .then(string_arg("mode").execute(WeatherHandler));
    context.register_command(weather, "pumpkin-unified-core:essentials.weather");

    let time = Command::new(&["time".to_string()], "Change time")
        .then(string_arg("mode").execute(TimeHandler));
    context.register_command(time, "pumpkin-unified-core:essentials.time");

    let butcher = Command::new(&["butcher".to_string()], "Kill mobs")
        .execute(ButcherHandler);
    context.register_command(butcher, "pumpkin-unified-core:essentials.butcher");

    let extinguish = Command::new(&["extinguish".to_string()], "Extinguish fires")
        .execute(ExtinguishHandler);
    context.register_command(extinguish, "pumpkin-unified-core:essentials.extinguish");
}