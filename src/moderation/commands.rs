use crate::moderation::{Moderation, Punishment};
use crate::utils::text::format_text;
use pumpkin_plugin_api::{
    command::{
        Arg, ArgumentType, Command, CommandError, CommandNode, CommandSender, ConsumedArgs,
        StringType,
    },
    commands::CommandHandler,
    Context, Server,
};
use pumpkin_plugin_api::wit::pumpkin::plugin::player::{
    BanPlayerOptions, JavaKickOptions, SocketTeardownPolicy,
};
use std::sync::Arc;

fn arg_string(args: &ConsumedArgs, key: &str) -> Option<String> {
    match args.get_value(key) {
        Arg::Simple(s) | Arg::Msg(s) => Some(s),
        _ => None,
    }
}

fn greedy_arg(name: &str) -> CommandNode {
    CommandNode::argument(name, &ArgumentType::String(StringType::Greedy))
}

fn string_arg(name: &str) -> CommandNode {
    CommandNode::argument(name, &ArgumentType::String(StringType::SingleWord))
}

/// Resolves a target player name to an online `Player`, sending an error if missing.
fn require_online(
    sender: &CommandSender,
    server: &Server,
    name: &str,
) -> Option<pumpkin_plugin_api::player::Player> {
    match server.get_player_by_name(name) {
        Some(p) => Some(p),
        None => {
            sender.send_message(format_text(&format!("&cPlayer not found: {name}")));
            None
        }
    }
}

// ---------------- kick ----------------

pub struct KickHandler;

impl CommandHandler for KickHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "target").unwrap_or_default();
        let reason = arg_string(&args, "reason").unwrap_or_else(|| "Kicked".to_string());
        let player = match require_online(&sender, &server, &name) {
            Some(p) => p,
            None => return Ok(1),
        };
        let options = JavaKickOptions {
            reason: format_text(&reason),
            log_to_console: true,
            teardown_policy: SocketTeardownPolicy::Graceful,
        };
        match player.as_java() {
            Some(java_player) => java_player.kick(options),
            None => {
                sender.send_message(format_text(
                    "&cCannot kick Bedrock player (no Java kick available)",
                ));
                return Ok(1);
            }
        }
        sender.send_message(format_text(&format!("&aKicked &f{name}")));
        Ok(1)
    }
}

// ---------------- ban ----------------

pub struct BanHandler {
    moderation: Arc<Moderation>,
}

impl CommandHandler for BanHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "target").unwrap_or_default();
        let reason = arg_string(&args, "reason").unwrap_or_else(|| "Banned".to_string());
        // Ban works for offline players too, via the ban manager. We need a uuid:
        // use the online player's uuid if online, else generate a placeholder keyed by name.
        let (uuid_string, uuid) = if let Some(player) = server.get_player_by_name(&name) {
            let id = player.get_id();
            (id.to_string(), Some(id))
        } else {
            (name.clone(), None)
        };

        let ban_manager = server.get_ban_manager();
        if let Some(id) = uuid {
            ban_manager.ban_player(
                &name,
                id,
                BanPlayerOptions {
                    reason: Some(format_text(&reason)),
                    source: Some("pumpkin-unified-core".to_string()),
                    expires_at_utc: None,
                    duration_seconds: None,
                    kick_if_online: true,
                    log_to_console: true,
                },
            );
        }

        self.moderation.set_punishment(&Punishment {
            uuid: uuid_string,
            name: name.clone(),
            kind: "ban".to_string(),
            reason: reason.clone(),
            created_at: crate::utils::time::now_timestamp(),
            expires_at: None,
        });
        sender.send_message(format_text(&format!(
            "&aBanned &f{name} &7({reason})"
        )));
        Ok(1)
    }
}

// ---------------- tempban ----------------

pub struct TempBanHandler {
    moderation: Arc<Moderation>,
}

impl CommandHandler for TempBanHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "target").unwrap_or_default();
        let duration = arg_string(&args, "duration").unwrap_or_default();
        let reason = arg_string(&args, "reason").unwrap_or_else(|| "Temp banned".to_string());
        let seconds = crate::utils::time::parse_duration_seconds(&duration);
        let seconds = match seconds {
            Some(s) => s,
            None => {
                sender.send_message(format_text("&cInvalid duration"));
                return Ok(1);
            }
        };

        let (uuid_string, uuid) = if let Some(player) = server.get_player_by_name(&name) {
            let id = player.get_id();
            (id.to_string(), Some(id))
        } else {
            (name.clone(), None)
        };

        if let Some(id) = uuid {
            let ban_manager = server.get_ban_manager();
            ban_manager.ban_player(
                &name,
                id,
                BanPlayerOptions {
                    reason: Some(format_text(&reason)),
                    source: Some("pumpkin-unified-core".to_string()),
                    expires_at_utc: None,
                    duration_seconds: Some(seconds as u64),
                    kick_if_online: true,
                    log_to_console: true,
                },
            );
        }

        let now = crate::utils::time::now_timestamp();
        self.moderation.set_punishment(&Punishment {
            uuid: uuid_string,
            name: name.clone(),
            kind: "tempban".to_string(),
            reason: reason.clone(),
            created_at: now,
            expires_at: Some(now + seconds),
        });
        sender.send_message(format_text(&format!(
            "&aTemporarily banned &f{name} &7for {}",
            crate::utils::time::format_duration(seconds)
        )));
        Ok(1)
    }
}

