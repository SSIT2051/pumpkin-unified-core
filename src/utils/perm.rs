//! Permission namespace helpers.
//!
//! PumpkinMC permission nodes use the format `namespace:node` with a COLON
//! separator (e.g. `minecraft:command.gamemode`, `pumpkin-unified-core:essentials.balance`).
//! The host rejects any registered permission that does not start with the
//! plugin's namespace followed by a colon:
//!
//! ```text
//! ERROR Failed to initialize plugin pumpkin-unified-core: Initialization failed:
//! Permission essentials.balance must use the plugin's namespace (pumpkin-unified-core)
//! ```
//!
//! These helpers let the rest of the codebase use short, LuckPerms-style nodes
//! (`essentials.balance`) while registration and lookup operate on the fully
//! namespaced form (`pumpkin-unified-core:essentials.balance`).

/// The plugin namespace enforced by PumpkinMC.
pub const PLUGIN_NAMESPACE: &str = "pumpkin-unified-core";

/// Returns `perm` prefixed with the plugin namespace, unless it already
/// carries a namespace (contains a `:`).
///
/// - `essentials.balance` -> `pumpkin-unified-core:essentials.balance`
/// - `pumpkin-unified-core:essentials.balance` -> unchanged
/// - `minecraft:command.gamemode` -> unchanged (foreign namespace)
pub fn namespaced(perm: &str) -> String {
    if perm.contains(':') {
        perm.to_string()
    } else {
        format!("{PLUGIN_NAMESPACE}:{perm}")
    }
}

/// Returns the shorthand form of `perm` (our namespace stripped), or `perm`
/// itself when it carries no namespace prefix.
///
/// - `pumpkin-unified-core:essentials.balance` -> `essentials.balance`
/// - `essentials.balance` -> `essentials.balance`
/// - `minecraft:command.gamemode` -> `minecraft:command.gamemode`
pub fn shorthand(perm: &str) -> &str {
    perm.strip_prefix(PLUGIN_NAMESPACE)
        .and_then(|rest| rest.strip_prefix(':'))
        .unwrap_or(perm)
}

/// True when `perm` starts with our namespace segment (`pumpkin-unified-core:`).
#[allow(dead_code)]
pub fn is_namespaced(perm: &str) -> bool {
    perm.strip_prefix(PLUGIN_NAMESPACE)
        .is_some_and(|rest| rest.starts_with(':'))
}

/// True when `stored` (a permission node as written by an admin, possibly
/// shorthand) grants `required` (a possibly namespaced node), considering both
/// the shorthand and namespaced forms plus wildcards.
///
/// Examples (required = `pumpkin-unified-core:essentials.balance`):
/// - stored `pumpkin-unified-core:essentials.balance` -> true (exact, full)
/// - stored `essentials.balance`                      -> true (exact, shorthand)
/// - stored `essentials.*`                            -> true (shorthand wildcard)
/// - stored `pumpkin-unified-core:essentials.*`       -> true (full wildcard)
/// - stored `pumpkin-unified-core:*`                  -> true (namespace wildcard)
/// - stored `*`                                       -> true (global wildcard)
/// - stored `essentials.home`                         -> false
pub fn node_matches(stored: &str, required: &str) -> bool {
    let req_full = namespaced(required);
    let req_short = shorthand(required);

    // Exact match in either form.
    if stored == required || stored == req_short || stored == req_full {
        return true;
    }

    // Bare "*" grants everything.
    if stored == "*" {
        return true;
    }

    // Namespace wildcard: "<ns>:*" grants everything under that namespace.
    if let Some(ns) = stored.strip_suffix(":*") {
        if req_full.starts_with(&format!("{ns}:")) {
            return true;
        }
    }

    // Dotted wildcard: "<prefix>.*" grants everything under that prefix,
    // matched against both the full and shorthand forms of `required`.
    if let Some(prefix) = stored.strip_suffix(".*") {
        if req_short.starts_with(&format!("{prefix}.")) || req_full.starts_with(&format!("{prefix}.")) {
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
        assert_eq!(namespaced("essentials.balance"), "pumpkin-unified-core:essentials.balance");
        assert_eq!(
            namespaced("pumpkin-unified-core:essentials.balance"),
            "pumpkin-unified-core:essentials.balance"
        );
        assert_eq!(namespaced("luckperms.admin"), "pumpkin-unified-core:luckperms.admin");
        // Foreign namespaces pass through untouched.
        assert_eq!(namespaced("minecraft:command.gamemode"), "minecraft:command.gamemode");
    }

    #[test]
    fn test_shorthand() {
        assert_eq!(shorthand("pumpkin-unified-core:essentials.balance"), "essentials.balance");
        assert_eq!(shorthand("essentials.balance"), "essentials.balance");
    }

    #[test]
    fn test_is_namespaced() {
        assert!(is_namespaced("pumpkin-unified-core:essentials.balance"));
        assert!(!is_namespaced("essentials.balance"));
        assert!(!is_namespaced("pumpkin-unified-core.essentials.balance"));
    }

    #[test]
    fn test_node_matches() {
        let req = "pumpkin-unified-core:essentials.balance";
        assert!(node_matches("pumpkin-unified-core:essentials.balance", req));
        assert!(node_matches("essentials.balance", req));
        assert!(node_matches("essentials.*", req));
        assert!(node_matches("pumpkin-unified-core:essentials.*", req));
        assert!(node_matches("pumpkin-unified-core:*", req));
        assert!(node_matches("*", req));
        assert!(!node_matches("essentials.home", req));
        assert!(!node_matches("pumpkin-unified-core:essentials.home", req));
        assert!(!node_matches("luckperms.*", req));
    }
}