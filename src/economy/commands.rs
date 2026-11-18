use crate::economy::Economy;
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

/// Extracts a string argument (single-word or greedy) from the consumed args.
fn arg_string(args: &ConsumedArgs, key: &str) -> Option<String> {
    match args.get_value(key) {
        Arg::Simple(s) | Arg::Msg(s) => Some(s),
        _ => None,
    }
}

pub struct BalanceHandler {
    econ: Arc<Economy>,
}

impl CommandHandler for BalanceHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        match sender.as_player() {
            Some(player) => {
                let balance = self.econ.get_balance(&player.get_id().to_string());
                sender.send_message(format_text(&format!(
                    "&aYour balance: {}",
                    self.econ.format_money(balance)
                )));
            }
            None => sender.send_message(format_text("&cOnly players have a balance")),
        }
        Ok(1)
    }
}

pub struct BalanceOthersHandler {
    econ: Arc<Economy>,
}

impl CommandHandler for BalanceOthersHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target_name = arg_string(&args, "target").unwrap_or_default();
        match server.get_player_by_name(&target_name) {
            Some(player) => {
                let balance = self.econ.get_balance(&player.get_id().to_string());
                sender.send_message(format_text(&format!(
                    "&a{}'s balance: {}",
                    player.get_name(),
                    self.econ.format_money(balance)
                )));
            }
            None => sender.send_message(format_text(&format!(
                "&cPlayer not found: {target_name}"
            ))),
        }
        Ok(1)
    }
}

pub struct PayHandler {
    econ: Arc<Economy>,
}

impl CommandHandler for PayHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target_name = arg_string(&args, "target").unwrap_or_default();
        let amount: f64 = arg_string(&args, "amount")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);

        let from = match sender.as_player() {
            Some(p) => p,
            None => {
                sender.send_message(format_text("&cOnly players can pay"));
                return Ok(1);
            }
        };
        let to = match server.get_player_by_name(&target_name) {
            Some(p) => p,
            None => {
                sender.send_message(format_text(&format!(
                    "&cPlayer not found: {target_name}"
                )));
                return Ok(1);
            }
        };

        match self.econ.pay(
            &from.get_id().to_string(),
            &from.get_name(),
            &to.get_id().to_string(),
            &to.get_name(),
            amount,
        ) {
            Ok(_) => {
                sender.send_message(format_text(&format!(
                    "&aYou paid {} {}",
                    to.get_name(),
                    self.econ.format_money(amount)
                )));
                to.send_system_message(
                    format_text(&format!(
                        "&aYou received {} from {}",
                        self.econ.format_money(amount),
                        from.get_name()
                    )),
                    false,
                );
            }
            Err(e) => sender.send_message(format_text(&format!("&c{e}"))),
        }
        Ok(1)
    }
}

pub struct BaltopHandler {
    econ: Arc<Economy>,
}

impl CommandHandler for BaltopHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        sender.send_message(format_text("&6&l--- Balance Top ---"));
        for (i, account) in self.econ.top_balances(10).iter().enumerate() {
            sender.send_message(format_text(&format!(
                "&e{}. &a{} &7- {}",
                i + 1,
                account.name,
                self.econ.format_money(account.balance)
            )));
        }
        Ok(1)
    }
}

pub struct EcoGiveHandler {
    econ: Arc<Economy>,
}

impl CommandHandler for EcoGiveHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target_name = arg_string(&args, "target").unwrap_or_default();
        let amount: f64 = arg_string(&args, "amount")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        match server.get_player_by_name(&target_name) {
            Some(player) => {
                self.econ.add_balance(&player.get_id().to_string(), &player.get_name(), amount);
                sender.send_message(format_text(&format!(
                    "&aGave {} to {}",
                    self.econ.format_money(amount),
                    player.get_name()
                )));
            }
            None => sender.send_message(format_text(&format!(
                "&cPlayer not found: {target_name}"
            ))),
        }
        Ok(1)
    }
}