// ---------------- unban ----------------

pub struct UnBanHandler {
    moderation: Arc<Moderation>,
}

impl CommandHandler for UnBanHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "target").unwrap_or_default();
        let ban_manager = server.get_ban_manager();
        // Try online uuid first, then fall back to stored punishment uuid.
        let removed = if let Some(player) = server.get_player_by_name(&name) {
            ban_manager.unban_player(player.get_id())
        } else if let Some(p) = self.moderation.get_punishment_by_name(&name) {
            self.moderation.remove_punishment(&p.uuid);
            true
        } else {
            false
        };
        if removed {
            sender.send_message(format_text(&format!("&aUnbanned &f{name}")));
        } else {
            sender.send_message(format_text(&format!("&c{} is not banned", name)));
        }
        Ok(1)
    }
}

// ---------------- mute ----------------

pub struct MuteHandler {
    moderation: Arc<Moderation>,
}

impl CommandHandler for MuteHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "target").unwrap_or_default();
        let reason = arg_string(&args, "reason").unwrap_or_else(|| "Muted".to_string());
        let uuid = name.clone();
        self.moderation.set_punishment(&Punishment {
            uuid: uuid.clone(),
            name: name.clone(),
            kind: "mute".to_string(),
            reason: reason.clone(),
            created_at: crate::utils::time::now_timestamp(),
            expires_at: None,
        });
        sender.send_message(format_text(&format!(
            "&aMuted &f{name} &7({reason})"
        )));
        Ok(1)
    }
}

pub struct UnMuteHandler {
    moderation: Arc<Moderation>,
}

impl CommandHandler for UnMuteHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "target").unwrap_or_default();
        self.moderation.remove_punishment(&name);
        sender.send_message(format_text(&format!("&aUnmuted &f{name}")));
        Ok(1)
    }
}

// ---------------- fly ----------------

pub struct FlyHandler {
    moderation: Arc<Moderation>,
}

impl CommandHandler for FlyHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "target");
        let player = if let Some(name) = name {
            match require_online(&sender, &server, &name) {
                Some(p) => p,
                None => return Ok(1),
            }
        } else {
            match sender.as_player() {
                Some(p) => p,
                None => {
                    sender.send_message(format_text("&cOnly players can use /fly"));
                    return Ok(1);
                }
            }
        };
        let uuid = player.get_id().to_string();
        let new_state = !self.moderation.state(&uuid).flying;
        self.moderation.set_flying(&uuid, new_state);
        player.set_allow_flight(new_state);
        player.set_flying(new_state);
        sender.send_message(format_text(&format!(
            "&aFlight {} for &f{}",
            if new_state { "enabled" } else { "disabled" },
            player.get_name()
        )));
        Ok(1)
    }
}

// ---------------- god ----------------

pub struct GodHandler {
    moderation: Arc<Moderation>,
}

impl CommandHandler for GodHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "target");
        let player = if let Some(name) = name {
            match require_online(&sender, &server, &name) {
                Some(p) => p,
                None => return Ok(1),
            }
        } else {
            match sender.as_player() {
                Some(p) => p,
                None => {
                    sender.send_message(format_text("&cOnly players can use /god"));
                    return Ok(1);
                }
            }
        };
        let uuid = player.get_id().to_string();
        let new_state = !self.moderation.state(&uuid).god;
        self.moderation.set_god(&uuid, new_state);
        player.set_invulnerable(new_state);
        sender.send_message(format_text(&format!(
            "&aGod mode {} for &f{}",
            if new_state { "enabled" } else { "disabled" },
            player.get_name()
        )));
        Ok(1)
    }
}

// ---------------- heal ----------------

pub struct HealHandler;

impl CommandHandler for HealHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "target");
        let player = if let Some(name) = name {
            match require_online(&sender, &server, &name) {
                Some(p) => p,
                None => return Ok(1),
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
        let max = player.get_max_health();
        player.set_health(max);
        player.set_food_level(20);
        sender.send_message(format_text(&format!(
            "&aHealed &f{}",
            player.get_name()
        )));
        Ok(1)
    }
}

