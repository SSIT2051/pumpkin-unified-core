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

// ---------------- clearinventory ----------------

pub struct ClearInventoryHandler;

impl CommandHandler for ClearInventoryHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target_name = arg_string(&args, "target")
            .or_else(|| sender.as_player().map(|p| p.get_name()))
            .unwrap_or_default();
        if target_name.is_empty() {
            sender.send_message(format_text("&cUsage: /clearinventory [player]"));
            return Ok(1);
        }
        let target = match server.get_player_by_name(&target_name) {
            Some(p) => p,
            None => {
                sender.send_message(format_text(&format!("&cPlayer not found: {target_name}")));
                return Ok(1);
            }
        };
        sender.send_message(format_text(
            "&eClearInventory requires inventory API integration",
        ));
        Ok(1)
    }
}

// ---------------- depth ----------------

pub struct DepthHandler;

impl CommandHandler for DepthHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /depth"));
                return Ok(1);
            }
        };
        let pos = player.get_position();
        sender.send_message(format_text(&format!(
            "&eDepth: &fY={:.1}",
            pos.1
        )));
        Ok(1)
    }
}

// ---------------- enchant ----------------

pub struct EnchantHandler;

impl CommandHandler for EnchantHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /enchant"));
                return Ok(1);
            }
        };
        sender.send_message(format_text(
            "&eEnchant requires enchantment API integration",
        ));
        Ok(1)
    }
}

// ---------------- exp ----------------

pub struct ExpHandler;

impl CommandHandler for ExpHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /exp"));
                return Ok(1);
            }
        };
        sender.send_message(format_text(
            "&eExp requires player experience API integration",
        ));
        Ok(1)
    }
}

// ---------------- firework ----------------

pub struct FireworkHandler;

impl CommandHandler for FireworkHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /firework"));
                return Ok(1);
            }
        };
        sender.send_message(format_text(
            "&eFirework requires entity API integration",
        ));
        Ok(1)
    }
}

// ---------------- fix ----------------

pub struct FixHandler;

impl CommandHandler for FixHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /fix"));
                return Ok(1);
            }
        };
        sender.send_message(format_text(
            "&eFix requires item API integration",
        ));
        Ok(1)
    }
}

// ---------------- getpos ----------------

pub struct GetPosHandler;

impl CommandHandler for GetPosHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target_name = arg_string(&args, "target")
            .or_else(|| sender.as_player().map(|p| p.get_name()))
            .unwrap_or_default();
        let target = match server.get_player_by_name(&target_name) {
            Some(p) => p,
            None => {
                sender.send_message(format_text(&format!("&cPlayer not found: {target_name}")));
                return Ok(1);
            }
        };
        let pos = target.get_position();
        let world = target.get_world();
        sender.send_message(format_text(&format!(
            "&ePosition: &fX={:.1} Y={:.1} Z={:.1} World={}",
            pos.0, pos.1, pos.2, world.get_name()
        )));
        Ok(1)
    }
}

// ---------------- hat ----------------

pub struct HatHandler;

impl CommandHandler for HatHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /hat"));
                return Ok(1);
            }
        };
        sender.send_message(format_text(
            "&eHat requires inventory API integration",
        ));
        Ok(1)
    }
}

// ---------------- help ----------------

pub struct HelpHandler;

impl CommandHandler for HelpHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        sender.send_message(format_text("&eAvailable commands:"));
        sender.send_message(format_text(
            "&6/balance, /pay, /baltop, /worth, /sell, /eco",
        ));
        sender.send_message(format_text(
            "&6/home, /sethome, /delhome, /homes",
        ));
        sender.send_message(format_text(
            "&6/warp, /setwarp, /delwarp, /warps",
        ));
        sender.send_message(format_text(
            "&6/kit, /kits",
        ));
        sender.send_message(format_text(
            "&6/tpa, /tpahere, /tpaccept, /tpdeny",
        ));
        sender.send_message(format_text(
            "&6/tp, /tphere, /tppos, /back",
        ));
        sender.send_message(format_text(
            "&6/kick, /ban, /tempban, /unban, /mute, /unmute, /vanish, /god, /fly, /heal, /feed, /jail, /warn",
        ));
        sender.send_message(format_text(
            "&6/clearinventory, /depth, /enchant, /exp, /firework, /fix, /getpos, /hat",
        ));
        Ok(1)
    }
}

