//! Live PostgreSQL execution adapter for data-layer bridge descriptors.
//!
//! This module executes deterministic SQL descriptors emitted by
//! `data_layer_postgres_repository_bridge` and applies migration artifacts from
//! `crates/kamn-core/migrations`.

use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{PgPool, Postgres, Row, Transaction};
use tracing::debug;

use crate::{
    data_layer_pg_project_insert_message_operation,
    data_layer_pg_project_select_message_by_id_operation, DataLayerM0EnvelopeRecord,
    DataLayerPgOperationKind, DataLayerPgRepositoryBridgeError, DataLayerPgRequesterSession,
};

/// Stable reason marker for invalid adapter database URL inputs.
pub const DATA_LAYER_PG_EXECUTION_INVALID_DATABASE_URL_REASON_CODE: &str =
    "data_layer_pg_execution_invalid_database_url";
/// Stable reason marker for migration-application failures.
pub const DATA_LAYER_PG_EXECUTION_MIGRATION_FAILED_REASON_CODE: &str =
    "data_layer_pg_execution_migration_failed";
/// Stable reason marker for SQL execution failures.
pub const DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE: &str =
    "data_layer_pg_execution_sql_failed";
/// Stable reason marker for requester session setup failures.
pub const DATA_LAYER_PG_EXECUTION_SESSION_FAILED_REASON_CODE: &str =
    "data_layer_pg_execution_session_failed";

const DATA_LAYER_PG_MIGRATIONS_DIR: &str = "migrations";

/// Configuration inputs for constructing a live PostgreSQL execution adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgExecutionAdapterConfig {
    /// PostgreSQL connection URL.
    pub database_url: String,
    /// Pool max-connections setting.
    pub max_connections: u32,
}

impl DataLayerPgExecutionAdapterConfig {
    /// Creates a configuration with default pool-size controls.
    pub fn new(database_url: impl Into<String>) -> Self {
        Self {
            database_url: database_url.into(),
            max_connections: 4,
        }
    }
}

/// Deterministic migration execution report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgMigrationReport {
    /// Migration file names discovered in deterministic order.
    pub migration_files: Vec<String>,
}

/// Stored message row projection returned by lookup operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgStoredMessage {
    /// Message identifier.
    pub message_id: String,
    /// Owner DID.
    pub owner_did: String,
    /// Sender DID.
    pub sender_did: String,
    /// Recipient DID.
    pub recipient_did: String,
    /// Ciphertext payload.
    pub envelope_ciphertext: String,
    /// Content hash.
    pub content_hash_sha256: String,
    /// Previous hash-chain pointer.
    pub hash_chain_prev: String,
    /// Retention class marker.
    pub retention_class: String,
}

/// Live execution adapter for running PostgreSQL bridge descriptors.
#[derive(Debug, Clone)]
pub struct DataLayerPgExecutionAdapter {
    pool: PgPool,
}

impl DataLayerPgExecutionAdapter {
    /// Connects to PostgreSQL and creates an execution adapter.
    pub async fn connect(
        config: DataLayerPgExecutionAdapterConfig,
    ) -> Result<Self, DataLayerPgExecutionAdapterError> {
        if config.database_url.trim().is_empty() {
            return Err(DataLayerPgExecutionAdapterError::EmptyField("database_url"));
        }
        if config.max_connections == 0 {
            return Err(DataLayerPgExecutionAdapterError::InvalidMaxConnections(
                config.max_connections,
            ));
        }

        let options =
            PgConnectOptions::from_str(config.database_url.as_str()).map_err(|error| {
                DataLayerPgExecutionAdapterError::InvalidDatabaseUrl {
                    field: "database_url",
                    reason_code: DATA_LAYER_PG_EXECUTION_INVALID_DATABASE_URL_REASON_CODE,
                    detail: error.to_string(),
                }
            })?;
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect_with(options)
            .await
            .map_err(
                |error| DataLayerPgExecutionAdapterError::SqlExecutionFailed {
                    operation: DataLayerPgOperationKind::SelectMessageById,
                    reason_code: DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE,
                    detail: format!("connection failed: {error}"),
                },
            )?;

        Ok(Self { pool })
    }

