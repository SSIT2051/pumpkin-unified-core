//! Permission namespace helpers.
//!
//! PumpkinMC requires every permission a plugin registers to be prefixed with
//! the plugin's own namespace (`pumpkin-unified-core.`). These helpers keep the
//! rest of the codebase readable by letting call sites use short, LuckPerms-style
//! nodes (`essentials.balance`) while registration and lookup operate on the
//! fully namespaced form.

/// The plugin namespace enforced by PumpkinMC.
pub const PLUGIN_NAMESPACE: &str = "pumpkin-unified-core";

/// Returns `perm` prefixed with the plugin namespace, unless it already is.
///
/// - `essentials.balance` -> `pumpkin-unified-core.essentials.balance`
/// - `pumpkin-unified-core.essentials.balance` -> unchanged
pub fn namespaced(perm: &str) -> String {
    if is_namespaced(perm) {
        perm.to_string()
    } else {
        format!("{PLUGIN_NAMESPACE}.{perm}")
    }
}

/// Returns the shorthand form of `perm` (namespace stripped), or `perm` itself
/// when it carries no namespace prefix.
///
/// - `pumpkin-unified-core.essentials.balance` -> `essentials.balance`
/// - `essentials.balance` -> `essentials.balance`
pub fn shorthand(perm: &str) -> &str {
    if is_namespaced(perm) {
        perm[PLUGIN_NAMESPACE.len() + 1..].trim()
    } else {
        perm
    }
}

/// True when `perm` already starts with the plugin namespace segment.
pub fn is_namespaced(perm: &str) -> bool {
    perm.strip_prefix(PLUGIN_NAMESPACE)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('.'))
}

/// True when `stored` (a permission node as written by an admin, possibly
/// shorthand) grants `required` (a possibly namespaced node), considering both
/// the shorthand and namespaced forms plus trailing `.*` wildcards.
///
/// Examples (required = `pumpkin-unified-core.essentials.balance`):
/// - stored `pumpkin-unified-core.essentials.balance` -> true (exact)
/// - stored `essentials.balance`                      -> true (shorthand exact)
/// - stored `essentials.*`                            -> true (shorthand wildcard)
/// - stored `pumpkin-unified-core.*`                  -> true (namespace wildcard)
/// - stored `*`                                       -> true (global wildcard)
/// - stored `essentials.home`                         -> false
pub fn node_matches(stored: &str, required: &str) -> bool {
    let required_short = shorthand(required);

    // Exact match in either form.
    if stored == required || stored == required_short {
        return true;
    }

    // A bare "*" grants everything.
    if stored == "*" {
        return true;
    }

    // Wildcard match: a trailing ".*" matches any node sharing the prefix.
    if let Some(prefix) = stored.strip_suffix(".*") {
        if required.starts_with(prefix) || required_short.starts_with(prefix) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespaced() {
        assert_eq!(namespaced("essentials.balance"), "pumpkin-unified-core.essentials.balance");
        assert_eq!(
            namespaced("pumpkin-unified-core.essentials.balance"),
            "pumpkin-unified-core.essentials.balance"
        );
        assert_eq!(namespaced("luckperms.admin"), "pumpkin-unified-core.luckperms.admin");
    }

    #[test]
    fn test_shorthand() {
        assert_eq!(shorthand("pumpkin-unified-core.essentials.balance"), "essentials.balance");
        assert_eq!(shorthand("essentials.balance"), "essentials.balance");
    }

    #[test]
    fn test_node_matches() {
        let req = "pumpkin-unified-core.essentials.balance";
        assert!(node_matches("pumpkin-unified-core.essentials.balance", req));
        assert!(node_matches("essentials.balance", req));
        assert!(node_matches("essentials.*", req));
        assert!(node_matches("pumpkin-unified-core.*", req));
        assert!(node_matches("*", req));
        assert!(node_matches("pumpkin-unified-core.essentials.*", req));
        assert!(!node_matches("essentials.home", req));
        assert!(!node_matches("pumpkin-unified-core.essentials.home", req));
    }
}