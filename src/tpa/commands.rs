use crate::tpa::Tpa;
use crate::utils::text::format_text;
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

pub struct TpaHandler {
    tpa: Arc<Tpa>,
    here: bool,
}

impl CommandHandler for TpaHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target_name = arg_string(&args, "target").unwrap_or_default();
        let requester = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can send tpa"));
                return Ok(1);
            }
        };
        let target = match server.get_player_by_name(&target_name) {
            Some(p) => p,
            None => {
                sender.send_message(format_text(&format!(
                    "&cPlayer not found: {target_name}"
                )));
                return Ok(1);
            }
        };
        let target_uuid = target.get_id().to_string();
        if !self.tpa.is_enabled(&target_uuid) {
            sender.send_message(format_text("&cThat player has disabled teleport requests"));
            return Ok(1);
        }
        let requester_uuid = requester.get_id().to_string();
        self.tpa.send_request(
            requester.get_name(),
            requester_uuid,
            target.get_name(),
            target_uuid.clone(),
            self.here,
        );
        sender.send_message(format_text(&format!(
            "&aSent teleport request to &f{}",
            target.get_name()
        )));
        target.send_system_message(
            format_text(&format!(
                "&e{} &ahas requested to teleport {}. Use /tpaccept or /tpdeny",
                requester.get_name(),
                if self.here { "to you" } else { "you to them" }
            )),
            false,
        );
        Ok(1)
    }
}

pub struct TpAcceptHandler {
    tpa: Arc<Tpa>,
}

impl CommandHandler for TpAcceptHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can accept tpa"));
                return Ok(1);
            }
        };
        let target_uuid = target.get_id().to_string();
        match self.tpa.accept(&target_uuid) {
            Some(request) => {
                if crate::utils::time::now_timestamp() > request.expires_at {
                    sender.send_message(format_text("&cThat teleport request has expired"));
                    return Ok(1);
                }
                // The requester teleports to the accepter's location.
                if let Some(requester) = server.get_player_by_name(&request.requester) {
                    let pos = target.get_position();
                    requester.teleport(pos, None, None, target.get_world());
                }
                sender.send_message(format_text("&aTeleport request accepted"));
            }
            None => sender.send_message(format_text("&cNo pending teleport request")),
        }
        Ok(1)
    }
}

pub struct TpDenyHandler {
    tpa: Arc<Tpa>,
}

impl CommandHandler for TpDenyHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target_uuid = sender
            .as_player()
            .map(|p| p.get_id().to_string())
            .unwrap_or_default();
        if self.tpa.deny(&target_uuid).is_some() {
            sender.send_message(format_text("&aTeleport request denied"));
        } else {
            sender.send_message(format_text("&cNo pending teleport request"));
        }
        Ok(1)
    }
}

pub struct TpToggleHandler {
    tpa: Arc<Tpa>,
}

impl CommandHandler for TpToggleHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let uuid = sender
            .as_player()
            .map(|p| p.get_id().to_string())
            .unwrap_or_default();
        let enabled = self.tpa.toggle(&uuid);
        sender.send_message(format_text(&format!(
            "&aTeleport requests {}",
            if enabled { "enabled" } else { "disabled" }
        )));
        Ok(1)
    }
}

pub fn register(tpa: Arc<Tpa>, context: &Context) {
    let tpa_cmd = Command::new(&["tpa".to_string()], "Request to teleport to a player")
        .then(string_arg("target").execute(TpaHandler {
            tpa: tpa.clone(),
            here: false,
        }));
    context.register_command(tpa_cmd, "pumpkin-unified-core.essentials.tpa");

    let tpahere = Command::new(&["tpahere".to_string()], "Request a player teleport to you")
        .then(string_arg("target").execute(TpaHandler {
            tpa: tpa.clone(),
            here: true,
        }));
    context.register_command(tpahere, "pumpkin-unified-core.essentials.tpahere");

    let tpaccept = Command::new(&["tpaccept".to_string()], "Accept a teleport request")
        .execute(TpAcceptHandler { tpa: tpa.clone() });
    context.register_command(tpaccept, "pumpkin-unified-core.essentials.tpaccept");

    let tpdeny = Command::new(&["tpdeny".to_string()], "Deny a teleport request")
        .execute(TpDenyHandler { tpa: tpa.clone() });
    context.register_command(tpdeny, "pumpkin-unified-core.essentials.tpdeny");

    let tptoggle = Command::new(&["tptoggle".to_string()], "Toggle teleport requests")
        .execute(TpToggleHandler { tpa });
    context.register_command(tptoggle, "pumpkin-unified-core.essentials.tptoggle");
}