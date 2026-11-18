use pumpkin_plugin_api::{
    Context, Plugin, PluginMetadata, Result,
    permission::{Permission, PermissionDefault, PermissionLevel},
};
use std::sync::Arc;

mod economy;
mod permissions;
mod storage;
mod utils;

use economy::Economy;
use permissions::Permissions;
use storage::Storage;

pub struct UnifiedPlugin;

impl Plugin for UnifiedPlugin {
    fn new() -> Self {
        UnifiedPlugin
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "pumpkin-unified-core".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: vec!["SSIT2051".into()],
            description: "Unified EssentialsX + LuckPerms plugin for PumpkinMC".into(),
            dependencies: vec![],
            permissions: vec![],
        }
    }

    fn on_load(&self, context: Context) -> Result<()> {
        tracing::info!("Loading pumpkin-unified-core...");

        let data_folder = context.get_data_folder();
        let storage = Arc::new(Storage::new(&data_folder));
        let economy = Arc::new(Economy::new(storage.clone()));
        let permissions = Arc::new(Permissions::new(storage.clone()));

        permissions::commands::register(permissions.clone(), &context);
        economy::commands::register(economy.clone(), &context);

        register_permissions(&context)?;

        tracing::info!("pumpkin-unified-core loaded successfully");
        Ok(())
    }

    fn on_unload(&self, _context: Context) -> Result<()> {
        tracing::info!("Unloading pumpkin-unified-core...");
        Ok(())
    }
}

fn permission(node: &str, description: &str, default: PermissionDefault) -> Permission {
    Permission {
        node: node.to_string(),
        description: description.to_string(),
        default,
        children: Vec::new(),
    }
}

fn register_permissions(context: &Context) -> Result<()> {
    let allow = PermissionDefault::Allow;
    let op = || PermissionDefault::Op(PermissionLevel::Four);

    let perms = vec![
        permission("essentials.balance", "Check your balance", allow),
        permission("essentials.balance.others", "Check others' balances", allow),
        permission("essentials.pay", "Pay another player", allow),
        permission("essentials.balancetop", "Show top balances", allow),
        permission("essentials.eco", "Economy admin", op()),
        permission("essentials.eco.give", "Give money", op()),
        permission("essentials.eco.take", "Take money", op()),
        permission("essentials.eco.set", "Set balance", op()),
        permission("luckperms.admin", "LuckPerms administration", op()),
        permission("luckperms.user.info", "View user info", op()),
        permission("luckperms.user.permission", "Manage user permissions", op()),
        permission("luckperms.user.parent", "Manage user groups", op()),
        permission("luckperms.group.info", "View group info", op()),
        permission("luckperms.group.permission", "Manage group permissions", op()),
        permission("luckperms.group.parent", "Manage group inheritance", op()),
        permission("luckperms.track", "Manage tracks", op()),
        permission("luckperms.sync", "Sync permissions", op()),
        permission("luckperms.info", "Plugin info", op()),
    ];

    for perm in &perms {
        context.register_permission(perm)?;
    }
    Ok(())
}

pumpkin_plugin_api::register_plugin!(UnifiedPlugin);