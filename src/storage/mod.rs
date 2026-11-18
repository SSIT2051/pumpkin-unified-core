//! WASI file-based key/value storage.
//!
//! Pumpkin plugins run inside a WebAssembly sandbox. The only durable storage is
//! the plugin's private WASI data folder, exposed through the normal file system.
//! This module persists a single JSON document (`<data_folder>/data.json`) that
//! maps table names to key/value records. Values are stored as `serde_json::Value`
//! so callers can (de)serialize their own typed structs.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Mutex;

type Table = BTreeMap<String, serde_json::Value>;

#[derive(Default, Serialize, Deserialize)]
struct Store {
    #[serde(default)]
    tables: BTreeMap<String, Table>,
}

/// A small synchronous, file-backed store guarded by a `Mutex`.
///
/// The plugin trait requires `Send + Sync`, and the WASM runtime is effectively
/// single-threaded, so a `Mutex` is cheap and safe here. Callers must not hold
/// the internal lock across a host API call; each method locks, copies out the
/// data it needs, and releases before returning.
pub struct Storage {
    store: Mutex<Store>,
    path: String,
}

impl Storage {
    /// Creates a store rooted at `data_folder`, loading any existing `data.json`.
    pub fn new(data_folder: &str) -> Self {
        let path = format!("{data_folder}/data.json");
        let store = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Self {
            store: Mutex::new(store),
            path,
        }
    }

    fn persist(&self, store: &Store) {
        if let Ok(json) = serde_json::to_string(store) {
            let _ = std::fs::write(&self.path, json);
        }
    }

    /// Loads and deserializes `key` from `table`, if present.
    pub fn get<T: for<'de> Deserialize<'de>>(&self, table: &str, key: &str) -> Option<T> {
        let store = self.store.lock().ok()?;
        store
            .tables
            .get(table)
            .and_then(|t| t.get(key))
            .cloned()
            .and_then(|v| serde_json::from_value(v).ok())
    }

    /// Serializes and stores `value` under `key` in `table`, then flushes to disk.
    pub fn set<T: Serialize>(&self, table: &str, key: &str, value: &T) {
        let Ok(mut store) = self.store.lock() else {
            return;
        };
        let Ok(json) = serde_json::to_value(value) else {
            return;
        };
        store
            .tables
            .entry(table.to_string())
            .or_default()
            .insert(key.to_string(), json);
        self.persist(&store);
    }

    /// Removes `key` from `table` and flushes to disk.
    pub fn delete(&self, table: &str, key: &str) {
        let Ok(mut store) = self.store.lock() else {
            return;
        };
        if let Some(t) = store.tables.get_mut(table) {
            t.remove(key);
        }
        self.persist(&store);
    }

    /// Returns all keys stored in `table`.
    pub fn keys(&self, table: &str) -> Vec<String> {
        self.store
            .lock()
            .ok()
            .and_then(|s| s.tables.get(table).map(|t| t.keys().cloned().collect()))
            .unwrap_or_default()
    }

    /// Returns all deserialized values in `table`.
    pub fn values<T: for<'de> Deserialize<'de>>(&self, table: &str) -> Vec<T> {
        let keys = self.keys(table);
        keys.iter().filter_map(|k| self.get::<T>(table, k)).collect()
    }
}