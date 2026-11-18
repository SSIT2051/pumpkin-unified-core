use crate::permissions::Permissions;
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

fn arg_bool(args: &ConsumedArgs, key: &str) -> bool {
    arg_string(args, key)
        .map(|s| !matches!(s.as_str(), "false" | "0" | "no"))
        .unwrap_or(true)
}

/// Resolves a player name to `(uuid, canonical_name)`, preferring online players
/// and falling back to stored user data for offline players.
fn resolve_target(server: &Server, perms: &Permissions, name: &str) -> Option<(String, String)> {
    if let Some(player) = server.get_player_by_name(name) {
        return Some((player.get_id().to_string(), player.get_name()));
    }
    perms.find_user_by_name(name).map(|u| (u.uuid, u.name))
}

// ---------------- root ----------------

pub struct LpHelpHandler;

impl CommandHandler for LpHelpHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        sender.send_message(format_text("&6&lLuckPerms"));
        sender.send_message(format_text(
            "&e/lp user <name> info|permission set|parent add",
        ));
        sender.send_message(format_text(
            "&e/lp group list|<name> info|permission set|parent add",
        ));
        Ok(1)
    }
}

pub struct InfoHandler;

impl CommandHandler for InfoHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        sender.send_message(format_text(
            "&6LuckPerms &7- Rust port for PumpkinMC",
        ));
        Ok(1)
    }
}

pub struct SyncHandler;

impl CommandHandler for SyncHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        sender.send_message(format_text("&aPermissions synced"));
        Ok(1)
    }
}

// ---------------- user ----------------

pub struct UserInfoHandler {
    perms: Arc<Permissions>,
}

impl CommandHandler for UserInfoHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target = arg_string(&args, "target").unwrap_or_default();
        match resolve_target(&server, &self.perms, &target) {
            Some((uuid, name)) => {
                sender.send_message(format_text(&format!("&6User: &f{name} &7({uuid})")));
                match self.perms.get_user(&uuid) {
                    Some(user) => {
                        let groups = if user.groups.is_empty() {
                            "none".to_string()
                        } else {
                            user.groups.join(", ")
                        };
                        sender.send_message(format_text(&format!("&7Groups: &f{groups}")));
                        sender.send_message(format_text("&7Permissions:"));
                        for p in &user.permissions {
                            sender.send_message(format_text(&format!(
                                "  &8- &f{} &7=> &f{}",
                                p.node, p.value
                            )));
                        }
                    }
                    None => sender.send_message(format_text("&7No stored data (defaults apply)")),
                }
            }
            None => sender.send_message(format_text(&format!("&cUser not found: {target}"))),
        }
        Ok(1)
    }
}

pub struct UserPermSetHandler {
    perms: Arc<Permissions>,
}

impl CommandHandler for UserPermSetHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target = arg_string(&args, "target").unwrap_or_default();
        let node = arg_string(&args, "node").unwrap_or_default();
        let value = arg_bool(&args, "value");
        match resolve_target(&server, &self.perms, &target) {
            Some((uuid, name)) => {
                self.perms.set_user_permission(&uuid, &name, node.clone(), value);
                sender.send_message(format_text(&format!(
                    "&aSet &f{node} &a=> &f{value} &afor &f{name}"
                )));
            }
            None => sender.send_message(format_text(&format!("&cUser not found: {target}"))),
        }
        Ok(1)
    }
}

pub struct UserPermUnsetHandler {
    perms: Arc<Permissions>,
}

impl CommandHandler for UserPermUnsetHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target = arg_string(&args, "target").unwrap_or_default();
        let node = arg_string(&args, "node").unwrap_or_default();
        match resolve_target(&server, &self.perms, &target) {
            Some((uuid, name)) => {
                self.perms.unset_user_permission(&uuid, &node);
                sender.send_message(format_text(&format!(
                    "&aUnset &f{node} &afrom &f{name}"
                )));
            }
            None => sender.send_message(format_text(&format!("&cUser not found: {target}"))),
        }
        Ok(1)
    }
}