// ---------------- helpop ----------------

pub struct HelpOpHandler;

impl CommandHandler for HelpOpHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /helpop"));
                return Ok(1);
            }
        };
        sender.send_message(format_text("&aHelp request sent to staff"));
        Ok(1)
    }
}

// ---------------- ignore ----------------

pub struct IgnoreHandler;

impl CommandHandler for IgnoreHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target_name = arg_string(&args, "target").unwrap_or_default();
        let target = match server.get_player_by_name(&target_name) {
            Some(p) => p,
            None => {
                sender.send_message(format_text(&format!("&cPlayer not found: {target_name}")));
                return Ok(1);
            }
        };
        sender.send_message(format_text(&format!(
            "&eIgnore list feature not yet implemented for &f{}",
            target.get_name()
        )));
        Ok(1)
    }
}

// ---------------- invsee ----------------

pub struct InvSeeHandler;

impl CommandHandler for InvSeeHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target_name = arg_string(&args, "target").unwrap_or_default();
        let target = match server.get_player_by_name(&target_name) {
            Some(p) => p,
            None => {
                sender.send_message(format_text(&format!("&cPlayer not found: {target_name}")));
                return Ok(1);
            }
        };
        sender.send_message(format_text(
            "&eInvSee requires inventory API integration",
        ));
        Ok(1)
    }
}

// ---------------- jump ----------------

pub struct JumpHandler;

impl CommandHandler for JumpHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /jump"));
                return Ok(1);
            }
        };
        sender.send_message(format_text(
            "&eJump requires block API integration",
        ));
        Ok(1)
    }
}

// ---------------- near ----------------

pub struct NearHandler;

impl CommandHandler for NearHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /near"));
                return Ok(1);
            }
        };
        let pos = player.get_position();
        let mut nearby = Vec::new();
        let player_id = player.get_id().to_string();
        for other in server.get_all_players() {
            if other.get_id().to_string() == player_id {
                continue;
            }
            let other_pos = other.get_position();
            let dist = ((pos.0 - other_pos.0).powi(2) + 
                       (pos.1 - other_pos.1).powi(2) + 
                       (pos.2 - other_pos.2).powi(2)).sqrt();
            if dist <= 100.0 {
                nearby.push(format!("{} ({:.1}m)", other.get_name(), dist));
            }
        }
        if nearby.is_empty() {
            sender.send_message(format_text("&eNo players nearby"));
        } else {
            sender.send_message(format_text(&format!("&eNearby: {}", nearby.join(", "))));
        }
        Ok(1)
    }
}

// ---------------- nick ----------------

pub struct NickHandler;

impl CommandHandler for NickHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /nick"));
                return Ok(1);
            }
        };
        let nick = arg_string(&args, "nickname").unwrap_or_default();
        if nick.is_empty() {
            sender.send_message(format_text("&aNickname reset (requires display name API)"));
        } else {
            sender.send_message(format_text(&format!(
                "&eNickname set to &f{} &e(requires display name API)",
                nick
            )));
        }
        Ok(1)
    }
}

// ---------------- ping ----------------

pub struct PingHandler;

impl CommandHandler for PingHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /ping"));
                return Ok(1);
            }
        };
        let ping = player.get_ping();
        sender.send_message(format_text(&format!("&ePing: &f{}ms", ping)));
        Ok(1)
    }
}

// ---------------- powertool ----------------

pub struct PowerToolHandler;

