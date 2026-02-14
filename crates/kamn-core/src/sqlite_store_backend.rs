//! Sqlite-backed key/value storage foundation for runtime persistence adapters.
//!
//! This module provides deterministic sqlite bootstrap/version checks and a
//! minimal namespace/key/value API that higher-level store adapters can use.

use rusqlite::{params, Connection, OptionalExtension};
use std::fmt;
use std::path::Path;

const META_SCHEMA_KEY: &str = "schema_version";
const CREATE_META_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS kamn_store_meta (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
)
"#;
const CREATE_ENTRIES_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS kamn_store_entries (
  namespace TEXT NOT NULL,
  entry_key TEXT NOT NULL,
  entry_value BLOB NOT NULL,
  PRIMARY KEY(namespace, entry_key)
)
"#;

/// Current sqlite schema version expected by `SqliteStoreBackend`.
pub const SQLITE_STORE_SCHEMA_VERSION: u32 = 1;

/// Sqlite backend error taxonomy for open/bootstrap/query operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqliteStoreBackendError {
    /// Backend path cannot be empty.
    InvalidPath,
    /// Required field is empty.
    EmptyField(&'static str),
    /// Connection open failed.
    Open(String),
    /// SQLite pragma setup failed.
    Pragma(String),
    /// Schema bootstrap DDL failed.
    Migration(String),
    /// Schema version row is missing.
    SchemaVersionMissing,
    /// Schema version row is present but invalid.
    SchemaVersionInvalid(String),
    /// Schema version does not match expected backend version.
    SchemaVersionMismatch {
        /// Expected schema version.
        expected: u32,
        /// Found schema version.
        found: u32,
    },
    /// Query execution failed.
    Query(String),
}

impl fmt::Display for SqliteStoreBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath => write!(f, "sqlite backend path cannot be empty"),
            Self::EmptyField(field) => write!(f, "sqlite backend field cannot be empty: {field}"),
            Self::Open(message) => write!(f, "sqlite backend open failed: {message}"),
            Self::Pragma(message) => write!(f, "sqlite backend pragma setup failed: {message}"),
            Self::Migration(message) => {
                write!(f, "sqlite backend schema bootstrap failed: {message}")
            }
            Self::SchemaVersionMissing => write!(f, "sqlite backend schema version row missing"),
            Self::SchemaVersionInvalid(value) => {
                write!(f, "sqlite backend schema version invalid: {value}")
            }
            Self::SchemaVersionMismatch { expected, found } => write!(
                f,
                "sqlite backend schema version mismatch: expected {expected}, found {found}"
            ),
            Self::Query(message) => write!(f, "sqlite backend query failed: {message}"),
        }
    }
}

impl std::error::Error for SqliteStoreBackendError {}

/// Minimal sqlite namespace/key/value backend used by higher-level stores.
#[derive(Debug)]
pub struct SqliteStoreBackend {
    connection: Connection,
    schema_version: u32,
}

impl SqliteStoreBackend {
    /// Opens backend at `path`, bootstrapping schema and validating version.
    pub fn open(path: &Path) -> Result<Self, SqliteStoreBackendError> {
        if path.as_os_str().is_empty() {
            return Err(SqliteStoreBackendError::InvalidPath);
        }
        let connection = Connection::open(path)
            .map_err(|error| SqliteStoreBackendError::Open(error.to_string()))?;
        Self::from_connection(connection)
    }

    /// Opens backend in-memory for tests and deterministic local probes.
    pub fn open_in_memory() -> Result<Self, SqliteStoreBackendError> {
        let connection = Connection::open_in_memory()
            .map_err(|error| SqliteStoreBackendError::Open(error.to_string()))?;
        Self::from_connection(connection)
    }

    /// Returns active schema version validated at backend initialization.
    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// Inserts or updates `value` for namespace/key.
    pub fn put(
        &mut self,
        namespace: &str,
        key: &str,
        value: &[u8],
    ) -> Result<(), SqliteStoreBackendError> {
        validate_namespace_and_key(namespace, key)?;
        self.connection
            .execute(
                r#"
                INSERT INTO kamn_store_entries (namespace, entry_key, entry_value)
                VALUES (?1, ?2, ?3)
                ON CONFLICT(namespace, entry_key)
                DO UPDATE SET entry_value = excluded.entry_value
                "#,
                params![namespace, key, value],
            )
            .map_err(|error| SqliteStoreBackendError::Query(error.to_string()))?;
        Ok(())
    }