pub struct EcoTakeHandler {
    econ: Arc<Economy>,
}

impl CommandHandler for EcoTakeHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target_name = arg_string(&args, "target").unwrap_or_default();
        let amount: f64 = arg_string(&args, "amount")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        match server.get_player_by_name(&target_name) {
            Some(player) => {
                self.econ.take_balance(&player.get_id().to_string(), &player.get_name(), amount);
                sender.send_message(format_text(&format!(
                    "&aTook {} from {}",
                    self.econ.format_money(amount),
                    player.get_name()
                )));
            }
            None => sender.send_message(format_text(&format!(
                "&cPlayer not found: {target_name}"
            ))),
        }
        Ok(1)
    }
}

pub struct EcoSetHandler {
    econ: Arc<Economy>,
}

impl CommandHandler for EcoSetHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target_name = arg_string(&args, "target").unwrap_or_default();
        let amount: f64 = arg_string(&args, "amount")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        match server.get_player_by_name(&target_name) {
            Some(player) => {
                self.econ.set_balance(&player.get_id().to_string(), &player.get_name(), amount);
                sender.send_message(format_text(&format!(
                    "&aSet {}'s balance to {}",
                    player.get_name(),
                    self.econ.format_money(amount)
                )));
            }
            None => sender.send_message(format_text(&format!(
                "&cPlayer not found: {target_name}"
            ))),
        }
        Ok(1)
    }
}

pub struct EcoUsageHandler;

impl CommandHandler for EcoUsageHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        sender.send_message(format_text(
            "&6/economy <give|take|set> <player> <amount>",
        ));
        Ok(1)
    }
}

pub struct PayUsageHandler;

impl CommandHandler for PayUsageHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        sender.send_message(format_text("&6/pay <player> <amount>"));
        Ok(1)
    }
}

fn string_arg(name: &str) -> CommandNode {
    CommandNode::argument(name, &ArgumentType::String(StringType::SingleWord))
}

pub fn register(econ: Arc<Economy>, context: &Context) {
    // /balance [target]
    let balance = Command::new(
        &[
            "balance".to_string(),
            "bal".to_string(),
            "money".to_string(),
        ],
        "Check your balance",
    )
    .execute(BalanceHandler {
        econ: econ.clone(),
    })
    .then(string_arg("target").execute(BalanceOthersHandler {
        econ: econ.clone(),
    }));
    context.register_command(balance, "pumpkin-unified-core:essentials.balance");

    // /pay <target> <amount>
    let pay = Command::new(&["pay".to_string()], "Pay another player")
        .execute(PayUsageHandler)
        .then(
            string_arg("target")
                .then(string_arg("amount").execute(PayHandler { econ: econ.clone() })),
        );
    context.register_command(pay, "pumpkin-unified-core:essentials.pay");

    // /baltop
    let baltop = Command::new(
        &["baltop".to_string(), "balancetop".to_string()],
        "Show top balances",
    )
    .execute(BaltopHandler {
        econ: econ.clone(),
    });
    context.register_command(baltop, "pumpkin-unified-core:essentials.balancetop");

    // /eco <give|take|set> <target> <amount>
    let eco = Command::new(
        &["eco".to_string(), "economy".to_string()],
        "Economy admin commands",
    )
    .execute(EcoUsageHandler)
    .then(
        CommandNode::literal("give").then(
            string_arg("target")
                .then(string_arg("amount").execute(EcoGiveHandler { econ: econ.clone() })),
        ),
    )
    .then(
        CommandNode::literal("take").then(
            string_arg("target")
                .then(string_arg("amount").execute(EcoTakeHandler { econ: econ.clone() })),
        ),
    )
    .then(
        CommandNode::literal("set").then(
            string_arg("target")
                .then(string_arg("amount").execute(EcoSetHandler { econ: econ.clone() })),
        ),
    );
    context.register_command(eco, "pumpkin-unified-core:essentials.eco");
}