// ---------------- feed ----------------

pub struct FeedHandler;

impl CommandHandler for FeedHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "target");
        let player = if let Some(name) = name {
            match require_online(&sender, &server, &name) {
                Some(p) => p,
                None => return Ok(1),
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
        player.set_food_level(20);
        player.set_saturation(20.0);
        sender.send_message(format_text(&format!(
            "&aFed &f{}",
            player.get_name()
        )));
        Ok(1)
    }
}

// ---------------- vanish ----------------

pub struct VanishHandler {
    moderation: Arc<Moderation>,
}

impl CommandHandler for VanishHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "target");
        let player = if let Some(name) = name {
            match require_online(&sender, &server, &name) {
                Some(p) => p,
                None => return Ok(1),
            }
        } else {
            match sender.as_player() {
                Some(p) => p,
                None => {
                    sender.send_message(format_text("&cOnly players can use /vanish"));
                    return Ok(1);
                }
            }
        };
        let uuid = player.get_id().to_string();
        let target_name = player.get_name();
        let new_state = !self.moderation.state(&uuid).vanished;
        self.moderation.set_vanished(&uuid, new_state);
        // Hide/show the target player to/from every other player. Because player
        // handles are not `Clone`, look the target up fresh by name each iteration.
        for other in server.get_all_players() {
            if other.get_name() == target_name {
                continue;
            }
            if let Some(target) = server.get_player_by_name(&target_name) {
                if new_state {
                    other.hide_player(target);
                } else {
                    other.show_player(target);
                }
            }
        }
        sender.send_message(format_text(&format!(
            "&aVanish {} for &f{}",
            if new_state { "enabled" } else { "disabled" },
            target_name
        )));
        Ok(1)
    }
}

// ---------------- registration ----------------

fn target_reason() -> CommandNode {
    string_arg("target").then(greedy_arg("reason"))
}

pub fn register(moderation: Arc<Moderation>, context: &Context) {
    let kick = Command::new(&["kick".to_string()], "Kick a player")
        .then(target_reason().execute(KickHandler));
    context.register_command(kick, "pumpkin-unified-core.essentials.kick");

    let ban = Command::new(&["ban".to_string()], "Ban a player")
        .then(target_reason().execute(BanHandler {
            moderation: moderation.clone(),
        }));
    context.register_command(ban, "pumpkin-unified-core.essentials.ban");

    let tempban = Command::new(&["tempban".to_string()], "Temporarily ban a player").then(
        string_arg("target")
            .then(string_arg("duration").then(greedy_arg("reason").execute(TempBanHandler {
                moderation: moderation.clone(),
            }))),
    );
    context.register_command(tempban, "pumpkin-unified-core.essentials.tempban");

    let unban = Command::new(&["unban".to_string()], "Unban a player")
        .then(string_arg("target").execute(UnBanHandler {
            moderation: moderation.clone(),
        }));
    context.register_command(unban, "pumpkin-unified-core.essentials.unban");

    let mute = Command::new(&["mute".to_string()], "Mute a player")
        .then(target_reason().execute(MuteHandler {
            moderation: moderation.clone(),
        }));
    context.register_command(mute, "pumpkin-unified-core.essentials.mute");

    let unmute = Command::new(&["unmute".to_string()], "Unmute a player")
        .then(string_arg("target").execute(UnMuteHandler {
            moderation: moderation.clone(),
        }));
    context.register_command(unmute, "pumpkin-unified-core.essentials.unmute");

    let fly = Command::new(&["fly".to_string()], "Toggle flight")
        .then(string_arg("target").execute(FlyHandler {
            moderation: moderation.clone(),
        }));
    context.register_command(fly, "pumpkin-unified-core.essentials.fly");

    let god = Command::new(&["god".to_string()], "Toggle god mode")
        .then(string_arg("target").execute(GodHandler {
            moderation: moderation.clone(),
        }));
    context.register_command(god, "pumpkin-unified-core.essentials.god");

    let heal = Command::new(&["heal".to_string()], "Heal a player")
        .then(string_arg("target").execute(HealHandler));
    context.register_command(heal, "pumpkin-unified-core.essentials.heal");

    let feed = Command::new(&["feed".to_string()], "Feed a player")
        .then(string_arg("target").execute(FeedHandler));
    context.register_command(feed, "pumpkin-unified-core.essentials.feed");

    let vanish = Command::new(&["vanish".to_string()], "Toggle vanish")
        .then(string_arg("target").execute(VanishHandler {
            moderation: moderation.clone(),
        }));
    context.register_command(vanish, "pumpkin-unified-core.essentials.vanish");
}