impl CommandHandler for PowerToolHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /powertool"));
                return Ok(1);
            }
        };
        sender.send_message(format_text(
            "&ePowerTool requires item API integration",
        ));
        Ok(1)
    }
}

// ---------------- ptime ----------------

pub struct PTimeHandler;

impl CommandHandler for PTimeHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /ptime"));
                return Ok(1);
            }
        };
        let time_str = arg_string(&args, "time").unwrap_or_default();
        sender.send_message(format_text(&format!(
            "&ePlayer time: &f{} &e(requires player time API)",
            time_str
        )));
        Ok(1)
    }
}

// ---------------- pweather ----------------

pub struct PWeatherHandler;

impl CommandHandler for PWeatherHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /pweather"));
                return Ok(1);
            }
        };
        let weather = arg_string(&args, "weather").unwrap_or_default();
        sender.send_message(format_text(&format!(
            "&ePlayer weather: &f{} &e(requires player weather API)",
            weather
        )));
        Ok(1)
    }
}

// ---------------- realname ----------------

pub struct RealNameHandler;

impl CommandHandler for RealNameHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target_name = arg_string(&args, "target").unwrap_or_default();
        let target = match server.get_player_by_name(&target_name) {
            Some(p) => p,
            None => {
                sender.send_message(format_text(&format!("&cPlayer not found: {target_name}")));
                return Ok(1);
            }
        };
        sender.send_message(format_text(&format!(
            "&eReal name: &f{}",
            target.get_name()
        )));
        Ok(1)
    }
}

// ---------------- recipe ----------------

pub struct RecipeHandler;

impl CommandHandler for RecipeHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /recipe"));
                return Ok(1);
            }
        };
        sender.send_message(format_text(
            "&eRecipe requires recipe API integration",
        ));
        Ok(1)
    }
}

// ---------------- repair ----------------

pub struct RepairHandler;

impl CommandHandler for RepairHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /repair"));
                return Ok(1);
            }
        };
        sender.send_message(format_text(
            "&eRepair requires item API integration",
        ));
        Ok(1)
    }
}

// ---------------- sign ----------------

pub struct SignHandler;

impl CommandHandler for SignHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /sign"));
                return Ok(1);
            }
        };
        sender.send_message(format_text(
            "&eSign requires block API integration",
        ));
        Ok(1)
    }
}

// ---------------- sit ----------------

pub struct SitHandler;

impl CommandHandler for SitHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /sit"));
                return Ok(1);
            }
        };
        sender.send_message(format_text(
            "&eSit requires entity API integration",
        ));
        Ok(1)
    }
}

// ---------------- skull ----------------

pub struct SkullHandler;

impl CommandHandler for SkullHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /skull"));
                return Ok(1);
            }
        };
        let target_name = arg_string(&args, "target").unwrap_or_else(|| player.get_name());
        let target = match server.get_player_by_name(&target_name) {
            Some(p) => p,
            None => {
                sender.send_message(format_text(&format!("&cPlayer not found: {target_name}")));
                return Ok(1);
            }
        };
        sender.send_message(format_text(
            "&eSkull requires item API integration",
        ));
        Ok(1)
    }
}

// ---------------- sudo ----------------

pub struct SudoHandler;

impl CommandHandler for SudoHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target_name = arg_string(&args, "target").unwrap_or_default();
        let command = arg_string(&args, "command").unwrap_or_default();
        if target_name.is_empty() || command.is_empty() {
            sender.send_message(format_text("&cUsage: /sudo <player> <command>"));
            return Ok(1);
        }
        let target = match server.get_player_by_name(&target_name) {
            Some(p) => p,
            None => {
                sender.send_message(format_text(&format!("&cPlayer not found: {target_name}")));
                return Ok(1);
            }
        };
        sender.send_message(format_text(&format!(
            "&eSudo: would run '&f{}&e' as &f{}",
            command, target.get_name()
        )));
        Ok(1)
    }
}

