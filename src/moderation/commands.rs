use crate::moderation::{Moderation, Punishment};
use crate::utils::text::format_text;
use crate::utils::uuid_resolver::ResolvedPlayer;
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

fn arg_greedy(args: &ConsumedArgs, key: &str) -> Option<String> {
    match args.get_value(key) {
        Arg::Msg(s) => Some(s),
        _ => None,
    }
}

fn greedy_arg(name: &str) -> CommandNode {
    CommandNode::argument(name, &ArgumentType::String(StringType::Greedy))
}

fn string_arg(name: &str) -> CommandNode {
    CommandNode::argument(name, &ArgumentType::String(StringType::SingleWord))
}

/// Resolves a target player identifier (name or UUID) to a ResolvedPlayer,
/// checking online players first, then UUID cache, then Mojang API.
async fn resolve_target(
    moderation: &Arc<Moderation>,
    server: &Server,
    identifier: &str,
) -> Option<ResolvedPlayer> {
    moderation.uuid_resolver().resolve(server, identifier).await
}

/// Resolves a target player identifier for commands that need online players only.
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
        
        // Resolve target using UUID resolver (supports offline players)
        let resolved = self.moderation.uuid_resolver().resolve_sync(&server, &name);
        let (uuid_string, uuid, display_name) = match resolved {
            Some(r) => (r.uuid, r.uuid.parse().ok(), r.name),
            None => {
                // Fallback: use name as UUID for completely unknown players
                self.moderation.uuid_resolver().cache_negative_sync(&name);
                (name.clone(), None, name.clone())
            }
        };

        let ban_manager = server.get_ban_manager();
        if let Some(id) = uuid {
            ban_manager.ban_player(
                &display_name,
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
            name: display_name.clone(),
            kind: "ban".to_string(),
            reason: reason.clone(),
            created_at: crate::utils::time::now_timestamp(),
            expires_at: None,
        });
        sender.send_message(format_text(&format!(
            "&aBanned &f{display_name} &7({reason})"
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

// ---------------- tempban (with UUID resolver) ----------------

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

        // Resolve target using UUID resolver
        let resolved = self.moderation.uuid_resolver().resolve_sync(&server, &name);
        let (uuid_string, uuid, display_name) = match resolved {
            Some(r) => (r.uuid, r.uuid.parse().ok(), r.name),
            None => {
                self.moderation.uuid_resolver().cache_negative_sync(&name);
                (name.clone(), None, name.clone())
            }
        };

        if let Some(id) = uuid {
            let ban_manager = server.get_ban_manager();
            ban_manager.ban_player(
                &display_name,
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
            name: display_name.clone(),
            kind: "tempban".to_string(),
            reason: reason.clone(),
            created_at: now,
            expires_at: Some(now + seconds),
        });
        sender.send_message(format_text(&format!(
            "&aTemporarily banned &f{display_name} &7for {}",
            crate::utils::time::format_duration(seconds)
        )));
        Ok(1)
    }
}

// ---------------- unban (with UUID resolver) ----------------

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
        
        // Resolve target using UUID resolver
        let resolved = self.moderation.uuid_resolver().resolve_sync(&server, &name);
        let (uuid_string, display_name) = match resolved {
            Some(r) => (r.uuid, r.name),
            None => {
                // For unban, we need to check stored punishments by name too
                if let Some(punishment) = self.moderation.get_punishment_by_name(&name) {
                    (punishment.uuid, punishment.name)
                } else {
                    (name.clone(), name.clone())
                }
            }
        };

        let ban_manager = server.get_ban_manager();
        ban_manager.pardon_player(&display_name);

        self.moderation.remove_punishment(&uuid_string);
        sender.send_message(format_text(&format!("&aUnbanned &f{display_name}")));
        Ok(1)
    }
}
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

// ---------------- jail ----------------

pub struct JailHandler {
    moderation: Arc<Moderation>,
}

impl CommandHandler for JailHandler {
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
        let uuid = target.get_id().to_string();
        let new_state = !self.moderation.state(&uuid).jailed;
        self.moderation.set_jailed(&uuid, new_state);
        sender.send_message(format_text(&format!(
            "&aJail {} for &f{}",
            if new_state { "enabled" } else { "disabled" },
            target_name
        )));
        Ok(1)
    }
}

// ---------------- warn ----------------

pub struct WarnHandler {
    moderation: Arc<Moderation>,
}