pub struct UserParentAddHandler {
    perms: Arc<Permissions>,
}

impl CommandHandler for UserParentAddHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target = arg_string(&args, "target").unwrap_or_default();
        let group = arg_string(&args, "group").unwrap_or_default();
        match resolve_target(&server, &self.perms, &target) {
            Some((uuid, name)) => {
                self.perms.add_user_group(&uuid, &name, group.clone());
                sender.send_message(format_text(&format!(
                    "&aAdded group &f{group} &ato &f{name}"
                )));
            }
            None => sender.send_message(format_text(&format!("&cUser not found: {target}"))),
        }
        Ok(1)
    }
}

pub struct UserParentRemoveHandler {
    perms: Arc<Permissions>,
}

impl CommandHandler for UserParentRemoveHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target = arg_string(&args, "target").unwrap_or_default();
        let group = arg_string(&args, "group").unwrap_or_default();
        match resolve_target(&server, &self.perms, &target) {
            Some((uuid, name)) => {
                self.perms.remove_user_group(&uuid, &group);
                sender.send_message(format_text(&format!(
                    "&aRemoved group &f{group} &afrom &f{name}"
                )));
            }
            None => sender.send_message(format_text(&format!("&cUser not found: {target}"))),
        }
        Ok(1)
    }
}

// ---------------- group ----------------

pub struct GroupListHandler {
    perms: Arc<Permissions>,
}

impl CommandHandler for GroupListHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        sender.send_message(format_text("&6Groups:"));
        for group in self.perms.list_groups() {
            sender.send_message(format_text(&format!(
                "&e{} &7(weight {})",
                group.name, group.weight
            )));
        }
        Ok(1)
    }
}

pub struct GroupInfoHandler {
    perms: Arc<Permissions>,
}

impl CommandHandler for GroupInfoHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "name").unwrap_or_default();
        match self.perms.get_group(&name) {
            Some(group) => {
                sender.send_message(format_text(&format!(
                    "&6Group: &f{} &7(weight {})",
                    group.name, group.weight
                )));
                let parents = if group.parents.is_empty() {
                    "none".to_string()
                } else {
                    group.parents.join(", ")
                };
                sender.send_message(format_text(&format!("&7Parents: &f{parents}")));
                sender.send_message(format_text("&7Permissions:"));
                for p in &group.permissions {
                    sender.send_message(format_text(&format!(
                        "  &8- &f{} &7=> &f{}",
                        p.node, p.value
                    )));
                }
            }
            None => sender.send_message(format_text(&format!("&cGroup not found: {name}"))),
        }
        Ok(1)
    }
}

pub struct GroupPermSetHandler {
    perms: Arc<Permissions>,
}

impl CommandHandler for GroupPermSetHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "name").unwrap_or_default();
        let node = arg_string(&args, "node").unwrap_or_default();
        let value = arg_bool(&args, "value");
        if self.perms.get_group(&name).is_none() {
            sender.send_message(format_text(&format!("&cGroup not found: {name}")));
            return Ok(1);
        }
        self.perms.set_group_permission(&name, node.clone(), value);
        sender.send_message(format_text(&format!(
            "&aSet &f{node} &a=> &f{value} &afor group &f{name}"
        )));
        Ok(1)
    }
}

pub struct GroupPermUnsetHandler {
    perms: Arc<Permissions>,
}

impl CommandHandler for GroupPermUnsetHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "name").unwrap_or_default();
        let node = arg_string(&args, "node").unwrap_or_default();
        if self.perms.get_group(&name).is_none() {
            sender.send_message(format_text(&format!("&cGroup not found: {name}")));
            return Ok(1);
        }
        self.perms.unset_group_permission(&name, &node);
        sender.send_message(format_text(&format!(
            "&aUnset &f{node} &afrom group &f{name}"
        )));
        Ok(1)
    }
}

pub struct GroupParentAddHandler {
    perms: Arc<Permissions>,
}