    /// Applies embedded migration files in deterministic order.
    pub async fn apply_migrations(
        &self,
    ) -> Result<DataLayerPgMigrationReport, DataLayerPgExecutionAdapterError> {
        let migration_files = data_layer_pg_collect_migration_files()?;
        debug!(
            migration_count = migration_files.len(),
            "applying data-layer postgres migrations"
        );
        for migration_file in &migration_files {
            let migration_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(DATA_LAYER_PG_MIGRATIONS_DIR)
                .join(migration_file);
            let migration_source =
                fs::read_to_string(migration_path.as_path()).map_err(|error| {
                    DataLayerPgExecutionAdapterError::MigrationIoFailed {
                        reason_code: DATA_LAYER_PG_EXECUTION_MIGRATION_FAILED_REASON_CODE,
                        detail: format!("failed reading {}: {error}", migration_path.display()),
                    }
                })?;
            let statements = data_layer_pg_split_migration_statements(migration_source.as_str());
            let mut transaction = self.pool.begin().await.map_err(|error| {
                DataLayerPgExecutionAdapterError::MigrationFailed {
                    reason_code: DATA_LAYER_PG_EXECUTION_MIGRATION_FAILED_REASON_CODE,
                    detail: format!(
                        "migration {} begin transaction failed: {error}",
                        migration_path.display()
                    ),
                }
            })?;
            for statement in statements {
                sqlx::query(statement.as_str())
                    .execute(&mut *transaction)
                    .await
                    .map_err(|error| DataLayerPgExecutionAdapterError::MigrationFailed {
                        reason_code: DATA_LAYER_PG_EXECUTION_MIGRATION_FAILED_REASON_CODE,
                        detail: format!(
                            "migration {} statement failed: {error}",
                            migration_path.display()
                        ),
                    })?;
            }
            transaction.commit().await.map_err(|error| {
                DataLayerPgExecutionAdapterError::MigrationFailed {
                    reason_code: DATA_LAYER_PG_EXECUTION_MIGRATION_FAILED_REASON_CODE,
                    detail: format!(
                        "migration {} commit failed: {error}",
                        migration_path.display()
                    ),
                }
            })?;
        }

        Ok(DataLayerPgMigrationReport { migration_files })
    }