    /// Loads value for namespace/key if present.
    pub fn get(
        &self,
        namespace: &str,
        key: &str,
    ) -> Result<Option<Vec<u8>>, SqliteStoreBackendError> {
        validate_namespace_and_key(namespace, key)?;
        self.connection
            .query_row(
                r#"
                SELECT entry_value
                FROM kamn_store_entries
                WHERE namespace = ?1 AND entry_key = ?2
                "#,
                params![namespace, key],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| SqliteStoreBackendError::Query(error.to_string()))
    }

    /// Lists keys for namespace in deterministic lexical order.
    pub fn list_keys(&self, namespace: &str) -> Result<Vec<String>, SqliteStoreBackendError> {
        if namespace.trim().is_empty() {
            return Err(SqliteStoreBackendError::EmptyField("namespace"));
        }
        let mut statement = self
            .connection
            .prepare(
                r#"
                SELECT entry_key
                FROM kamn_store_entries
                WHERE namespace = ?1
                ORDER BY entry_key ASC
                "#,
            )
            .map_err(|error| SqliteStoreBackendError::Query(error.to_string()))?;
        let rows = statement
            .query_map(params![namespace], |row| row.get::<_, String>(0))
            .map_err(|error| SqliteStoreBackendError::Query(error.to_string()))?;
        let mut keys = Vec::new();
        for row in rows {
            keys.push(row.map_err(|error| SqliteStoreBackendError::Query(error.to_string()))?);
        }
        Ok(keys)
    }

    /// Deletes namespace/key row and returns `true` when a row was removed.
    pub fn delete(&mut self, namespace: &str, key: &str) -> Result<bool, SqliteStoreBackendError> {
        validate_namespace_and_key(namespace, key)?;
        let deleted = self
            .connection
            .execute(
                r#"
                DELETE FROM kamn_store_entries
                WHERE namespace = ?1 AND entry_key = ?2
                "#,
                params![namespace, key],
            )
            .map_err(|error| SqliteStoreBackendError::Query(error.to_string()))?;
        Ok(deleted > 0)
    }

    fn from_connection(connection: Connection) -> Result<Self, SqliteStoreBackendError> {
        connection
            .execute_batch(
                r#"
                PRAGMA foreign_keys = ON;
                PRAGMA busy_timeout = 5000;
                "#,
            )
            .map_err(|error| SqliteStoreBackendError::Pragma(error.to_string()))?;
        connection
            .execute_batch(CREATE_META_TABLE_SQL)
            .map_err(|error| SqliteStoreBackendError::Migration(error.to_string()))?;
        connection
            .execute_batch(CREATE_ENTRIES_TABLE_SQL)
            .map_err(|error| SqliteStoreBackendError::Migration(error.to_string()))?;
        let schema_version = bootstrap_and_validate_schema_version(&connection)?;
        Ok(Self {
            connection,
            schema_version,
        })
    }
}

fn validate_namespace_and_key(namespace: &str, key: &str) -> Result<(), SqliteStoreBackendError> {
    if namespace.trim().is_empty() {
        return Err(SqliteStoreBackendError::EmptyField("namespace"));
    }
    if key.trim().is_empty() {
        return Err(SqliteStoreBackendError::EmptyField("key"));
    }
    Ok(())
}

fn bootstrap_and_validate_schema_version(
    connection: &Connection,
) -> Result<u32, SqliteStoreBackendError> {
    let maybe_schema_version: Option<String> = connection
        .query_row(
            "SELECT value FROM kamn_store_meta WHERE key = ?1",
            params![META_SCHEMA_KEY],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| SqliteStoreBackendError::Query(error.to_string()))?;

    if maybe_schema_version.is_none() {
        connection
            .execute(
                "INSERT INTO kamn_store_meta (key, value) VALUES (?1, ?2)",
                params![META_SCHEMA_KEY, SQLITE_STORE_SCHEMA_VERSION.to_string()],
            )
            .map_err(|error| SqliteStoreBackendError::Migration(error.to_string()))?;
    }

    let current = connection
        .query_row(
            "SELECT value FROM kamn_store_meta WHERE key = ?1",
            params![META_SCHEMA_KEY],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| SqliteStoreBackendError::Query(error.to_string()))?
        .ok_or(SqliteStoreBackendError::SchemaVersionMissing)?;

    let parsed = current
        .parse::<u32>()
        .map_err(|_| SqliteStoreBackendError::SchemaVersionInvalid(current.clone()))?;
    if parsed != SQLITE_STORE_SCHEMA_VERSION {
        return Err(SqliteStoreBackendError::SchemaVersionMismatch {
            expected: SQLITE_STORE_SCHEMA_VERSION,
            found: parsed,
        });
    }
    Ok(parsed)
}