impl CommandHandler for GroupParentAddHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "name").unwrap_or_default();
        let parent = arg_string(&args, "parent").unwrap_or_default();
        if self.perms.get_group(&name).is_none() {
            sender.send_message(format_text(&format!("&cGroup not found: {name}")));
            return Ok(1);
        }
        self.perms.add_group_parent(&name, parent.clone());
        sender.send_message(format_text(&format!(
            "&aAdded parent &f{parent} &ato group &f{name}"
        )));
        Ok(1)
    }
}

pub struct GroupParentRemoveHandler {
    perms: Arc<Permissions>,
}

impl CommandHandler for GroupParentRemoveHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "name").unwrap_or_default();
        let parent = arg_string(&args, "parent").unwrap_or_default();
        if self.perms.get_group(&name).is_none() {
            sender.send_message(format_text(&format!("&cGroup not found: {name}")));
            return Ok(1);
        }
        self.perms.remove_group_parent(&name, &parent);
        sender.send_message(format_text(&format!(
            "&aRemoved parent &f{parent} &afrom group &f{name}"
        )));
        Ok(1)
    }
}

pub struct GroupSetWeightHandler {
    perms: Arc<Permissions>,
}

impl CommandHandler for GroupSetWeightHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "name").unwrap_or_default();
        let weight: i32 = arg_string(&args, "weight")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        if self.perms.get_group(&name).is_none() {
            sender.send_message(format_text(&format!("&cGroup not found: {name}")));
            return Ok(1);
        }
        self.perms.set_group_weight(&name, weight);
        sender.send_message(format_text(&format!(
            "&aSet weight of &f{name} &ato &f{weight}"
        )));
        Ok(1)
    }
}

pub struct GroupSetDisplayHandler {
    perms: Arc<Permissions>,
}

impl CommandHandler for GroupSetDisplayHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "name").unwrap_or_default();
        let display = arg_string(&args, "display").unwrap_or_default();
        if self.perms.get_group(&name).is_none() {
            sender.send_message(format_text(&format!("&cGroup not found: {name}")));
            return Ok(1);
        }
        self.perms.set_group_display_name(&name, display.clone());
        sender.send_message(format_text(&format!(
            "&aSet display name of &f{name} &ato &f{display}"
        )));
        Ok(1)
    }
}

// ---------------- track ----------------

pub struct TrackCreateHandler {
    perms: Arc<Permissions>,
}

impl CommandHandler for TrackCreateHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "name").unwrap_or_default();
        self.perms.create_track(&name);
        sender.send_message(format_text(&format!("&aCreated track &f{name}")));
        Ok(1)
    }
}

pub struct TrackDeleteHandler {
    perms: Arc<Permissions>,
}

impl CommandHandler for TrackDeleteHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "name").unwrap_or_default();
        self.perms.delete_track(&name);
        sender.send_message(format_text(&format!("&aDeleted track &f{name}")));
        Ok(1)
    }
}

pub struct TrackListHandler {
    perms: Arc<Permissions>,
}

impl CommandHandler for TrackListHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        sender.send_message(format_text("&6Tracks:"));
        for track in self.perms.list_tracks() {
            sender.send_message(format_text(&format!("&e{}", track.name)));
        }
        Ok(1)
    }
}

pub struct TrackAppendHandler {
    perms: Arc<Permissions>,
}

impl CommandHandler for TrackAppendHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = arg_string(&args, "name").unwrap_or_default();
        let group = arg_string(&args, "group").unwrap_or_default();
        if self.perms.get_track(&name).is_none() {
            sender.send_message(format_text(&format!("&cTrack not found: {name}")));
            return Ok(1);
        }
        self.perms.append_track(&name, group.clone());
        sender.send_message(format_text(&format!(
            "&aAppended &f{group} &ato track &f{name}"
        )));
        Ok(1)
    }
}

// ---------------- registration ----------------