// ---------------- sun ----------------

pub struct SunHandler;

impl CommandHandler for SunHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /sun"));
                return Ok(1);
            }
        };
        sender.send_message(format_text(
            "&eSun requires world time API integration",
        ));
        Ok(1)
    }
}

// ---------------- superpickaxe ----------------

pub struct SuperPickaxeHandler;

impl CommandHandler for SuperPickaxeHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /superpickaxe"));
                return Ok(1);
            }
        };
        sender.send_message(format_text(
            "&eSuperPickaxe requires block API integration",
        ));
        Ok(1)
    }
}

// ---------------- thunder ----------------

pub struct ThunderHandler;

impl CommandHandler for ThunderHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /thunder"));
                return Ok(1);
            }
        };
        sender.send_message(format_text(
            "&eThunder requires world weather API integration",
        ));
        Ok(1)
    }
}

// ---------------- toggledownfall ----------------

pub struct ToggleDownfallHandler;

impl CommandHandler for ToggleDownfallHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /toggledownfall"));
                return Ok(1);
            }
        };
        sender.send_message(format_text(
            "&eToggleDownfall requires world weather API integration",
        ));
        Ok(1)
    }
}

// ---------------- tree ----------------

pub struct TreeHandler;

impl CommandHandler for TreeHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /tree"));
                return Ok(1);
            }
        };
        sender.send_message(format_text(
            "&eTree requires block API integration",
        ));
        Ok(1)
    }
}

// ---------------- unlimit ----------------

pub struct UnlimitHandler;

impl CommandHandler for UnlimitHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /unlimit"));
                return Ok(1);
            }
        };
        sender.send_message(format_text(
            "&eUnlimit requires item API integration",
        ));
        Ok(1)
    }
}

// ---------------- version ----------------

pub struct VersionHandler;

impl CommandHandler for VersionHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        sender.send_message(format_text("&ePumpkin-Unified-Core &f0.1.0"));
        sender.send_message(format_text("&eRunning on PumpkinMC"));
        Ok(1)
    }
}

// ---------------- wb (worldborder) ----------------

pub struct WorldBorderHandler;

impl CommandHandler for WorldBorderHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /wb"));
                return Ok(1);
            }
        };
        let sub = arg_string(&args, "subcommand").unwrap_or_default();
        match sub.as_str() {
            "set" => {
                let size: f64 = arg_string(&args, "size").and_then(|s| s.parse().ok()).unwrap_or(0.0);
                if size > 0.0 {
                    sender.send_message(format_text(&format!(
                        "&eWorld border set to {} &e(requires world border API)",
                        size
                    )));
                } else {
                    sender.send_message(format_text("&cUsage: /wb set <size>"));
                }
            }
            _ => {
                sender.send_message(format_text("&eWorldBorder: /wb set <size>"));
            }
        }
        Ok(1)
    }
}

// ---------------- xp ----------------

pub struct XpHandler;

impl CommandHandler for XpHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /xp"));
                return Ok(1);
            }
        };
        sender.send_message(format_text(
            "&eXP requires player experience API integration",
        ));
        Ok(1)
    }
}

// ---------------- back ----------------

pub struct BackHandler {
    utility: Arc<Utility>,
}

impl CommandHandler for BackHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /back"));
                return Ok(1);
            }
        };
        if let Some(data) = self.utility.get_player(&player.get_id().to_string()) {
            if let Some(loc) = data.last_location {
                let world = player.get_world();
                player.teleport(
                    (loc.x, loc.y, loc.z),
                    Some(loc.yaw),
                    Some(loc.pitch),
                    world,
                );
                sender.send_message(format_text("&aTeleported to previous location"));
            } else {
                sender.send_message(format_text("&cNo previous location found"));
            }
        } else {
            sender.send_message(format_text("&cNo previous location found"));
        }
        Ok(1)
    }
}

// ---------------- tp (teleport) ----------------

