//! LuckPerms-compatible permission engine.
//!
//! Stores per-user and per-group permission nodes plus group inheritance, and
//! resolves effective permissions with wildcard matching. Data is persisted via
//! the file-based [`Storage`].

use crate::config::PermissionsConfig;
use crate::storage::Storage;
use std::collections::HashSet;
use std::sync::Arc;

pub mod commands;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PermissionNode {
    pub node: String,
    pub value: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserData {
    pub uuid: String,
    pub name: String,
    #[serde(default)]
    pub permissions: Vec<PermissionNode>,
    #[serde(default)]
    pub groups: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GroupData {
    pub name: String,
    #[serde(default)]
    pub permissions: Vec<PermissionNode>,
    #[serde(default)]
    pub parents: Vec<String>,
    #[serde(default)]
    pub weight: i32,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(default)]
    pub suffix: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrackData {
    pub name: String,
    #[serde(default)]
    pub groups: Vec<String>,
}

pub struct Permissions {
    storage: Arc<Storage>,
    config: PermissionsConfig,
    default_group: String,
}

impl Permissions {
    pub fn new(storage: Arc<Storage>, config: PermissionsConfig) -> Self {
        let perms = Self {
            storage,
            config: config.clone(),
            default_group: config.default_group.clone(),
        };
        perms.ensure_default_group();
        perms
    }

    fn ensure_default_group(&self) {
        if self.get_group(&self.default_group).is_none() {
            self.create_group(&self.default_group);
        }
    }

    // ----- users -----

    pub fn get_user(&self, uuid: &str) -> Option<UserData> {
        self.storage.get("users", uuid)
    }

    pub fn find_user_by_name(&self, name: &str) -> Option<UserData> {
        for key in self.storage.keys("users") {
            if let Some(user) = self.storage.get::<UserData>("users", &key) {
                if user.name.eq_ignore_ascii_case(name) {
                    return Some(user);
                }
            }
        }
        None
    }

    pub fn save_user(&self, user: &UserData) {
        self.storage.set("users", &user.uuid, user);
    }

    pub fn set_user_permission(&self, uuid: &str, name: &str, node: String, value: bool) {
        let mut user = self.get_user(uuid).unwrap_or(UserData {
            uuid: uuid.to_string(),
            name: name.to_string(),
            permissions: Vec::new(),
            groups: Vec::new(),
        });
        user.name = name.to_string();
        user.permissions.retain(|p| p.node != node);
        user.permissions.push(PermissionNode { node, value });
        self.save_user(&user);
    }

    pub fn unset_user_permission(&self, uuid: &str, node: &str) {
        if let Some(mut user) = self.get_user(uuid) {
            user.permissions.retain(|p| p.node != node);
            self.save_user(&user);
        }
    }

    pub fn add_user_group(&self, uuid: &str, name: &str, group: String) {
        let mut user = self.get_user(uuid).unwrap_or(UserData {
            uuid: uuid.to_string(),
            name: name.to_string(),
            permissions: Vec::new(),
            groups: Vec::new(),
        });
        user.name = name.to_string();
        if !user.groups.contains(&group) {
            user.groups.push(group);
            self.save_user(&user);
        }
    }

    pub fn remove_user_group(&self, uuid: &str, group: &str) {
        if let Some(mut user) = self.get_user(uuid) {
            user.groups.retain(|g| g != group);
            self.save_user(&user);
        }
    }

    // ----- groups -----

    pub fn get_group(&self, name: &str) -> Option<GroupData> {
        self.storage.get("groups", name)
    }

    pub fn create_group(&self, name: &str) {
        if self.get_group(name).is_none() {
            let group = GroupData {
                name: name.to_string(),
                permissions: Vec::new(),
                parents: Vec::new(),
                weight: 0,
                display_name: None,
                prefix: None,
                suffix: None,
            };
            self.storage.set("groups", name, &group);
        }
    }

    pub fn delete_group(&self, name: &str) {
        self.storage.delete("groups", name);
    }

    pub fn list_groups(&self) -> Vec<GroupData> {
        self.storage.values("groups")
    }

    pub fn set_group_permission(&self, name: &str, node: String, value: bool) {
        if let Some(mut group) = self.get_group(name) {
            group.permissions.retain(|p| p.node != node);
            group.permissions.push(PermissionNode { node, value });
            self.storage.set("groups", name, &group);
        }
    }

    pub fn unset_group_permission(&self, name: &str, node: &str) {
        if let Some(mut group) = self.get_group(name) {
            group.permissions.retain(|p| p.node != node);
            self.storage.set("groups", name, &group);
        }
    }

    pub fn add_group_parent(&self, name: &str, parent: String) {
        if let Some(mut group) = self.get_group(name) {
            if !group.parents.contains(&parent) {
                group.parents.push(parent);
                self.storage.set("groups", name, &group);
            }
        }
    }

    pub fn remove_group_parent(&self, name: &str, parent: &str) {
        if let Some(mut group) = self.get_group(name) {
            group.parents.retain(|p| p != parent);
            self.storage.set("groups", name, &group);
        }
    }

    pub fn set_group_weight(&self, name: &str, weight: i32) {
        if let Some(mut group) = self.get_group(name) {
            group.weight = weight;
            self.storage.set("groups", name, &group);
        }
    }

    pub fn set_group_display_name(&self, name: &str, display_name: String) {
        if let Some(mut group) = self.get_group(name) {
            group.display_name = Some(display_name);
            self.storage.set("groups", name, &group);
        }
    }

    pub fn set_group_prefix(&self, name: &str, prefix: String) {
        if let Some(mut group) = self.get_group(name) {
            group.prefix = Some(prefix);
            self.storage.set("groups", name, &group);
        }
    }

    pub fn set_group_suffix(&self, name: &str, suffix: String) {
        if let Some(mut group) = self.get_group(name) {
            group.suffix = Some(suffix);
            self.storage.set("groups", name, &group);
        }
    }

    // ----- tracks -----

    pub fn create_track(&self, name: &str) {
        if self.get_track(name).is_none() {
            let track = TrackData {
                name: name.to_string(),
                groups: Vec::new(),
            };
            self.storage.set("tracks", name, &track);
        }
    }

    pub fn get_track(&self, name: &str) -> Option<TrackData> {
        self.storage.get("tracks", name)
    }

    pub fn delete_track(&self, name: &str) {
        self.storage.delete("tracks", name);
    }

    pub fn list_tracks(&self) -> Vec<TrackData> {
        self.storage.values("tracks")
    }

    pub fn append_track(&self, name: &str, group: String) {
        if let Some(mut track) = self.get_track(name) {
            if !track.groups.contains(&group) {
                track.groups.push(group);
                self.storage.set("tracks", name, &track);
            }
        }
    }

    // ----- resolution -----

    fn collect_group_permissions(
        &self,
        name: &str,
        out: &mut Vec<PermissionNode>,
        visited: &mut HashSet<String>,
    ) {
        if !visited.insert(name.to_string()) {
            return;
        }
        if let Some(group) = self.get_group(name) {
            out.extend(group.permissions.iter().cloned());
            for parent in &group.parents {
                self.collect_group_permissions(parent, out, visited);
            }
        }
    }

    /// Resolves whether `uuid` has `node`, considering group inheritance and
    /// wildcards. `node` may be given in the fully namespaced form
    /// (`pumpkin-unified-core:essentials.balance`) or the shorthand form
    /// (`essentials.balance`); stored nodes may use either form as well.
    /// Exact matches override wildcard matches; user nodes override group nodes.
    pub fn has_permission(&self, uuid: &str, node: &str) -> bool {
        let user = match self.get_user(uuid) {
            Some(u) => u,
            None => return false,
        };

        let mut perms: Vec<PermissionNode> = Vec::new();
        let mut visited = HashSet::new();
        for group in &user.groups {
            self.collect_group_permissions(group, &mut perms, &mut visited);
        }
        // User nodes are appended last so they win over group nodes.
        perms.extend(user.permissions.iter().cloned());

        let node_short = crate::utils::perm::shorthand(node);

        let mut exact: Option<bool> = None;
        let mut wildcard: Option<bool> = None;
        for perm in &perms {
            // Check both namespaced and shorthand forms for exact match
            if perm.node == node || perm.node == node_short {
                exact = Some(perm.value);
            } else if crate::utils::perm::node_matches(&perm.node, node) ||
                      crate::utils::perm::node_matches(&perm.node, node_short) {
                wildcard = Some(perm.value);
            }
        }

        exact.or(wildcard).unwrap_or(false)
    }

    // ----- meta (prefix/suffix) -----

    pub fn get_prefix(&self, uuid: &str) -> Option<String> {
        let user = self.get_user(uuid)?;
        for group in &user.groups {
            if let Some(group_data) = self.get_group(group) {
                if let Some(prefix) = group_data.prefix {
                    return Some(prefix);
                }
            }
        }
        None
    }

    pub fn get_suffix(&self, uuid: &str) -> Option<String> {
        let user = self.get_user(uuid)?;
        for group in &user.groups {
            if let Some(group_data) = self.get_group(group) {
                if let Some(suffix) = group_data.suffix {
                    return Some(suffix);
                }
            }
        }
        None
    }

    pub fn get_primary_group(&self, uuid: &str) -> Option<String> {
        let user = self.get_user(uuid)?;
        user.groups.first().cloned()
    }
}