fn build_user_node(perms: Arc<Permissions>) -> CommandNode {
    let set = CommandNode::literal("set").then(
        string_arg("node").then(string_arg("value").execute(UserPermSetHandler {
            perms: perms.clone(),
        })),
    );
    let unset = CommandNode::literal("unset")
        .then(string_arg("node").execute(UserPermUnsetHandler {
            perms: perms.clone(),
        }));
    let perm = CommandNode::literal("permission").then(set).then(unset);

    let add = CommandNode::literal("add").then(string_arg("group").execute(UserParentAddHandler {
        perms: perms.clone(),
    }));
    let remove = CommandNode::literal("remove").then(string_arg("group").execute(
        UserParentRemoveHandler {
            perms: perms.clone(),
        },
    ));
    let parent = CommandNode::literal("parent").then(add).then(remove);

    let target = string_arg("target")
        .then(perm)
        .then(parent)
        .then(CommandNode::literal("info").execute(UserInfoHandler {
            perms: perms.clone(),
        }));

    CommandNode::literal("user").then(target)
}

fn build_group_node(perms: Arc<Permissions>) -> CommandNode {
    let list = CommandNode::literal("list").execute(GroupListHandler {
        perms: perms.clone(),
    });

    let info = CommandNode::literal("info").execute(GroupInfoHandler {
        perms: perms.clone(),
    });

    let set = CommandNode::literal("set").then(
        string_arg("node").then(string_arg("value").execute(GroupPermSetHandler {
            perms: perms.clone(),
        })),
    );
    let unset = CommandNode::literal("unset")
        .then(string_arg("node").execute(GroupPermUnsetHandler {
            perms: perms.clone(),
        }));
    let perm = CommandNode::literal("permission").then(set).then(unset);

    let add = CommandNode::literal("add").then(string_arg("parent").execute(
        GroupParentAddHandler {
            perms: perms.clone(),
        },
    ));
    let remove = CommandNode::literal("remove").then(string_arg("parent").execute(
        GroupParentRemoveHandler {
            perms: perms.clone(),
        },
    ));
    let parent = CommandNode::literal("parent").then(add).then(remove);

    let setweight = CommandNode::literal("setweight").then(string_arg("weight").execute(
        GroupSetWeightHandler {
            perms: perms.clone(),
        },
    ));
    let setdisplayname = CommandNode::literal("setdisplayname").then(string_arg("display").execute(
        GroupSetDisplayHandler {
            perms: perms.clone(),
        },
    ));

    let name = string_arg("name")
        .then(info)
        .then(perm)
        .then(parent)
        .then(setweight)
        .then(setdisplayname);

    CommandNode::literal("group").then(list).then(name)
}

fn build_track_node(perms: Arc<Permissions>) -> CommandNode {
    let list = CommandNode::literal("list").execute(TrackListHandler {
        perms: perms.clone(),
    });

    let createtrack = CommandNode::literal("createtrack")
        .then(string_arg("name").execute(TrackCreateHandler {
            perms: perms.clone(),
        }));
    let deletetrack = CommandNode::literal("deletetrack")
        .then(string_arg("name").execute(TrackDeleteHandler {
            perms: perms.clone(),
        }));
    let append = CommandNode::literal("append").then(string_arg("group").execute(
        TrackAppendHandler {
            perms: perms.clone(),
        },
    ));

    let name = string_arg("name").then(append);

    CommandNode::literal("track")
        .then(list)
        .then(createtrack)
        .then(deletetrack)
        .then(name)
}

pub fn register(perms: Arc<Permissions>, context: &Context) {
    let command = Command::new(&["lp".to_string()], "LuckPerms commands")
        .execute(LpHelpHandler)
        .then(build_user_node(perms.clone()))
        .then(build_group_node(perms.clone()))
        .then(build_track_node(perms.clone()))
        .then(CommandNode::literal("sync").execute(SyncHandler))
        .then(CommandNode::literal("info").execute(InfoHandler));

    context.register_command(command, "luckperms.admin");
}