    /// Executes one message insert descriptor projected by the repository bridge.
    pub async fn execute_insert_message(
        &self,
        record: &DataLayerM0EnvelopeRecord,
        owner_did: &str,
        requester_did: &str,
    ) -> Result<u64, DataLayerPgExecutionAdapterError> {
        let descriptor =
            data_layer_pg_project_insert_message_operation(record, owner_did, requester_did)
                .map_err(
                    |error| DataLayerPgExecutionAdapterError::BridgeProjectionFailed {
                        operation: DataLayerPgOperationKind::InsertMessage,
                        detail: error.to_string(),
                    },
                )?;
        let recipient_did = record.recipient_dids.first().ok_or(
            DataLayerPgExecutionAdapterError::BridgeProjectionFailed {
                operation: DataLayerPgOperationKind::InsertMessage,
                detail: "recipient_dids must include at least one entry".to_owned(),
            },
        )?;
        let envelope_nonce = i64::try_from(record.envelope_nonce).map_err(|error| {
            DataLayerPgExecutionAdapterError::BridgeProjectionFailed {
                operation: DataLayerPgOperationKind::InsertMessage,
                detail: format!("envelope_nonce conversion failed: {error}"),
            }
        })?;

        debug!(
            operation = "insert_message",
            message_id = %record.message_id,
            requester_did = %descriptor.session.requester_did,
            "executing postgres insert descriptor"
        );

        let mut tx = self
            .begin_transaction(DataLayerPgOperationKind::InsertMessage)
            .await?;
        self.apply_requester_session(
            &mut tx,
            &descriptor.session,
            DataLayerPgOperationKind::InsertMessage,
        )
        .await?;
        let execution = sqlx::query(descriptor.sql.as_str())
            .bind(record.message_id.as_str())
            .bind(owner_did)
            .bind(record.sender_did.as_str())
            .bind(recipient_did.as_str())
            .bind(record.envelope_ciphertext.as_bytes())
            .bind(envelope_nonce)
            .bind(record.content_hash.as_str())
            .bind(record.hash_chain_prev.as_str())
            .bind("{}")
            .bind(record.message_type.as_str())
            .execute(&mut *tx)
            .await
            .map_err(
                |error| DataLayerPgExecutionAdapterError::SqlExecutionFailed {
                    operation: DataLayerPgOperationKind::InsertMessage,
                    reason_code: DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE,
                    detail: error.to_string(),
                },
            )?;
        tx.commit().await.map_err(|error| {
            DataLayerPgExecutionAdapterError::SqlExecutionFailed {
                operation: DataLayerPgOperationKind::InsertMessage,
                reason_code: DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE,
                detail: format!("commit failed: {error}"),
            }
        })?;
        Ok(execution.rows_affected())
    }

    /// Executes one message lookup descriptor projected by the repository bridge.
    pub async fn execute_select_message_by_id(
        &self,
        message_id: &str,
        requester_did: &str,
    ) -> Result<Option<DataLayerPgStoredMessage>, DataLayerPgExecutionAdapterError> {
        let descriptor =
            data_layer_pg_project_select_message_by_id_operation(message_id, requester_did)
                .map_err(
                    |error| DataLayerPgExecutionAdapterError::BridgeProjectionFailed {
                        operation: DataLayerPgOperationKind::SelectMessageById,
                        detail: error.to_string(),
                    },
                )?;

        debug!(
            operation = "select_message_by_id",
            message_id = message_id,
            requester_did = %descriptor.session.requester_did,
            "executing postgres lookup descriptor"
        );

        let mut tx = self
            .begin_transaction(DataLayerPgOperationKind::SelectMessageById)
            .await?;
        self.apply_requester_session(
            &mut tx,
            &descriptor.session,
            DataLayerPgOperationKind::SelectMessageById,
        )
        .await?;
        let row = sqlx::query(descriptor.sql.as_str())
            .bind(message_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(
                |error| DataLayerPgExecutionAdapterError::SqlExecutionFailed {
                    operation: DataLayerPgOperationKind::SelectMessageById,
                    reason_code: DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE,
                    detail: error.to_string(),
                },
            )?;
        tx.commit().await.map_err(|error| {
            DataLayerPgExecutionAdapterError::SqlExecutionFailed {
                operation: DataLayerPgOperationKind::SelectMessageById,
                reason_code: DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE,
                detail: format!("commit failed: {error}"),
            }
        })?;

        row.map(data_layer_pg_decode_stored_message).transpose()
    }

    async fn begin_transaction(
        &self,
        operation: DataLayerPgOperationKind,
    ) -> Result<Transaction<'_, Postgres>, DataLayerPgExecutionAdapterError> {
        self.pool.begin().await.map_err(|error| {
            DataLayerPgExecutionAdapterError::SqlExecutionFailed {
                operation,
                reason_code: DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE,
                detail: format!("begin transaction failed: {error}"),
            }
        })
    }

