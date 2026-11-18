use crate::homes::{Home, Homes};
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

pub struct SetHomeHandler {
    homes: Arc<Homes>,
}

impl CommandHandler for SetHomeHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "name").unwrap_or_else(|| "home".to_string());
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can set homes"));
                return Ok(1);
            }
        };
        let owner = player.get_name();
        if self.homes.get(&owner, &name).is_none() && self.homes.count(&owner) >= self.homes.max_homes()
        {
            sender.send_message(format_text(&format!(
                "&cYou have reached the maximum of {} homes",
                self.homes.max_homes()
            )));
            return Ok(1);
        }
        let pos = player.get_position();
        let home = Home {
            owner: owner.clone(),
            name: name.clone(),
            world: player.get_world().get_name(),
            x: pos.0,
            y: pos.1,
            z: pos.2,
            yaw: player.get_yaw(),
            pitch: player.get_pitch(),
        };
        self.homes.set(&home);
        sender.send_message(format_text(&format!("&aSet home &f{name}")));
        Ok(1)
    }
}

pub struct HomeHandler {
    homes: Arc<Homes>,
}

impl CommandHandler for HomeHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "name").unwrap_or_else(|| "home".to_string());
        let player = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can use homes"));
                return Ok(1);
            }
        };
        let owner = player.get_name();
        match self.homes.get(&owner, &name) {
            Some(home) => {
                let pos = (home.x, home.y, home.z);
                let world = server.get_world_by_name(&home.world);
                match world {
                    Some(w) => player.teleport(pos, Some(home.yaw), Some(home.pitch), w),
                    None => player.teleport(pos, Some(home.yaw), Some(home.pitch), player.get_world()),
                }
                sender.send_message(format_text(&format!("&aTeleported to home &f{name}")));
            }
            None => sender.send_message(format_text(&format!("&cHome not found: {name}"))),
        }
        Ok(1)
    }
}

pub struct DelHomeHandler {
    homes: Arc<Homes>,
}

impl CommandHandler for DelHomeHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "name").unwrap_or_else(|| "home".to_string());
        let owner = sender
            .as_player()
            .map(|p| p.get_name())
            .unwrap_or_default();
        if self.homes.delete(&owner, &name) {
            sender.send_message(format_text(&format!("&aDeleted home &f{name}")));
        } else {
            sender.send_message(format_text(&format!("&cHome not found: {name}")));
        }
        Ok(1)
    }
}

pub struct HomesListHandler {
    homes: Arc<Homes>,
}

impl CommandHandler for HomesListHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let owner = sender
            .as_player()
            .map(|p| p.get_name())
            .unwrap_or_default();
        let homes = self.homes.list(&owner);
        if homes.is_empty() {
            sender.send_message(format_text("&7No homes set"));
            return Ok(1);
        }
        sender.send_message(format_text("&6Your homes:"));
        for home in homes {
            sender.send_message(format_text(&format!(
                "&e{} &7in {}",
                home.name, home.world
            )));
        }
        Ok(1)
    }
}

pub fn register(homes: Arc<Homes>, context: &Context) {
    let sethome = Command::new(&["sethome".to_string()], "Set a home")
        .then(string_arg("name").execute(SetHomeHandler {
            homes: homes.clone(),
        }));
    context.register_command(sethome, "essentials.sethome");

    let home = Command::new(&["home".to_string()], "Teleport to a home")
        .then(string_arg("name").execute(HomeHandler {
            homes: homes.clone(),
        }));
    context.register_command(home, "essentials.home");

    let delhome = Command::new(&["delhome".to_string()], "Delete a home")
        .then(string_arg("name").execute(DelHomeHandler {
            homes: homes.clone(),
        }));
    context.register_command(delhome, "essentials.delhome");

    let homes_list = Command::new(&["homes".to_string()], "List homes")
        .execute(HomesListHandler { homes });
    context.register_command(homes_list, "essentials.homes");
}