impl CommandHandler for WarnHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target_name = arg_string(&args, "target").unwrap_or_default();
        let reason = arg_greedy(&args, "reason").unwrap_or("No reason given".to_string());
        let target = match server.get_player_by_name(&target_name) {
            Some(p) => p,
            None => {
                sender.send_message(format_text(&format!("&cPlayer not found: {target_name}")));
                return Ok(1);
            }
        };
        // Store warning as punishment
        let punishment = Punishment {
            uuid: target.get_id().to_string(),
            name: target.get_name(),
            kind: "warn".to_string(),
            reason,
            created_at: crate::utils::time::now_timestamp(),
            expires_at: None,
        };
        self.moderation.set_punishment(&punishment);
        target.send_system_message(
            format_text(&format!("&cWarning: &f{}", punishment.reason)),
            false,
        );
        sender.send_message(format_text(&format!(
            "&aWarned &f{} &afor &f{}",
            target.get_name(),
            punishment.reason
        )));
        Ok(1)
    }
}

// ---------------- socialspy ----------------

pub struct SocialSpyHandler {
    moderation: Arc<Moderation>,
}

impl CommandHandler for SocialSpyHandler {
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
            sender.send_message(format_text("&cUsage: /socialspy [player]"));
            return Ok(1);
        }
        let target = match server.get_player_by_name(&target_name) {
            Some(p) => p,
            None => {
                sender.send_message(format_text(&format!("&cPlayer not found: {target_name}")));
                return Ok(1);
            }
        };
        let uuid = target.get_id().to_string();
        let new_state = !self.moderation.state(&uuid).social_spy;
        self.moderation.set_social_spy(&uuid, new_state);
        sender.send_message(format_text(&format!(
            "&aSocial Spy {} for &f{}",
            if new_state { "enabled" } else { "disabled" },
            target_name
        )));
        Ok(1)
    }
}

// ---------------- nopain ----------------

pub struct NoPainHandler {
    moderation: Arc<Moderation>,
}

impl CommandHandler for NoPainHandler {
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
            sender.send_message(format_text("&cUsage: /nopain [player]"));
            return Ok(1);
        }
        let target = match server.get_player_by_name(&target_name) {
            Some(p) => p,
            None => {
                sender.send_message(format_text(&format!("&cPlayer not found: {target_name}")));
                return Ok(1);
            }
        };
        let uuid = target.get_id().to_string();
        let new_state = !self.moderation.state(&uuid).no_pain;
        self.moderation.set_no_pain(&uuid, new_state);
        sender.send_message(format_text(&format!(
            "&aNo Pain {} for &f{}",
            if new_state { "enabled" } else { "disabled" },
            target_name
        )));
        Ok(1)
    }
}

// ---------------- keepinv ----------------

pub struct KeepInvHandler {
    moderation: Arc<Moderation>,
}

impl CommandHandler for KeepInvHandler {
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
            sender.send_message(format_text("&cUsage: /keepinv [player]"));
            return Ok(1);
        }
        let target = match server.get_player_by_name(&target_name) {
            Some(p) => p,
            None => {
                sender.send_message(format_text(&format!("&cPlayer not found: {target_name}")));
                return Ok(1);
            }
        };
        let uuid = target.get_id().to_string();
        let new_state = !self.moderation.state(&uuid).keep_inventory;
        self.moderation.set_keep_inventory(&uuid, new_state);
        sender.send_message(format_text(&format!(
            "&aKeep Inventory {} for &f{}",
            if new_state { "enabled" } else { "disabled" },
            target_name
        )));
        Ok(1)
    }
}

// ---------------- kill ----------------

pub struct KillHandler;

impl CommandHandler for KillHandler {
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
            sender.send_message(format_text("&cUsage: /kill [player]"));
            return Ok(1);
        }
        let target = match server.get_player_by_name(&target_name) {
            Some(p) => p,
            None => {
                sender.send_message(format_text(&format!("&cPlayer not found: {target_name}")));
                return Ok(1);
            }
        };
        target.set_health(0.0);
        sender.send_message(format_text(&format!("&aKilled &f{}", target.get_name())));
        Ok(1)
    }
}

// ---------------- killall ----------------

pub struct KillAllHandler;

impl CommandHandler for KillAllHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        // This would require entity API to kill all mobs
        sender.send_message(format_text("&eKillAll requires entity API integration"));
        Ok(1)
    }
}

// ---------------- suicide ----------------

pub struct SuicideHandler;