    async fn apply_requester_session(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        session: &DataLayerPgRequesterSession,
        operation: DataLayerPgOperationKind,
    ) -> Result<(), DataLayerPgExecutionAdapterError> {
        sqlx::query("SELECT set_config($1, $2, TRUE);")
            .bind(session.setting_key)
            .bind(session.requester_did.as_str())
            .execute(&mut **transaction)
            .await
            .map_err(
                |error| DataLayerPgExecutionAdapterError::SqlExecutionFailed {
                    operation,
                    reason_code: DATA_LAYER_PG_EXECUTION_SESSION_FAILED_REASON_CODE,
                    detail: error.to_string(),
                },
            )?;
        Ok(())
    }
}

/// Collects migration SQL file names from the canonical migrations directory.
pub fn data_layer_pg_collect_migration_files(
) -> Result<Vec<String>, DataLayerPgExecutionAdapterError> {
    let migrations_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DATA_LAYER_PG_MIGRATIONS_DIR);
    let directory_iter = fs::read_dir(migrations_dir.as_path()).map_err(|error| {
        DataLayerPgExecutionAdapterError::MigrationIoFailed {
            reason_code: DATA_LAYER_PG_EXECUTION_MIGRATION_FAILED_REASON_CODE,
            detail: format!(
                "failed reading migrations directory {}: {error}",
                migrations_dir.display()
            ),
        }
    })?;

    let mut files = Vec::new();
    for entry in directory_iter {
        let entry = entry.map_err(
            |error| DataLayerPgExecutionAdapterError::MigrationIoFailed {
                reason_code: DATA_LAYER_PG_EXECUTION_MIGRATION_FAILED_REASON_CODE,
                detail: format!("failed reading migration directory entry: {error}"),
            },
        )?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|extension| extension == "sql") {
            let file_name = path
                .file_name()
                .map(|value| value.to_string_lossy().to_string())
                .ok_or(DataLayerPgExecutionAdapterError::MigrationIoFailed {
                    reason_code: DATA_LAYER_PG_EXECUTION_MIGRATION_FAILED_REASON_CODE,
                    detail: format!("invalid migration file path: {}", path.display()),
                })?;
            files.push(file_name);
        }
    }
    files.sort();
    Ok(files)
}

fn data_layer_pg_split_migration_statements(source: &str) -> Vec<String> {
    let mut sanitized = String::new();
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("--") {
            continue;
        }
        sanitized.push_str(line);
        sanitized.push('\n');
    }

    sanitized
        .split(';')
        .map(str::trim)
        .filter(|statement| !statement.is_empty())
        .filter(|statement| !statement.eq_ignore_ascii_case("BEGIN"))
        .filter(|statement| !statement.eq_ignore_ascii_case("COMMIT"))
        .map(|statement| format!("{statement};"))
        .collect()
}

fn data_layer_pg_decode_stored_message(
    row: sqlx::postgres::PgRow,
) -> Result<DataLayerPgStoredMessage, DataLayerPgExecutionAdapterError> {
    let envelope_ciphertext_bytes: Vec<u8> =
        row.try_get("envelope_ciphertext").map_err(|error| {
            DataLayerPgExecutionAdapterError::DecodeFailed {
                field: "envelope_ciphertext",
                detail: error.to_string(),
            }
        })?;
    let envelope_ciphertext = String::from_utf8(envelope_ciphertext_bytes).map_err(|error| {
        DataLayerPgExecutionAdapterError::DecodeFailed {
            field: "envelope_ciphertext",
            detail: error.to_string(),
        }
    })?;

    Ok(DataLayerPgStoredMessage {
        message_id: row.try_get("message_id").map_err(|error| {
            DataLayerPgExecutionAdapterError::DecodeFailed {
                field: "message_id",
                detail: error.to_string(),
            }
        })?,
        owner_did: row.try_get("owner_did").map_err(|error| {
            DataLayerPgExecutionAdapterError::DecodeFailed {
                field: "owner_did",
                detail: error.to_string(),
            }
        })?,
        sender_did: row.try_get("sender_did").map_err(|error| {
            DataLayerPgExecutionAdapterError::DecodeFailed {
                field: "sender_did",
                detail: error.to_string(),
            }
        })?,
        recipient_did: row.try_get("recipient_did").map_err(|error| {
            DataLayerPgExecutionAdapterError::DecodeFailed {
                field: "recipient_did",
                detail: error.to_string(),
            }
        })?,
        envelope_ciphertext,
        content_hash_sha256: row.try_get("content_hash_sha256").map_err(|error| {
            DataLayerPgExecutionAdapterError::DecodeFailed {
                field: "content_hash_sha256",
                detail: error.to_string(),
            }
        })?,
        hash_chain_prev: row.try_get("hash_chain_prev").map_err(|error| {
            DataLayerPgExecutionAdapterError::DecodeFailed {
                field: "hash_chain_prev",
                detail: error.to_string(),
            }
        })?,
        retention_class: row.try_get("retention_class").map_err(|error| {
            DataLayerPgExecutionAdapterError::DecodeFailed {
                field: "retention_class",
                detail: error.to_string(),
            }
        })?,
    })
}

