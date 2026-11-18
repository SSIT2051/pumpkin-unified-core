//! JSON file-based storage with file locking and batch writes.
//!
//! WASM-compatible, uses exclusive file locks for safety.

use super::{StorageBackend, StorageBackendType, StorageError, StorageResult};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

type Table = BTreeMap<String, serde_json::Value>;

#[derive(Default, Serialize, Deserialize)]
struct Store {
    #[serde(default)]
    tables: BTreeMap<String, Table>,
}

/// Configuration for JSON storage
#[derive(Debug, Clone)]
pub struct JsonStorageConfig {
    pub flush_interval: Duration,
    pub max_batch_size: usize,
}

impl Default for JsonStorageConfig {
    fn default() -> Self {
        Self {
            flush_interval: Duration::from_secs(5),
            max_batch_size: 100,
        }
    }
}

/// JSON file storage with locking and batching
pub struct JsonStorage {
    path: PathBuf,
    config: JsonStorageConfig,
    store: RwLock<Store>,
    pending_writes: Mutex<Vec<PendingWrite>>,
    last_flush: Mutex<Instant>,
}

#[derive(Debug, Clone)]
struct PendingWrite {
    table: String,
    key: String,
    value: serde_json::Value,
    is_delete: bool,
}

impl JsonStorage {
    /// Create new JSON storage at the given data folder
    pub fn new(data_folder: &Path) -> StorageResult<Self> {
        let path = data_folder.join("data.json");
        let store = if path.exists() {
            let file = File::open(&path)?;
            // Acquire shared lock for reading
            file.lock_shared().map_err(|e| StorageError::Lock(e.to_string()))?;
            let store: Store = serde_json::from_reader(&file).unwrap_or_default();
            file.unlock().map_err(|e| StorageError::Lock(e.to_string()))?;
            store
        } else {
            Store::default()
        };

        Ok(Self {
            path,
            config: JsonStorageConfig::default(),
            store: RwLock::new(store),
            pending_writes: Mutex::new(Vec::new()),
            last_flush: Mutex::new(Instant::now()),
        })
    }

    /// Create with custom config
    pub fn with_config(data_folder: &Path, config: JsonStorageConfig) -> StorageResult<Self> {
        let path = data_folder.join("data.json");
        let store = if path.exists() {
            let file = File::open(&path)?;
            file.lock_shared().map_err(|e| StorageError::Lock(e.to_string()))?;
            let store: Store = serde_json::from_reader(&file).unwrap_or_default();
            file.unlock().map_err(|e| StorageError::Lock(e.to_string()))?;
            store
        } else {
            Store::default()
        };

        Ok(Self {
            path,
            config,
            store: RwLock::new(store),
            pending_writes: Mutex::new(Vec::new()),
            last_flush: Mutex::new(Instant::now()),
        })
    }

    /// Get a value from a table
    pub fn get<T: for<'de> Deserialize<'de>>(&self, table: &str, key: &str) -> StorageResult<Option<T>> {
        let store = self.store.read().map_err(|e| StorageError::Lock(e.to_string()))?;
        Ok(store
            .tables
            .get(table)
            .and_then(|t| t.get(key))
            .cloned()
            .and_then(|v| serde_json::from_value(v).ok()))
    }

    /// Set a value in a table (batched)
    pub fn set<T: Serialize>(&self, table: &str, key: &str, value: &T) -> StorageResult<()> {
        let json = serde_json::to_value(value)?;
        
        // Update in-memory store immediately
        {
            let mut store = self.store.write().map_err(|e| StorageError::Lock(e.to_string()))?;
            store
                .tables
                .entry(table.to_string())
                .or_default()
                .insert(key.to_string(), json.clone());
        }

        // Queue for batched write
        {
            let mut pending = self.pending_writes.lock().map_err(|e| StorageError::Lock(e.to_string()))?;
            pending.push(PendingWrite {
                table: table.to_string(),
                key: key.to_string(),
                value: json,
                is_delete: false,
            });
            self.maybe_flush(pending)?;
        }

        Ok(())
    }