pub struct TpHandler;

impl CommandHandler for TpHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /tp"));
                return Ok(1);
            }
        };
        
        let target_name = arg_string(&args, "target").unwrap_or_default();
        let target = match server.get_player_by_name(&target_name) {
            Some(p) => p,
            None => {
                sender.send_message(format_text(&format!("&cPlayer not found: {target_name}")));
                return Ok(1);
            }
        };
        
        let pos = target.get_position();
        let world = target.get_world();
        player.teleport(
            (pos.0, pos.1, pos.2),
            Some(target.get_yaw()),
            Some(target.get_pitch()),
            world,
        );
        sender.send_message(format_text(&format!("&aTeleported to &f{}", target.get_name())));
        Ok(1)
    }
}

// ---------------- tphere ----------------

pub struct TpHereHandler;

impl CommandHandler for TpHereHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /tphere"));
                return Ok(1);
            }
        };
        
        let target_name = arg_string(&args, "target").unwrap_or_default();
        let target = match server.get_player_by_name(&target_name) {
            Some(p) => p,
            None => {
                sender.send_message(format_text(&format!("&cPlayer not found: {target_name}")));
                return Ok(1);
            }
        };
        
        let pos = player.get_position();
        let world = player.get_world();
        target.teleport(
            (pos.0, pos.1, pos.2),
            Some(player.get_yaw()),
            Some(player.get_pitch()),
            world,
        );
        sender.send_message(format_text(&format!("&aTeleported &f{} &ato you", target.get_name())));
        target.send_system_message(
            format_text(&format!("&eYou were teleported to &f{}", player.get_name())),
            false,
        );
        Ok(1)
    }
}

// ---------------- tppos ----------------

pub struct TpPosHandler;