/// Error taxonomy for live PostgreSQL adapter behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerPgExecutionAdapterError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// Pool max-connections configuration is invalid.
    InvalidMaxConnections(u32),
    /// Database URL failed validation.
    InvalidDatabaseUrl {
        /// Field name carrying the URL.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
        /// Parser detail.
        detail: String,
    },
    /// Bridge projection failed before SQL execution.
    BridgeProjectionFailed {
        /// Operation kind being projected.
        operation: DataLayerPgOperationKind,
        /// Error detail from bridge layer.
        detail: String,
    },
    /// SQL execution failed.
    SqlExecutionFailed {
        /// Operation that failed.
        operation: DataLayerPgOperationKind,
        /// Stable reason marker.
        reason_code: &'static str,
        /// SQL error detail.
        detail: String,
    },
    /// Migration discovery or migration-IO failed.
    MigrationIoFailed {
        /// Stable reason marker.
        reason_code: &'static str,
        /// IO error detail.
        detail: String,
    },
    /// Migration application failed.
    MigrationFailed {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Migration error detail.
        detail: String,
    },
    /// Row decoding failed.
    DecodeFailed {
        /// Field that failed to decode.
        field: &'static str,
        /// Decode error detail.
        detail: String,
    },
}

impl fmt::Display for DataLayerPgExecutionAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(formatter, "{field} must not be empty"),
            Self::InvalidMaxConnections(value) => {
                write!(formatter, "max_connections must be > 0 (got {value})")
            }
            Self::InvalidDatabaseUrl {
                field,
                reason_code,
                detail,
            } => write!(
                formatter,
                "invalid database url field {field}: {reason_code} ({detail})"
            ),
            Self::BridgeProjectionFailed { operation, detail } => {
                write!(
                    formatter,
                    "bridge projection failed for {operation:?}: {detail}"
                )
            }
            Self::SqlExecutionFailed {
                operation,
                reason_code,
                detail,
            } => write!(
                formatter,
                "sql execution failed for {operation:?}: {reason_code} ({detail})"
            ),
            Self::MigrationIoFailed {
                reason_code,
                detail,
            } => write!(formatter, "migration io failed: {reason_code} ({detail})"),
            Self::MigrationFailed {
                reason_code,
                detail,
            } => write!(formatter, "migration failed: {reason_code} ({detail})"),
            Self::DecodeFailed { field, detail } => {
                write!(formatter, "decode failed for {field}: {detail}")
            }
        }
    }
}

impl std::error::Error for DataLayerPgExecutionAdapterError {}

impl From<DataLayerPgRepositoryBridgeError> for DataLayerPgExecutionAdapterError {
    fn from(error: DataLayerPgRepositoryBridgeError) -> Self {
        Self::BridgeProjectionFailed {
            operation: DataLayerPgOperationKind::SelectMessageById,
            detail: error.to_string(),
        }
    }
}
