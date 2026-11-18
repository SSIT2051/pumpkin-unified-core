use crate::chat::Chat;
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

fn greedy_arg(name: &str) -> CommandNode {
    CommandNode::argument(name, &ArgumentType::String(StringType::Greedy))
}

// ---------------- msg ----------------

pub struct MsgHandler {
    chat: Arc<Chat>,
}

impl CommandHandler for MsgHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target_name = arg_string(&args, "target").unwrap_or_default();
        let message = arg_string(&args, "message").unwrap_or_default();
        let sender_name = sender.get_name();

        let target = match server.get_player_by_name(&target_name) {
            Some(p) => p,
            None => {
                sender.send_message(format_text(&format!(
                    "&cPlayer not found: {target_name}"
                )));
                return Ok(1);
            }
        };

        self.chat.set_reply(&target_name, &sender_name);
        self.chat.set_reply(&sender_name, &target_name);

        sender.send_message(format_text(&format!(
            "&7[me -> {}] &f{}",
            target.get_name(),
            message
        )));
        target.send_system_message(
            format_text(&format!(
                "&7[{}] &f{}",
                sender_name, message
            )),
            false,
        );
        Ok(1)
    }
}

// ---------------- r (reply) ----------------

pub struct ReplyHandler {
    chat: Arc<Chat>,
}

impl CommandHandler for ReplyHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let message = arg_string(&args, "message").unwrap_or_default();
        let sender_name = sender.get_name();
        let target_name = match self.chat.get_reply(&sender_name) {
            Some(t) => t,
            None => {
                sender.send_message(format_text("&cNo one to reply to"));
                return Ok(1);
            }
        };
        let target = match server.get_player_by_name(&target_name) {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cThat player is no longer online"));
                return Ok(1);
            }
        };
        sender.send_message(format_text(&format!(
            "&7[me -> {}] &f{}",
            target.get_name(),
            message
        )));
        target.send_system_message(
            format_text(&format!(
                "&7[{}] &f{}",
                sender_name, message
            )),
            false,
        );
        Ok(1)
    }
}

// ---------------- mail ----------------

pub struct MailSendHandler {
    chat: Arc<Chat>,
}

impl CommandHandler for MailSendHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target_name = arg_string(&args, "target").unwrap_or_default();
        let message = arg_string(&args, "message").unwrap_or_default();
        self.chat
            .send_mail(&sender.get_name(), &target_name, &message);
        sender.send_message(format_text(&format!(
            "&aMail sent to &f{target_name}"
        )));
        Ok(1)
    }
}

pub struct MailReadHandler {
    chat: Arc<Chat>,
}

impl CommandHandler for MailReadHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = sender.get_name();
        let mails = self.chat.read_mail(&name);
        if mails.is_empty() {
            sender.send_message(format_text("&7No mail"));
            return Ok(1);
        }
        sender.send_message(format_text("&6Mail:"));
        for mail in mails {
            sender.send_message(format_text(&format!(
                "&e{}: &f{}",
                mail.sender, mail.message
            )));
        }
        Ok(1)
    }
}

// ---------------- broadcast ----------------

pub struct BroadcastHandler;

impl CommandHandler for BroadcastHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let message = arg_string(&args, "message").unwrap_or_default();
        server.broadcast(&format!("&6[Broadcast] &e{}", message));
        sender.send_message(format_text("&aBroadcast sent"));
        Ok(1)
    }
}

pub struct BroadcastWorldHandler;

impl CommandHandler for BroadcastWorldHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let message = arg_string(&args, "message").unwrap_or_default();
        server.broadcast(&format!("&6[Broadcast] &e{}", message));
        sender.send_message(format_text("&aBroadcast sent"));
        Ok(1)
    }
}

// ---------------- registration ----------------

pub fn register(chat: Arc<Chat>, context: &Context) {
    let msg = Command::new(&["msg".to_string(), "tell".to_string(), "w".to_string()], "Message a player")
        .then(string_arg("target").then(greedy_arg("message").execute(MsgHandler {
            chat: chat.clone(),
        })));
    context.register_command(msg, "essentials.msg");

    let reply = Command::new(&["r".to_string(), "reply".to_string()], "Reply to a message")
        .then(greedy_arg("message").execute(ReplyHandler {
            chat: chat.clone(),
        }));
    context.register_command(reply, "essentials.reply");

    let mail = Command::new(&["mail".to_string()], "Mail")
        .then(
            CommandNode::literal("send")
                .then(string_arg("target").then(greedy_arg("message").execute(MailSendHandler {
                    chat: chat.clone(),
                }))),
        )
        .then(CommandNode::literal("read").execute(MailReadHandler { chat: chat.clone() }));
    context.register_command(mail, "essentials.mail");

    let broadcast = Command::new(&["broadcast".to_string(), "bc".to_string()], "Broadcast a message")
        .then(greedy_arg("message").execute(BroadcastHandler));
    context.register_command(broadcast, "essentials.broadcast");

    let broadcastworld = Command::new(&["broadcastworld".to_string()], "Broadcast to world")
        .then(greedy_arg("message").execute(BroadcastWorldHandler));
    context.register_command(broadcastworld, "essentials.broadcastworld");
}