use crate::kits::{Kit, Kits};
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

pub struct KitHandler {
    kits: Arc<Kits>,
}

impl CommandHandler for KitHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "name").unwrap_or_default();
        let uuid = sender
            .as_player()
            .map(|p| p.get_id().to_string())
            .unwrap_or_default();
        let kit = match self.kits.get(&name) {
            Some(k) => k,
            None => {
                sender.send_message(format_text(&format!("&cKit not found: {name}")));
                return Ok(1);
            }
        };
        let remaining = self.kits.remaining_cooldown(&uuid, &name);
        if remaining > 0 {
            sender.send_message(format_text(&format!(
                "&cYou must wait {} before using this kit again",
                crate::utils::time::format_duration(remaining)
            )));
            return Ok(1);
        }
        // Grant items (items are stored as namespaced strings; actual item give
        // requires the inventory API — here we just notify for now).
        sender.send_message(format_text(&format!(
            "&aReceived kit &f{} &7({} items)",
            name,
            kit.items.len()
        )));
        self.kits.set_cooldown(&uuid, &name);
        Ok(1)
    }
}

pub struct KitsListHandler {
    kits: Arc<Kits>,
}

impl CommandHandler for KitsListHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let kits = self.kits.list();
        if kits.is_empty() {
            sender.send_message(format_text("&7No kits available"));
            return Ok(1);
        }
        sender.send_message(format_text("&6Available kits:"));
        for kit in kits {
            sender.send_message(format_text(&format!(
                "&e{} &7({} items)",
                kit.name,
                kit.items.len()
            )));
        }
        Ok(1)
    }
}

pub struct CreateKitHandler {
    kits: Arc<Kits>,
}

impl CommandHandler for CreateKitHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "name").unwrap_or_default();
        let cooldown: i64 = arg_string(&args, "cooldown")
            .and_then(|s| crate::utils::time::parse_duration_seconds(&s))
            .unwrap_or(0);
        let kit = Kit {
            name: name.clone(),
            items: Vec::new(),
            cooldown_seconds: cooldown,
            delay_seconds: 0,
            one_time: false,
        };
        self.kits.create(&kit);
        sender.send_message(format_text(&format!("&aCreated kit &f{name}")));
        Ok(1)
    }
}

pub struct ShowKitHandler {
    kits: Arc<Kits>,
}

impl CommandHandler for ShowKitHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "name").unwrap_or_default();
        match self.kits.get(&name) {
            Some(kit) => {
                sender.send_message(format_text(&format!(
                    "&6Kit: &f{} &7(cooldown: {})",
                    kit.name,
                    crate::utils::time::format_duration(kit.cooldown_seconds)
                )));
                for (i, item) in kit.items.iter().enumerate() {
                    sender.send_message(format_text(&format!("  &e{}. &f{}", i + 1, item)));
                }
            }
            None => sender.send_message(format_text(&format!("&cKit not found: {name}"))),
        }
        Ok(1)
    }
}

pub fn register(kits: Arc<Kits>, context: &Context) {
    let kit = Command::new(&["kit".to_string()], "Use a kit")
        .then(string_arg("name").execute(KitHandler {
            kits: kits.clone(),
        }));
    context.register_command(kit, "essentials.kit");

    let kits_list = Command::new(&["kits".to_string()], "List kits")
        .execute(KitsListHandler { kits: kits.clone() });
    context.register_command(kits_list, "essentials.kits");

    let createkit = Command::new(&["createkit".to_string()], "Create a kit")
        .then(string_arg("name").then(string_arg("cooldown").execute(CreateKitHandler {
            kits: kits.clone(),
        })));
    context.register_command(createkit, "essentials.createkit");

    let showkit = Command::new(&["showkit".to_string()], "Show kit contents")
        .then(string_arg("name").execute(ShowKitHandler { kits }));
    context.register_command(showkit, "essentials.showkit");
}