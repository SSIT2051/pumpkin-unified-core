//! SQLite storage backend (native only).
//!
//! Uses rusqlite with WAL mode for better concurrency.
//! Only compiled for non-WASM targets.

#[cfg(not(target_arch = "wasm32"))]
use super::{StorageBackend, StorageBackendType, StorageError, StorageResult};
#[cfg(not(target_arch = "wasm32"))]
use rusqlite::{Connection, OpenFlags, params};
#[cfg(not(target_arch = "wasm32"))]
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, Mutex};

#[cfg(not(target_arch = "wasm32"))]
/// SQLite storage with WAL mode
pub struct SqliteStorage {
    conn: Arc<Mutex<Connection>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl SqliteStorage {
    /// Create new SQLite storage at the given data folder
    pub fn new(data_folder: &Path) -> StorageResult<Self> {
        let path = data_folder.join("data.db");
        let mut conn = Connection::open_with_flags(
            &path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX
                | OpenFlags::SQLITE_OPEN_URI,
        )?;

        // Enable WAL mode for better concurrency
        conn.execute("PRAGMA journal_mode = WAL", [])?;
        conn.execute("PRAGMA synchronous = NORMAL", [])?;
        conn.execute("PRAGMA busy_timeout = 5000", [])?;

        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS kv_store (
                table_name TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                PRIMARY KEY (table_name, key)
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_kv_table ON kv_store(table_name)",
            [],
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl StorageBackend for SqliteStorage {
    fn get<T: for<'de> Deserialize<'de>>(&self, table: &str, key: &str) -> StorageResult<Option<T>> {
        let conn = self.conn.lock().map_err(|e| StorageError::Lock(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT value FROM kv_store WHERE table_name = ?1 AND key = ?2"
        )?;
        let value: Option<String> = stmt.query_row(params![table, key], |row| row.get(0)).ok();
        Ok(value.and_then(|v| serde_json::from_str(&v).ok()))
    }

    fn set<T: Serialize>(&self, table: &str, key: &str, value: &T) -> StorageResult<()> {
        let json = serde_json::to_string(value)?;
        let conn = self.conn.lock().map_err(|e| StorageError::Lock(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO kv_store (table_name, key, value) VALUES (?1, ?2, ?3)",
            params![table, key, json],
        )?;
        Ok(())
    }

    fn delete(&self, table: &str, key: &str) -> StorageResult<()> {
        let conn = self.conn.lock().map_err(|e| StorageError::Lock(e.to_string()))?;
        conn.execute(
            "DELETE FROM kv_store WHERE table_name = ?1 AND key = ?2",
            params![table, key],
        )?;
        Ok(())
    }

    fn keys(&self, table: &str) -> StorageResult<Vec<String>> {
        let conn = self.conn.lock().map_err(|e| StorageError::Lock(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT key FROM kv_store WHERE table_name = ?1"
        )?;
        let keys = stmt.query_map(params![table], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        Ok(keys)
    }

    fn values<T: for<'de> Deserialize<'de>>(&self, table: &str) -> StorageResult<Vec<T>> {
        let conn = self.conn.lock().map_err(|e| StorageError::Lock(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT value FROM kv_store WHERE table_name = ?1"
        )?;
        let values = stmt.query_map(params![table], |row| {
            let json: String = row.get(0)?;
            serde_json::from_str(&json).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
        })?
        .collect::<Result<Vec<T>, _>>()?;
        Ok(values)
    }

    fn flush(&self) -> StorageResult<()> {
        // SQLite with WAL mode auto-flushes, but we can force a checkpoint
        let conn = self.conn.lock().map_err(|e| StorageError::Lock(e.to_string()))?;
        conn.execute("PRAGMA wal_checkpoint(TRUNCATE)", [])?;
        Ok(())
    }

    fn backend_type(&self) -> StorageBackendType {
        StorageBackendType::Sqlite
    }
}