    /// Delete a key from a table (batched)
    pub fn delete(&self, table: &str, key: &str) -> StorageResult<()> {
        // Update in-memory store immediately
        {
            let mut store = self.store.write().map_err(|e| StorageError::Lock(e.to_string()))?;
            if let Some(t) = store.tables.get_mut(table) {
                t.remove(key);
            }
        }

        // Queue for batched write
        {
            let mut pending = self.pending_writes.lock().map_err(|e| StorageError::Lock(e.to_string()))?;
            pending.push(PendingWrite {
                table: table.to_string(),
                key: key.to_string(),
                value: serde_json::Value::Null,
                is_delete: true,
            });
            self.maybe_flush(pending)?;
        }

        Ok(())
    }

    /// Get all keys in a table
    pub fn keys(&self, table: &str) -> StorageResult<Vec<String>> {
        let store = self.store.read().map_err(|e| StorageError::Lock(e.to_string()))?;
        Ok(store
            .tables
            .get(table)
            .map(|t| t.keys().cloned().collect())
            .unwrap_or_default())
    }

    /// Get all values in a table
    pub fn values<T: for<'de> Deserialize<'de>>(&self, table: &str) -> StorageResult<Vec<T>> {
        let keys = self.keys(table)?;
        let mut results = Vec::with_capacity(keys.len());
        for key in keys {
            if let Some(value) = self.get(table, &key)? {
                results.push(value);
            }
        }
        Ok(results)
    }

    /// Flush all pending writes to disk
    pub fn flush(&self) -> StorageResult<()> {
        let pending = {
            let mut pending = self.pending_writes.lock().map_err(|e| StorageError::Lock(e.to_string()))?;
            if pending.is_empty() {
                return Ok(());
            }
            std::mem::take(&mut *pending)
        };

        self.write_to_disk(pending)?;
        *self.last_flush.lock().map_err(|e| StorageError::Lock(e.to_string()))? = Instant::now();
        Ok(())
    }

    fn maybe_flush(&self, mut pending: std::sync::MutexGuard<'_, Vec<PendingWrite>>) -> StorageResult<()> {
        let should_flush = pending.len() >= self.config.max_batch_size
            || self.last_flush.lock().map_err(|e| StorageError::Lock(e.to_string()))?.elapsed() >= self.config.flush_interval;

        if should_flush {
            let writes = std::mem::take(&mut *pending);
            drop(pending);
            self.write_to_disk(writes)?;
            *self.last_flush.lock().map_err(|e| StorageError::Lock(e.to_string()))? = Instant::now();
        }
        Ok(())
    }

    fn write_to_disk(&self, writes: Vec<PendingWrite>) -> StorageResult<()> {
        if writes.is_empty() {
            return Ok(());
        }

        // Apply writes to in-memory store first (already done, but ensure consistency)
        {
            let mut store = self.store.write().map_err(|e| StorageError::Lock(e.to_string()))?;
            for write in &writes {
                if write.is_delete {
                    if let Some(t) = store.tables.get_mut(&write.table) {
                        t.remove(&write.key);
                    }
                } else {
                    store
                        .tables
                        .entry(write.table.clone())
                        .or_default()
                        .insert(write.key.clone(), write.value.clone());
                }
            }
        }

        // Write to disk with exclusive lock
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)?;
        
        file.lock_exclusive().map_err(|e| StorageError::Lock(e.to_string()))?;
        
        let store = self.store.read().map_err(|e| StorageError::Lock(e.to_string()))?;
        serde_json::to_writer(&file, &*store)?;
        
        file.flush()?;
        file.unlock().map_err(|e| StorageError::Lock(e.to_string()))?;

        Ok(())
    }
}

impl StorageBackend for JsonStorage {
    fn get<T: for<'de> Deserialize<'de>>(&self, table: &str, key: &str) -> StorageResult<Option<T>> {
        self.get(table, key)
    }

    fn set<T: Serialize>(&self, table: &str, key: &str, value: &T) -> StorageResult<()> {
        self.set(table, key, value)
    }

    fn delete(&self, table: &str, key: &str) -> StorageResult<()> {
        self.delete(table, key)
    }

    fn keys(&self, table: &str) -> StorageResult<Vec<String>> {
        self.keys(table)
    }

    fn values<T: for<'de> Deserialize<'de>>(&self, table: &str) -> StorageResult<Vec<T>> {
        self.values(table)
    }

    fn flush(&self) -> StorageResult<()> {
        self.flush()
    }

    fn backend_type(&self) -> StorageBackendType {
        StorageBackendType::Json
    }
}

impl Drop for JsonStorage {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}