use crate::utils::text::format_text;
use crate::warps::{Warp, Warps};
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

pub struct WarpHandler {
    warps: Arc<Warps>,
}

impl CommandHandler for WarpHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "name").unwrap_or_default();
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use warps"));
                return Ok(1);
            }
        };
        match self.warps.get(&name) {
            Some(warp) => {
                let world = server.get_world_by_name(&warp.world);
                let pos = (warp.x, warp.y, warp.z);
                match world {
                    Some(w) => player.teleport(pos, Some(warp.yaw), Some(warp.pitch), w),
                    None => player.teleport(pos, Some(warp.yaw), Some(warp.pitch), player.get_world()),
                }
                sender.send_message(format_text(&format!("&aTeleported to warp &f{name}")));
            }
            None => sender.send_message(format_text(&format!("&cWarp not found: {name}"))),
        }
        Ok(1)
    }
}

pub struct SetWarpHandler {
    warps: Arc<Warps>,
}

impl CommandHandler for SetWarpHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "name").unwrap_or_default();
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can set warps"));
                return Ok(1);
            }
        };
        let pos = player.get_position();
        let warp = Warp {
            name: name.clone(),
            world: player.get_world().get_name(),
            x: pos.0,
            y: pos.1,
            z: pos.2,
            yaw: player.get_yaw(),
            pitch: player.get_pitch(),
            created_by: player.get_name(),
            created_at: crate::utils::time::now_timestamp(),
        };
        self.warps.set(&warp);
        sender.send_message(format_text(&format!("&aCreated warp &f{name}")));
        Ok(1)
    }
}

pub struct DelWarpHandler {
    warps: Arc<Warps>,
}

impl CommandHandler for DelWarpHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "name").unwrap_or_default();
        if self.warps.delete(&name) {
            sender.send_message(format_text(&format!("&aDeleted warp &f{name}")));
        } else {
            sender.send_message(format_text(&format!("&cWarp not found: {name}")));
        }
        Ok(1)
    }
}

pub struct WarpsListHandler {
    warps: Arc<Warps>,
}

impl CommandHandler for WarpsListHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let warps = self.warps.list();
        if warps.is_empty() {
            sender.send_message(format_text("&7No warps set"));
            return Ok(1);
        }
        sender.send_message(format_text("&6Warps:"));
        for warp in warps {
            sender.send_message(format_text(&format!(
                "&e{} &7(by {})",
                warp.name, warp.created_by
            )));
        }
        Ok(1)
    }
}

pub fn register(warps: Arc<Warps>, context: &Context) {
    let warp = Command::new(&["warp".to_string()], "Teleport to a warp")
        .then(string_arg("name").execute(WarpHandler {
            warps: warps.clone(),
        }));
    context.register_command(warp, "essentials.warp");

    let setwarp = Command::new(&["setwarp".to_string()], "Create a warp")
        .then(string_arg("name").execute(SetWarpHandler {
            warps: warps.clone(),
        }));
    context.register_command(setwarp, "essentials.setwarp");

    let delwarp = Command::new(&["delwarp".to_string()], "Delete a warp")
        .then(string_arg("name").execute(DelWarpHandler {
            warps: warps.clone(),
        }));
    context.register_command(delwarp, "essentials.delwarp");

    let warps_list = Command::new(&["warps".to_string()], "List warps")
        .execute(WarpsListHandler { warps });
    context.register_command(warps_list, "essentials.warps");
}