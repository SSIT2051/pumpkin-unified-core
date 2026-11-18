//! Offline player UUID resolution with Mojang API and local caching.

use crate::storage::{Storage, StorageError, StorageResult};
use pumpkin_plugin_api::Server;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

#[cfg(feature = "http")]
use reqwest::Client;
#[cfg(feature = "http")]
use uuid::Uuid;

/// UUID resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedPlayer {
    pub uuid: String,
    pub name: String,
}

/// Cached UUID entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedUuid {
    uuid: String,
    name: String,
    resolved_at: u64, // Unix timestamp
}

/// UUID resolver with Mojang API and local cache
pub struct UuidResolver {
    storage: Arc<Storage>,
    cache: RwLock<HashMap<String, CachedUuid>>, // name_lower -> cached
    reverse_cache: RwLock<HashMap<String, CachedUuid>>, // uuid -> cached
    #[cfg(feature = "http")]
    client: Client,
    cache_ttl: Duration,
    negative_cache_ttl: Duration,
}

impl UuidResolver {
    /// Create new UUID resolver
    pub fn new(storage: Arc<Storage>) -> Self {
        Self {
            storage,
            cache: RwLock::new(HashMap::new()),
            reverse_cache: RwLock::new(HashMap::new()),
            #[cfg(feature = "http")]
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| Client::new()),
            cache_ttl: Duration::from_secs(86400 * 30), // 30 days
            negative_cache_ttl: Duration::from_secs(3600), // 1 hour
        }
    }

    /// Create with custom cache TTL
    pub fn with_ttl(storage: Arc<Storage>, cache_ttl: Duration, negative_cache_ttl: Duration) -> Self {
        Self {
            storage,
            cache: RwLock::new(HashMap::new()),
            reverse_cache: RwLock::new(HashMap::new()),
            #[cfg(feature = "http")]
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| Client::new()),
            cache_ttl,
            negative_cache_ttl,
        }
    }

    /// Resolve a player name to UUID (checks online players first, then cache, then Mojang API)
    pub async fn resolve_name(&self, server: &Server, name: &str) -> Option<ResolvedPlayer> {
        let name_lower = name.to_lowercase();

        // 1. Check online players first
        if let Some(player) = server.get_player_by_name(name) {
            return Some(ResolvedPlayer {
                uuid: player.get_id().to_string(),
                name: player.get_name(),
            });
        }

        // 2. Check positive cache
        if let Some(cached) = self.get_from_cache(&name_lower, true).await {
            return Some(ResolvedPlayer {
                uuid: cached.uuid,
                name: cached.name,
            });
        }

        // 3. Check negative cache (known not to exist)
        if self.is_negative_cached(&name_lower).await {
            return None;
        }

        // 4. Try Mojang API
        #[cfg(feature = "http")]
        if let Some(resolved) = self.resolve_via_mojang(&name_lower).await {
            self.cache_positive(&name_lower, &resolved).await;
            return Some(resolved);
        }

        // 5. Cache negative result
        self.cache_negative(&name_lower).await;
        None
    }

    /// Resolve UUID to name (checks cache first)
    pub async fn resolve_uuid(&self, uuid: &str) -> Option<String> {
        let uuid_lower = uuid.to_lowercase();

        // Check reverse cache
        if let Some(cached) = self.get_from_reverse_cache(&uuid_lower).await {
            return Some(cached.name);
        }

        // Try to get from storage (user data)
        if let Some(user) = self.storage.get::<crate::permissions::UserData>("users", uuid).ok().flatten() {
            let name = user.name.clone();
            self.cache_reverse(&uuid_lower, &name).await;
            return Some(name);
        }

        #[cfg(feature = "http")]
        {
            // Try Mojang API for UUID -> name
            if let Some(name) = self.resolve_uuid_via_mojang(&uuid_lower).await {
                self.cache_reverse(&uuid_lower, &name).await;
                return Some(name);
            }
        }

        None
    }

    /// Get player by name or UUID (tries both)
    pub async fn resolve(&self, server: &Server, identifier: &str) -> Option<ResolvedPlayer> {
        // Try as UUID first (36 chars with hyphens or 32 without)
        if identifier.len() >= 32 && identifier.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
            if let Some(name) = self.resolve_uuid(identifier).await {
                return Some(ResolvedPlayer {
                    uuid: identifier.to_string(),
                    name,
                });
            }
        }

        // Try as name
        self.resolve_name(server, identifier).await
    }

    /// Resolve via Mojang API (name -> UUID)
    #[cfg(feature = "http")]
    async fn resolve_via_mojang(&self, name: &str) -> Option<ResolvedPlayer> {
        let url = format!("https://api.mojang.com/users/profiles/minecraft/{}", name);
        let response = self.client.get(&url).send().await.ok()?;
        
        if !response.status().is_success() {
            return None;
        }

        #[derive(Deserialize)]
        struct MojangResponse {
            id: String,
            name: String,
        }

        let data: MojangResponse = response.json().await.ok()?;
        let uuid = format_uuid(&data.id)?;
        
        Some(ResolvedPlayer { uuid, name: data.name })
    }

    /// Resolve via Mojang API (UUID -> name)
    #[cfg(feature = "http")]
    async fn resolve_uuid_via_mojang(&self, uuid: &str) -> Option<String> {
        let clean_uuid = uuid.replace('-', "");
        let url = format!("https://sessionserver.mojang.com/session/minecraft/profile/{}", clean_uuid);
        let response = self.client.get(&url).send().await.ok()?;
        
        if !response.status().is_success() {
            return None;
        }

        #[derive(Deserialize)]
        struct ProfileResponse {
            name: String,
        }

        let data: ProfileResponse = response.json().await.ok()?;
        Some(data.name)
    }

    async fn get_from_cache(&self, name: &str, check_ttl: bool) -> Option<CachedUuid> {
        let cache = self.cache.read().ok()?;
        cache.get(name).and_then(|cached| {
            if check_ttl && cached.resolved_at + self.cache_ttl.as_secs() < current_timestamp() {
                return None;
            }
            Some(cached.clone())
        })
    }

    async fn is_negative_cached(&self, name: &str) -> bool {
        let cache = self.cache.read().ok()?;
        cache.get(name).map(|cached| {
            cached.uuid.is_empty() && cached.resolved_at + self.negative_cache_ttl.as_secs() > current_timestamp()
        }).unwrap_or(false)
    }

    async fn cache_positive(&self, name: &str, resolved: &ResolvedPlayer) {
        let mut cache = match self.cache.write() {
            Ok(c) => c,
            Err(_) => return,
        };
        let mut reverse_cache = match self.reverse_cache.write() {
            Ok(c) => c,
            Err(_) => return,
        };
        
        let cached = CachedUuid {
            uuid: resolved.uuid.clone(),
            name: resolved.name.clone(),
            resolved_at: current_timestamp(),
        };
        
        cache.insert(name.to_string(), cached.clone());
        reverse_cache.insert(resolved.uuid.to_lowercase(), cached);
        
        // Persist to storage
        let _ = self.storage.set("uuid_cache", name, &cached);
    }

    async fn cache_negative(&self, name: &str) {
        let mut cache = match self.cache.write() {
            Ok(c) => c,
            Err(_) => return,
        };
        
        let cached = CachedUuid {
            uuid: String::new(), // Empty UUID = negative cache
            name: String::new(),
            resolved_at: current_timestamp(),
        };
        
        cache.insert(name.to_string(), cached);
    }

    async fn get_from_reverse_cache(&self, uuid: &str) -> Option<CachedUuid> {
        let cache = self.reverse_cache.read().ok()?;
        cache.get(uuid).cloned()
    }

    async fn cache_reverse(&self, uuid: &str, name: &str) {
        let mut cache = match self.reverse_cache.write() {
            Ok(c) => c,
            Err(_) => return,
        };
        
        let cached = CachedUuid {
            uuid: uuid.to_string(),
            name: name.to_string(),
            resolved_at: current_timestamp(),
        };
        
        cache.insert(uuid.to_string(), cached);
    }

    /// Load cache from storage on startup
    pub fn load_cache(&self) {
        if let Ok(keys) = self.storage.keys("uuid_cache") {
            for key in keys {
                if let Ok(Some(cached)) = self.storage.get::<CachedUuid>("uuid_cache", &key) {
                    if cached.uuid.is_empty() {
                        // Negative cache entry
                        if let Ok(mut cache) = self.cache.write() {
                            cache.insert(key, cached);
                        }
                    } else {
                        // Positive cache entry
                        if let (Ok(mut cache), Ok(mut reverse_cache)) = (self.cache.write(), self.reverse_cache.write()) {
                            cache.insert(key.clone(), cached.clone());
                            reverse_cache.insert(cached.uuid.to_lowercase(), cached);
                        }
                    }
                }
            }
        }
    }

    /// Synchronously resolve a player identifier (name or UUID) using cache only.
    /// Returns ResolvedPlayer if found in cache or online, None otherwise.
    /// For cache misses, returns None (caller should handle fallback).
    pub fn resolve_sync(&self, server: &Server, identifier: &str) -> Option<ResolvedPlayer> {
        let identifier_lower = identifier.to_lowercase();

        // 1. Check online players first
        if let Some(player) = server.get_player_by_name(identifier) {
            return Some(ResolvedPlayer {
                uuid: player.get_id().to_string(),
                name: player.get_name(),
            });
        }

        // 2. Check if it looks like a UUID (32 or 36 chars, hex)
        let is_uuid = identifier.len() >= 32 && identifier.chars().all(|c| c.is_ascii_hexdigit() || c == '-');
        if is_uuid {
            // Try reverse cache
            if let Some(cached) = self.get_from_reverse_cache_sync(&identifier_lower) {
                return Some(ResolvedPlayer {
                    uuid: cached.uuid,
                    name: cached.name,
                });
            }
            // Not in cache, but looks like UUID - return as-is
            return Some(ResolvedPlayer {
                uuid: identifier.to_string(),
                name: identifier.to_string(),
            });
        }

        // 3. Check positive cache by name
        if let Some(cached) = self.get_from_cache_sync(&identifier_lower, true) {
            return Some(ResolvedPlayer {
                uuid: cached.uuid,
                name: cached.name,
            });
        }

        // 4. Check negative cache
        if self.is_negative_cached_sync(&identifier_lower) {
            return None;
        }

        // 5. Not in cache - return None (caller handles fallback)
        None
    }

    /// Synchronously get from positive cache
    fn get_from_cache_sync(&self, name: &str, check_ttl: bool) -> Option<CachedUuid> {
        let cache = self.cache.read().ok()?;
        cache.get(name).and_then(|cached| {
            if check_ttl && cached.resolved_at + self.cache_ttl.as_secs() < current_timestamp() {
                return None;
            }
            Some(cached.clone())
        })
    }

    /// Synchronously check negative cache
    fn is_negative_cached_sync(&self, name: &str) -> bool {
        let cache = self.cache.read().ok()?;
        cache.get(name).map(|cached| {
            cached.uuid.is_empty() && cached.resolved_at + self.negative_cache_ttl.as_secs() > current_timestamp()
        }).unwrap_or(false)
    }

    /// Synchronously get from reverse cache
    fn get_from_reverse_cache_sync(&self, uuid: &str) -> Option<CachedUuid> {
        let cache = self.reverse_cache.read().ok()?;
        cache.get(uuid).cloned()
    }

    /// Synchronously cache a positive result
    pub fn cache_positive_sync(&self, name: &str, resolved: &ResolvedPlayer) {
        let mut cache = match self.cache.write() {
            Ok(c) => c,
            Err(_) => return,
        };
        let mut reverse_cache = match self.reverse_cache.write() {
            Ok(c) => c,
            Err(_) => return,
        };
        
        let cached = CachedUuid {
            uuid: resolved.uuid.clone(),
            name: resolved.name.clone(),
            resolved_at: current_timestamp(),
        };
        
        cache.insert(name.to_string(), cached.clone());
        reverse_cache.insert(resolved.uuid.to_lowercase(), cached);
        
        // Persist to storage
        let _ = self.storage.set("uuid_cache", name, &cached);
    }

    /// Synchronously cache a negative result
    pub fn cache_negative_sync(&self, name: &str) {
        let mut cache = match self.cache.write() {
            Ok(c) => c,
            Err(_) => return,
        };
        
        let cached = CachedUuid {
            uuid: String::new(), // Empty UUID = negative cache
            name: String::new(),
            resolved_at: current_timestamp(),
        };
        
        cache.insert(name.to_string(), cached);
    }
}

/// Format UUID string to standard format (8-4-4-4-12)
fn format_uuid(input: &str) -> Option<String> {
    let clean = input.replace('-', "");
    if clean.len() != 32 {
        return None;
    }
    Some(format!(
        "{}-{}-{}-{}-{}",
        &clean[0..8],
        &clean[8..12],
        &clean[12..16],
        &clean[16..20],
        &clean[20..32]
    ))
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_uuid() {
        assert_eq!(
            format_uuid("1234567890abcdef1234567890abcdef"),
            Some("12345678-90ab-cdef-1234-567890abcdef".to_string())
        );
        assert_eq!(format_uuid("invalid"), None);
    }
}