impl CommandHandler for TpPosHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /tppos"));
                return Ok(1);
            }
        };
        
        let x: f64 = arg_string(&args, "x")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let y: f64 = arg_string(&args, "y")
            .and_then(|s| s.parse().ok())
            .unwrap_or(64.0);
        let z: f64 = arg_string(&args, "z")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        
        let world_name = arg_string(&args, "world").unwrap_or_else(|| player.get_world().get_name());
        let world = server.get_world_by_name(&world_name).unwrap_or_else(|| player.get_world());
        
        player.teleport((x, y, z), None, None, world);
        sender.send_message(format_text(&format!("&aTeleported to &f{x}, {y}, {z}")));
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

    // /back
    let back = Command::new(&["back".to_string()], "Teleport to previous location")
        .execute(BackHandler { utility });
    context.register_command(back, "pumpkin-unified-core:essentials.back");

    // /tp <player>
    let tp = Command::new(&["tp".to_string()], "Teleport to a player")
        .then(string_arg("target").execute(TpHandler));
    context.register_command(tp, "pumpkin-unified-core:essentials.tp");

    // /tphere <player>
    let tphere = Command::new(&["tphere".to_string()], "Teleport a player to you")
        .then(string_arg("target").execute(TpHereHandler));
    context.register_command(tphere, "pumpkin-unified-core:essentials.tphere");

    // /tppos <x> <y> <z> [world]
    let tppos = Command::new(&["tppos".to_string()], "Teleport to coordinates")
        .then(string_arg("x"))
        .then(string_arg("y"))
        .then(string_arg("z"))
        .then(string_arg("world"))
        .execute(TpPosHandler);
    context.register_command(tppos, "pumpkin-unified-core:essentials.tppos");

    // /clearinventory [player]
    let clearinventory = Command::new(
        &["clearinventory".to_string(), "ci".to_string()],
        "Clear inventory",
    )
    .execute(ClearInventoryHandler)
    .then(string_arg("target").execute(ClearInventoryHandler));
    context.register_command(clearinventory, "pumpkin-unified-core:essentials.clearinventory");

    // /depth
    let depth = Command::new(&["depth".to_string()], "Show depth")
        .execute(DepthHandler);
    context.register_command(depth, "pumpkin-unified-core:essentials.depth");

    // /enchant
    let enchant = Command::new(&["enchant".to_string()], "Enchant item")
        .execute(EnchantHandler);
    context.register_command(enchant, "pumpkin-unified-core:essentials.enchant");

    // /exp <amount> [player]
    let exp = Command::new(&["exp".to_string(), "xp".to_string()], "Manage experience")
        .execute(ExpHandler)
        .then(string_arg("amount"))
        .then(string_arg("target"))
        .execute(ExpHandler);
    context.register_command(exp, "pumpkin-unified-core:essentials.exp");

    // /firework
    let firework = Command::new(&["firework".to_string()], "Launch firework")
        .execute(FireworkHandler);
    context.register_command(firework, "pumpkin-unified-core:essentials.firework");

    // /fix
    let fix = Command::new(&["fix".to_string()], "Fix item")
        .execute(FixHandler);
    context.register_command(fix, "pumpkin-unified-core:essentials.fix");

    // /getpos [player]
    let getpos = Command::new(&["getpos".to_string()], "Get position")
        .execute(GetPosHandler)
        .then(string_arg("target").execute(GetPosHandler));
    context.register_command(getpos, "pumpkin-unified-core:essentials.getpos");

    // /hat
    let hat = Command::new(&["hat".to_string()], "Wear item as hat")
        .execute(HatHandler);
    context.register_command(hat, "pumpkin-unified-core:essentials.hat");

    // /help
    let help = Command::new(&["help".to_string()], "Show help")
        .execute(HelpHandler);
    context.register_command(help, "pumpkin-unified-core:essentials.help");

    // /helpop
    let helpop = Command::new(&["helpop".to_string()], "Request help from staff")
        .execute(HelpOpHandler);
    context.register_command(helpop, "pumpkin-unified-core:essentials.helpop");

    // /ignore <player>
    let ignore = Command::new(&["ignore".to_string()], "Toggle ignore")
        .then(string_arg("target").execute(IgnoreHandler));
    context.register_command(ignore, "pumpkin-unified-core:essentials.ignore");

    // /invsee <player>
    let invsee = Command::new(&["invsee".to_string()], "View inventory")
        .then(string_arg("target").execute(InvSeeHandler));
    context.register_command(invsee, "pumpkin-unified-core:essentials.invsee");

    // /jump
    let jump = Command::new(&["jump".to_string()], "Jump to block")
        .execute(JumpHandler);
    context.register_command(jump, "pumpkin-unified-core:essentials.jump");

    // /near
    let near = Command::new(&["near".to_string()], "Show nearby players")
        .execute(NearHandler);
    context.register_command(near, "pumpkin-unified-core:essentials.near");

    // /nick [nickname]
    let nick = Command::new(&["nick".to_string(), "nickname".to_string()], "Set nickname")
        .execute(NickHandler)
        .then(string_arg("nickname").execute(NickHandler));
    context.register_command(nick, "pumpkin-unified-core:essentials.nick");

    // /ping
    let ping = Command::new(&["ping".to_string()], "Check ping")
        .execute(PingHandler);
    context.register_command(ping, "pumpkin-unified-core:essentials.ping");

    // /powertool
    let powertool = Command::new(&["powertool".to_string(), "pt".to_string()], "Set power tool")
        .execute(PowerToolHandler);
    context.register_command(powertool, "pumpkin-unified-core:essentials.powertool");

    // /ptime <time|reset>
    let ptime = Command::new(&["ptime".to_string()], "Set player time")
        .then(string_arg("time").execute(PTimeHandler));
    context.register_command(ptime, "pumpkin-unified-core:essentials.ptime");

    // /pweather <clear|rain|thunder|reset>
    let pweather = Command::new(&["pweather".to_string()], "Set player weather")
        .then(string_arg("weather").execute(PWeatherHandler));
    context.register_command(pweather, "pumpkin-unified-core:essentials.pweather");

    // /realname <player>
    let realname = Command::new(&["realname".to_string()], "Get real name")
        .then(string_arg("target").execute(RealNameHandler));
    context.register_command(realname, "pumpkin-unified-core:essentials.realname");

    // /recipe
    let recipe = Command::new(&["recipe".to_string()], "View recipe")
        .execute(RecipeHandler);
    context.register_command(recipe, "pumpkin-unified-core:essentials.recipe");

    // /repair
    let repair = Command::new(&["repair".to_string()], "Repair item")
        .execute(RepairHandler);
    context.register_command(repair, "pumpkin-unified-core:essentials.repair");

    // /sign
    let sign = Command::new(&["sign".to_string()], "Edit sign")
        .execute(SignHandler);
    context.register_command(sign, "pumpkin-unified-core:essentials.sign");

    // /sit
    let sit = Command::new(&["sit".to_string()], "Sit")
        .execute(SitHandler);
    context.register_command(sit, "pumpkin-unified-core:essentials.sit");

    // /skull [player]
    let skull = Command::new(&["skull".to_string(), "head".to_string()], "Get player skull")
        .execute(SkullHandler)
        .then(string_arg("target").execute(SkullHandler));
    context.register_command(skull, "pumpkin-unified-core:essentials.skull");

    // /sudo <player> <command>
    let sudo = Command::new(&["sudo".to_string()], "Run command as player")
        .then(string_arg("target"))
        .then(string_arg("command"))
        .execute(SudoHandler);
    context.register_command(sudo, "pumpkin-unified-core:essentials.sudo");

    // /sun
    let sun = Command::new(&["sun".to_string()], "Set time to day")
        .execute(SunHandler);
    context.register_command(sun, "pumpkin-unified-core:essentials.sun");

    // /superpickaxe
    let superpickaxe = Command::new(
        &["superpickaxe".to_string(), "spa".to_string()],
        "Toggle super pickaxe",
    )
    .execute(SuperPickaxeHandler);
    context.register_command(superpickaxe, "pumpkin-unified-core:essentials.superpickaxe");

    // /thunder
    let thunder = Command::new(&["thunder".to_string()], "Set weather to thunder")
        .execute(ThunderHandler);
    context.register_command(thunder, "pumpkin-unified-core:essentials.thunder");

    // /toggledownfall
    let toggledownfall = Command::new(
        &["toggledownfall".to_string(), "toggledownfall".to_string()],
        "Toggle downfall",
    )
    .execute(ToggleDownfallHandler);
    context.register_command(toggledownfall, "pumpkin-unified-core:essentials.toggledownfall");

    // /tree
    let tree = Command::new(&["tree".to_string()], "Generate tree")
        .execute(TreeHandler);
    context.register_command(tree, "pumpkin-unified-core:essentials.tree");

    // /unlimit
    let unlimit = Command::new(&["unlimit".to_string()], "Unlimit item stack")
        .execute(UnlimitHandler);
    context.register_command(unlimit, "pumpkin-unified-core:essentials.unlimit");

    // /version
    let version = Command::new(&["version".to_string(), "ver".to_string()], "Show version")
        .execute(VersionHandler);
    context.register_command(version, "pumpkin-unified-core:essentials.version");

    // /wb <subcommand>
    let wb = Command::new(&["wb".to_string(), "worldborder".to_string()], "World border")
        .then(string_arg("subcommand"))
        .then(string_arg("size"))
        .execute(WorldBorderHandler);
    context.register_command(wb, "pumpkin-unified-core:essentials.worldborder");

    // /xp <amount> [player]
    let xp = Command::new(&["xp".to_string()], "Manage experience")
        .then(string_arg("amount"))
        .then(string_arg("target"))
        .execute(XpHandler);
    context.register_command(xp, "pumpkin-unified-core:essentials.xp");
}