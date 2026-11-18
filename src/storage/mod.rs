//! Multi-backend storage for PumpkinMC plugins.
//!
//! Supports JSON (WASM-compatible) and SQLite (native) backends.
//! Provides file locking, batch writes, and proper error handling.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use thiserror::Error;

pub mod json;
pub mod sqlite;

use json::JsonStorage;
#[cfg(not(target_arch = "wasm32"))]
use sqlite::SqliteStorage;

/// Storage backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    Json,
    Sqlite,
}

/// Storage errors
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Lock error: {0}")]
    Lock(String),
    #[error("Key not found: {0}")]
    NotFound(String),
    #[error("Backend error: {0}")]
    Backend(String),
    #[error("SQLite error: {0}")]
    #[cfg(not(target_arch = "wasm32"))]
    Sqlite(#[from] rusqlite::Error),
}

pub type StorageResult<T> = Result<T, StorageError>;

/// Trait for storage backends
pub trait StorageBackend: Send + Sync {
    fn get<T: for<'de> Deserialize<'de>>(&self, table: &str, key: &str) -> StorageResult<Option<T>>;
    fn set<T: Serialize>(&self, table: &str, key: &str, value: &T) -> StorageResult<()>;
    fn delete(&self, table: &str, key: &str) -> StorageResult<()>;
    fn keys(&self, table: &str) -> StorageResult<Vec<String>>;
    fn values<T: for<'de> Deserialize<'de>>(&self, table: &str) -> StorageResult<Vec<T>>;
    fn flush(&self) -> StorageResult<()>;
    fn backend_type(&self) -> StorageBackend;
}

/// Main storage handle - clones are cheap (Arc internally)
#[derive(Clone)]
pub struct Storage {
    inner: Arc<dyn StorageBackend>,
}

impl Storage {
    /// Create storage with auto-detected backend based on config and platform
    pub fn new(data_folder: &str, backend: StorageBackend) -> StorageResult<Self> {
        let path = PathBuf::from(data_folder);
        std::fs::create_dir_all(&path)?;

        let inner: Arc<dyn StorageBackend> = match backend {
            StorageBackend::Json => Arc::new(JsonStorage::new(&path)?),
            #[cfg(not(target_arch = "wasm32"))]
            StorageBackend::Sqlite => Arc::new(SqliteStorage::new(&path)?),
            #[cfg(target_arch = "wasm32")]
            StorageBackend::Sqlite => {
                tracing::warn!("SQLite not available on WASM, falling back to JSON");
                Arc::new(JsonStorage::new(&path)?)
            }
        };

        Ok(Self { inner })
    }

    /// Create JSON storage (always available)
    pub fn new_json(data_folder: &str) -> StorageResult<Self> {
        let path = PathBuf::from(data_folder);
        std::fs::create_dir_all(&path)?;
        let inner = Arc::new(JsonStorage::new(&path)?);
        Ok(Self { inner })
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Create SQLite storage (native only)
    pub fn new_sqlite(data_folder: &str) -> StorageResult<Self> {
        let path = PathBuf::from(data_folder);
        std::fs::create_dir_all(&path)?;
        let inner = Arc::new(SqliteStorage::new(&path)?);
        Ok(Self { inner })
    }

    pub fn get<T: for<'de> Deserialize<'de>>(&self, table: &str, key: &str) -> StorageResult<Option<T>> {
        self.inner.get(table, key)
    }

    pub fn set<T: Serialize>(&self, table: &str, key: &str, value: &T) -> StorageResult<()> {
        self.inner.set(table, key, value)
    }

    pub fn delete(&self, table: &str, key: &str) -> StorageResult<()> {
        self.inner.delete(table, key)
    }

    pub fn keys(&self, table: &str) -> StorageResult<Vec<String>> {
        self.inner.keys(table)
    }

    pub fn values<T: for<'de> Deserialize<'de>>(&self, table: &str) -> StorageResult<Vec<T>> {
        self.inner.values(table)
    }

    pub fn flush(&self) -> StorageResult<()> {
        self.inner.flush()
    }

    pub fn backend_type(&self) -> StorageBackend {
        self.inner.backend_type()
    }
}

impl std::fmt::Debug for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Storage")
            .field("backend", &self.inner.backend_type())
            .finish()
    }
}

/// Re-export for convenience
pub use self::json::JsonStorage;
#[cfg(not(target_arch = "wasm32"))]
pub use self::sqlite::SqliteStorage;