impl CommandHandler for SuicideHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use /suicide"));
                return Ok(1);
            }
        };
        player.set_health(0.0);
        sender.send_message(format_text("&aYou committed suicide"));
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
    context.register_command(kick, "pumpkin-unified-core:essentials.kick");

    let ban = Command::new(&["ban".to_string()], "Ban a player")
        .then(target_reason().execute(BanHandler {
            moderation: moderation.clone(),
        }));
    context.register_command(ban, "pumpkin-unified-core:essentials.ban");

    let tempban = Command::new(&["tempban".to_string()], "Temporarily ban a player").then(
        string_arg("target")
            .then(string_arg("duration").then(greedy_arg("reason").execute(TempBanHandler {
                moderation: moderation.clone(),
            }))),
    );
    context.register_command(tempban, "pumpkin-unified-core:essentials.tempban");

    let unban = Command::new(&["unban".to_string()], "Unban a player")
        .then(string_arg("target").execute(UnBanHandler {
            moderation: moderation.clone(),
        }));
    context.register_command(unban, "pumpkin-unified-core:essentials.unban");

    let mute = Command::new(&["mute".to_string()], "Mute a player")
        .then(target_reason().execute(MuteHandler {
            moderation: moderation.clone(),
        }));
    context.register_command(mute, "pumpkin-unified-core:essentials.mute");

    let unmute = Command::new(&["unmute".to_string()], "Unmute a player")
        .then(string_arg("target").execute(UnMuteHandler {
            moderation: moderation.clone(),
        }));
    context.register_command(unmute, "pumpkin-unified-core:essentials.unmute");

    let fly = Command::new(&["fly".to_string()], "Toggle flight")
        .then(string_arg("target").execute(FlyHandler {
            moderation: moderation.clone(),
        }));
    context.register_command(fly, "pumpkin-unified-core:essentials.fly");

    let god = Command::new(&["god".to_string()], "Toggle god mode")
        .then(string_arg("target").execute(GodHandler {
            moderation: moderation.clone(),
        }));
    context.register_command(god, "pumpkin-unified-core:essentials.god");

    let heal = Command::new(&["heal".to_string()], "Heal a player")
        .then(string_arg("target").execute(HealHandler));
    context.register_command(heal, "pumpkin-unified-core:essentials.heal");

    let feed = Command::new(&["feed".to_string()], "Feed a player")
        .then(string_arg("target").execute(FeedHandler));
    context.register_command(feed, "pumpkin-unified-core:essentials.feed");

    let vanish = Command::new(&["vanish".to_string()], "Toggle vanish")
        .then(string_arg("target").execute(VanishHandler {
            moderation: moderation.clone(),
        }));
    context.register_command(vanish, "pumpkin-unified-core:essentials.vanish");

    // /jail <player>
    let jail = Command::new(&["jail".to_string()], "Jail a player")
        .then(string_arg("target").execute(JailHandler {
            moderation: moderation.clone(),
        }));
    context.register_command(jail, "pumpkin-unified-core:essentials.jail");

    // /warn <player> <reason>
    let warn = Command::new(&["warn".to_string()], "Warn a player")
        .then(target_reason().execute(WarnHandler {
            moderation: moderation.clone(),
        }));
    context.register_command(warn, "pumpkin-unified-core:essentials.warn");

    // /socialspy [player]
    let socialspy = Command::new(&["socialspy".to_string()], "Toggle social spy")
        .then(string_arg("target").execute(SocialSpyHandler {
            moderation: moderation.clone(),
        }))
        .execute(SocialSpyHandler {
            moderation: moderation.clone(),
        });
    context.register_command(socialspy, "pumpkin-unified-core:essentials.socialspy");

    // /nopain [player]
    let nopain = Command::new(&["nopain".to_string()], "Toggle no pain")
        .then(string_arg("target").execute(NoPainHandler {
            moderation: moderation.clone(),
        }))
        .execute(NoPainHandler {
            moderation: moderation.clone(),
        });
    context.register_command(nopain, "pumpkin-unified-core:essentials.nopain");

    // /keepinv [player]
    let keepinv = Command::new(&["keepinv".to_string()], "Toggle keep inventory")
        .then(string_arg("target").execute(KeepInvHandler {
            moderation: moderation.clone(),
        }))
        .execute(KeepInvHandler {
            moderation: moderation.clone(),
        });
    context.register_command(keepinv, "pumpkin-unified-core:essentials.keepinv");

    // /kill [player]
    let kill = Command::new(&["kill".to_string()], "Kill a player")
        .then(string_arg("target").execute(KillHandler))
        .execute(KillHandler);
    context.register_command(kill, "pumpkin-unified-core:essentials.kill");

    // /killall
    let killall = Command::new(&["killall".to_string()], "Kill all entities")
        .execute(KillAllHandler);
    context.register_command(killall, "pumpkin-unified-core:essentials.killall");

    // /suicide
    let suicide = Command::new(&["suicide".to_string()], "Kill yourself")
        .execute(SuicideHandler);
    context.register_command(suicide, "pumpkin-unified-core:essentials.suicide");
}