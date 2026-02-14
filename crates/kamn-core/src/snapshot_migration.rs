//! File-to-sqlite snapshot migration and parity-check contracts.

use crate::{
    ChannelSnapshotStore, DurableGuardBundleSnapshotStore, FileChannelSnapshotStore,
    FileDurableGuardSnapshotStore, FileMessageLifecycleSnapshotStore, FileRuntimeSnapshotStore,
    FileTaskOperationSnapshotStore, MessageLifecycleSnapshotStore, RuntimeSnapshotStore,
    SqliteChannelSnapshotStore, SqliteDurableGuardSnapshotStore,
    SqliteMessageLifecycleSnapshotStore, SqliteRuntimeSnapshotStore,
    SqliteTaskOperationSnapshotStore, TaskOperationSnapshotStore,
};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;

const CHANNEL_STORE_FILE_NAME: &str = "channel.snapshot";
const MESSAGE_LIFECYCLE_STORE_FILE_NAME: &str = "message-lifecycle.snapshot";
const TASK_OPERATION_STORE_FILE_NAME: &str = "task-operation.snapshot";
const DURABLE_GUARD_STORE_FILE_NAME: &str = "durable-guard.snapshot";
const RUNTIME_STORE_FILE_NAME: &str = "runtime.snapshot";

const CHANNEL_STORE_DOMAIN: &str = "channel-snapshot-store";
const MESSAGE_LIFECYCLE_STORE_DOMAIN: &str = "message-lifecycle-snapshot-store";
const TASK_OPERATION_STORE_DOMAIN: &str = "task-operation-snapshot-store";
const DURABLE_GUARD_STORE_DOMAIN: &str = "durable-guard-snapshot-store";
const RUNTIME_STORE_DOMAIN: &str = "runtime-snapshot-store";

const REASON_STORAGE_ROOT_INVALID: &str = "snapshot_migration_storage_root_invalid";
const REASON_SQLITE_PATH_INVALID: &str = "snapshot_migration_sqlite_path_invalid";
const REASON_LEGACY_STORE_LOAD_FAILED: &str = "snapshot_migration_legacy_store_load_failed";
const REASON_SQLITE_STORE_WRITE_FAILED: &str = "snapshot_migration_sqlite_store_write_failed";
const REASON_SQLITE_STORE_READ_FAILED: &str = "snapshot_migration_sqlite_store_read_failed";
const REASON_PARITY_MISMATCH: &str = "snapshot_migration_parity_mismatch";
const REASON_PARITY_PASS: &str = "snapshot_migration_parity_pass";

#[derive(Debug, Clone, PartialEq, Eq)]
/// File-to-sqlite migration parity report.
pub struct SnapshotMigrationParityReport {
    /// Deterministic terminal reason code.
    pub reason_code: &'static str,
    /// Ordered domain identifiers migrated during this run.
    pub migrated_domains: Vec<&'static str>,
    /// Number of migrated snapshot records across all domains.
    pub migrated_snapshot_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// File-to-sqlite migration error taxonomy.
pub enum SnapshotMigrationError {
    /// Source storage root is missing, invalid, or not directory-backed.
    InvalidStorageRoot {
        /// Deterministic fail-closed reason code.
        reason_code: &'static str,
        /// Error detail for audit/debug traces.
        detail: String,
    },
    /// Sqlite path selector is invalid.
    InvalidSqlitePath {
        /// Deterministic fail-closed reason code.
        reason_code: &'static str,
        /// Error detail for audit/debug traces.
        detail: String,
    },
    /// Legacy file-backed store failed to load or parse.
    LegacyStoreLoad {
        /// Logical store domain.
        domain: &'static str,
        /// Deterministic fail-closed reason code.
        reason_code: &'static str,
        /// Error detail for audit/debug traces.
        detail: String,
    },
    /// Sqlite store failed while writing migrated records.
    SqliteStoreWrite {
        /// Logical store domain.
        domain: &'static str,
        /// Deterministic fail-closed reason code.
        reason_code: &'static str,
        /// Error detail for audit/debug traces.
        detail: String,
    },
    /// Sqlite store failed while reading parity records.
    SqliteStoreRead {
        /// Logical store domain.
        domain: &'static str,
        /// Deterministic fail-closed reason code.
        reason_code: &'static str,
        /// Error detail for audit/debug traces.
        detail: String,
    },
    /// Roundtrip parity mismatch between legacy snapshot payload and sqlite result.
    ParityMismatch {
        /// Logical store domain.
        domain: &'static str,
        /// Deterministic fail-closed reason code.
        reason_code: &'static str,
        /// Error detail for audit/debug traces.
        detail: String,
    },
}

impl SnapshotMigrationError {
    /// Returns deterministic reason code for fail-closed lane contracts.
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::InvalidStorageRoot { reason_code, .. }
            | Self::InvalidSqlitePath { reason_code, .. }
            | Self::LegacyStoreLoad { reason_code, .. }
            | Self::SqliteStoreWrite { reason_code, .. }
            | Self::SqliteStoreRead { reason_code, .. }
            | Self::ParityMismatch { reason_code, .. } => reason_code,
        }
    }
}

impl Display for SnapshotMigrationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidStorageRoot { detail, .. } => write!(f, "invalid storage root: {detail}"),
            Self::InvalidSqlitePath { detail, .. } => write!(f, "invalid sqlite path: {detail}"),
            Self::LegacyStoreLoad { domain, detail, .. } => {
                write!(f, "failed to load legacy store {domain}: {detail}")
            }
            Self::SqliteStoreWrite { domain, detail, .. } => {
                write!(f, "failed to write sqlite store {domain}: {detail}")
            }
            Self::SqliteStoreRead { domain, detail, .. } => {
                write!(f, "failed to read sqlite store {domain}: {detail}")
            }
            Self::ParityMismatch { domain, detail, .. } => {
                write!(f, "parity mismatch in {domain}: {detail}")
            }
        }
    }
}

impl Error for SnapshotMigrationError {}

/// Migrates legacy file snapshots into sqlite and enforces roundtrip parity.
pub fn migrate_file_snapshots_to_sqlite_parity(
    storage_root: &Path,
    sqlite_path: &Path,
) -> Result<SnapshotMigrationParityReport, SnapshotMigrationError> {
    if !storage_root.exists() || !storage_root.is_dir() {
        return Err(SnapshotMigrationError::InvalidStorageRoot {
            reason_code: REASON_STORAGE_ROOT_INVALID,
            detail: format!("storage root must exist and be a directory: {storage_root:?}"),
        });
    }
    if sqlite_path.as_os_str().is_empty() {
        return Err(SnapshotMigrationError::InvalidSqlitePath {
            reason_code: REASON_SQLITE_PATH_INVALID,
            detail: "sqlite path must not be empty".to_owned(),
        });
    }

    let mut migrated_domains = Vec::new();
    let mut migrated_snapshot_count = 0_usize;

    let channel_file_store = FileChannelSnapshotStore::new(
        storage_root.join(CHANNEL_STORE_FILE_NAME),
    )
    .map_err(|error| SnapshotMigrationError::LegacyStoreLoad {
        domain: CHANNEL_STORE_DOMAIN,
        reason_code: REASON_LEGACY_STORE_LOAD_FAILED,
        detail: error.to_string(),
    })?;
    let channel_snapshot = channel_file_store.read_latest().map_err(|error| {
        SnapshotMigrationError::LegacyStoreLoad {
            domain: CHANNEL_STORE_DOMAIN,
            reason_code: REASON_LEGACY_STORE_LOAD_FAILED,
            detail: error.to_string(),
        }
    })?;
    if let Some(snapshot) = channel_snapshot {
        let mut sqlite_store =
            SqliteChannelSnapshotStore::new(sqlite_path.to_path_buf()).map_err(|error| {
                SnapshotMigrationError::SqliteStoreWrite {
                    domain: CHANNEL_STORE_DOMAIN,
                    reason_code: REASON_SQLITE_STORE_WRITE_FAILED,
                    detail: error.to_string(),
                }
            })?;
        sqlite_store.write(snapshot.clone()).map_err(|error| {
            SnapshotMigrationError::SqliteStoreWrite {
                domain: CHANNEL_STORE_DOMAIN,
                reason_code: REASON_SQLITE_STORE_WRITE_FAILED,
                detail: error.to_string(),
            }
        })?;
        let sqlite_snapshot = sqlite_store.read_latest().map_err(|error| {
            SnapshotMigrationError::SqliteStoreRead {
                domain: CHANNEL_STORE_DOMAIN,
                reason_code: REASON_SQLITE_STORE_READ_FAILED,
                detail: error.to_string(),
            }
        })?;
        if sqlite_snapshot != Some(snapshot) {
            return Err(SnapshotMigrationError::ParityMismatch {
                domain: CHANNEL_STORE_DOMAIN,
                reason_code: REASON_PARITY_MISMATCH,
                detail: "sqlite channel snapshot does not match legacy payload".to_owned(),
            });
        }
        migrated_domains.push(CHANNEL_STORE_DOMAIN);
        migrated_snapshot_count += 1;
    }

    let message_file_store = FileMessageLifecycleSnapshotStore::new(
        storage_root.join(MESSAGE_LIFECYCLE_STORE_FILE_NAME),
    )
    .map_err(|error| SnapshotMigrationError::LegacyStoreLoad {
        domain: MESSAGE_LIFECYCLE_STORE_DOMAIN,
        reason_code: REASON_LEGACY_STORE_LOAD_FAILED,
        detail: error.to_string(),
    })?;
    let message_snapshot = message_file_store.read_latest().map_err(|error| {
        SnapshotMigrationError::LegacyStoreLoad {
            domain: MESSAGE_LIFECYCLE_STORE_DOMAIN,
            reason_code: REASON_LEGACY_STORE_LOAD_FAILED,
            detail: error.to_string(),
        }
    })?;
    if let Some(snapshot) = message_snapshot {
        let mut sqlite_store = SqliteMessageLifecycleSnapshotStore::new(sqlite_path.to_path_buf())
            .map_err(|error| SnapshotMigrationError::SqliteStoreWrite {
                domain: MESSAGE_LIFECYCLE_STORE_DOMAIN,
                reason_code: REASON_SQLITE_STORE_WRITE_FAILED,
                detail: error.to_string(),
            })?;
        sqlite_store.write(snapshot.clone()).map_err(|error| {
            SnapshotMigrationError::SqliteStoreWrite {
                domain: MESSAGE_LIFECYCLE_STORE_DOMAIN,
                reason_code: REASON_SQLITE_STORE_WRITE_FAILED,
                detail: error.to_string(),
            }
        })?;
        let sqlite_snapshot = sqlite_store.read_latest().map_err(|error| {
            SnapshotMigrationError::SqliteStoreRead {
                domain: MESSAGE_LIFECYCLE_STORE_DOMAIN,
                reason_code: REASON_SQLITE_STORE_READ_FAILED,
                detail: error.to_string(),
            }
        })?;
        if sqlite_snapshot != Some(snapshot) {
            return Err(SnapshotMigrationError::ParityMismatch {
                domain: MESSAGE_LIFECYCLE_STORE_DOMAIN,
                reason_code: REASON_PARITY_MISMATCH,
                detail: "sqlite message lifecycle snapshot does not match legacy payload"
                    .to_owned(),
            });
        }
        migrated_domains.push(MESSAGE_LIFECYCLE_STORE_DOMAIN);
        migrated_snapshot_count += 1;
    }

    let task_file_store =
        FileTaskOperationSnapshotStore::new(storage_root.join(TASK_OPERATION_STORE_FILE_NAME))
            .map_err(|error| SnapshotMigrationError::LegacyStoreLoad {
                domain: TASK_OPERATION_STORE_DOMAIN,
                reason_code: REASON_LEGACY_STORE_LOAD_FAILED,
                detail: error.to_string(),
            })?;
    let task_snapshot =
        task_file_store
            .read_latest()
            .map_err(|error| SnapshotMigrationError::LegacyStoreLoad {
                domain: TASK_OPERATION_STORE_DOMAIN,
                reason_code: REASON_LEGACY_STORE_LOAD_FAILED,
                detail: error.to_string(),
            })?;
    if let Some(snapshot) = task_snapshot {
        let mut sqlite_store = SqliteTaskOperationSnapshotStore::new(sqlite_path.to_path_buf())
            .map_err(|error| SnapshotMigrationError::SqliteStoreWrite {
                domain: TASK_OPERATION_STORE_DOMAIN,
                reason_code: REASON_SQLITE_STORE_WRITE_FAILED,
                detail: error.to_string(),
            })?;
        sqlite_store.write(snapshot.clone()).map_err(|error| {
            SnapshotMigrationError::SqliteStoreWrite {
                domain: TASK_OPERATION_STORE_DOMAIN,
                reason_code: REASON_SQLITE_STORE_WRITE_FAILED,
                detail: error.to_string(),
            }
        })?;
        let sqlite_snapshot = sqlite_store.read_latest().map_err(|error| {
            SnapshotMigrationError::SqliteStoreRead {
                domain: TASK_OPERATION_STORE_DOMAIN,
                reason_code: REASON_SQLITE_STORE_READ_FAILED,
                detail: error.to_string(),
            }
        })?;
        if sqlite_snapshot != Some(snapshot) {
            return Err(SnapshotMigrationError::ParityMismatch {
                domain: TASK_OPERATION_STORE_DOMAIN,
                reason_code: REASON_PARITY_MISMATCH,
                detail: "sqlite task operation snapshot does not match legacy payload".to_owned(),
            });
        }
        migrated_domains.push(TASK_OPERATION_STORE_DOMAIN);
        migrated_snapshot_count += 1;
    }

    let durable_file_store =
        FileDurableGuardSnapshotStore::new(storage_root.join(DURABLE_GUARD_STORE_FILE_NAME))
            .map_err(|error| SnapshotMigrationError::LegacyStoreLoad {
                domain: DURABLE_GUARD_STORE_DOMAIN,
                reason_code: REASON_LEGACY_STORE_LOAD_FAILED,
                detail: error.to_string(),
            })?;
    let durable_bundle = durable_file_store.load_bundle().map_err(|error| {
        SnapshotMigrationError::LegacyStoreLoad {
            domain: DURABLE_GUARD_STORE_DOMAIN,
            reason_code: REASON_LEGACY_STORE_LOAD_FAILED,
            detail: error.to_string(),
        }
    })?;
    if let Some(bundle) = durable_bundle {
        let mut sqlite_store = SqliteDurableGuardSnapshotStore::new(sqlite_path.to_path_buf())
            .map_err(|error| SnapshotMigrationError::SqliteStoreWrite {
                domain: DURABLE_GUARD_STORE_DOMAIN,
                reason_code: REASON_SQLITE_STORE_WRITE_FAILED,
                detail: error.to_string(),
            })?;
        sqlite_store.save_bundle(bundle.clone()).map_err(|error| {
            SnapshotMigrationError::SqliteStoreWrite {
                domain: DURABLE_GUARD_STORE_DOMAIN,
                reason_code: REASON_SQLITE_STORE_WRITE_FAILED,
                detail: error.to_string(),
            }
        })?;
        let sqlite_bundle = sqlite_store.load_bundle().map_err(|error| {
            SnapshotMigrationError::SqliteStoreRead {
                domain: DURABLE_GUARD_STORE_DOMAIN,
                reason_code: REASON_SQLITE_STORE_READ_FAILED,
                detail: error.to_string(),
            }
        })?;
        if sqlite_bundle != Some(bundle) {
            return Err(SnapshotMigrationError::ParityMismatch {
                domain: DURABLE_GUARD_STORE_DOMAIN,
                reason_code: REASON_PARITY_MISMATCH,
                detail: "sqlite durable guard bundle does not match legacy payload".to_owned(),
            });
        }
        migrated_domains.push(DURABLE_GUARD_STORE_DOMAIN);
        migrated_snapshot_count += 1;
    }

    let runtime_file_store = FileRuntimeSnapshotStore::new(
        storage_root.join(RUNTIME_STORE_FILE_NAME),
    )
    .map_err(|error| SnapshotMigrationError::LegacyStoreLoad {
        domain: RUNTIME_STORE_DOMAIN,
        reason_code: REASON_LEGACY_STORE_LOAD_FAILED,
        detail: error.to_string(),
    })?;
    let runtime_snapshots =
        runtime_file_store
            .list()
            .map_err(|error| SnapshotMigrationError::LegacyStoreLoad {
                domain: RUNTIME_STORE_DOMAIN,
                reason_code: REASON_LEGACY_STORE_LOAD_FAILED,
                detail: error.to_string(),
            })?;
    if !runtime_snapshots.is_empty() {
        let mut sqlite_store =
            SqliteRuntimeSnapshotStore::new(sqlite_path.to_path_buf()).map_err(|error| {
                SnapshotMigrationError::SqliteStoreWrite {
                    domain: RUNTIME_STORE_DOMAIN,
                    reason_code: REASON_SQLITE_STORE_WRITE_FAILED,
                    detail: error.to_string(),
                }
            })?;
        for snapshot in &runtime_snapshots {
            sqlite_store.write(snapshot.clone()).map_err(|error| {
                SnapshotMigrationError::SqliteStoreWrite {
                    domain: RUNTIME_STORE_DOMAIN,
                    reason_code: REASON_SQLITE_STORE_WRITE_FAILED,
                    detail: error.to_string(),
                }
            })?;
        }
        let sqlite_snapshots =
            sqlite_store
                .list()
                .map_err(|error| SnapshotMigrationError::SqliteStoreRead {
                    domain: RUNTIME_STORE_DOMAIN,
                    reason_code: REASON_SQLITE_STORE_READ_FAILED,
                    detail: error.to_string(),
                })?;
        if sqlite_snapshots != runtime_snapshots {
            return Err(SnapshotMigrationError::ParityMismatch {
                domain: RUNTIME_STORE_DOMAIN,
                reason_code: REASON_PARITY_MISMATCH,
                detail: "sqlite runtime snapshots do not match legacy payload".to_owned(),
            });
        }
        migrated_domains.push(RUNTIME_STORE_DOMAIN);
        migrated_snapshot_count += runtime_snapshots.len();
    }

    Ok(SnapshotMigrationParityReport {
        reason_code: REASON_PARITY_PASS,
        migrated_domains,
        migrated_snapshot_count,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        migrate_file_snapshots_to_sqlite_parity, SnapshotMigrationError,
        REASON_SQLITE_PATH_INVALID, REASON_STORAGE_ROOT_INVALID,
    };
    use std::path::PathBuf;

    #[test]
    fn unit_migration_rejects_missing_storage_root() {
        let missing_root = PathBuf::from("/tmp/kamn-nonexistent-migration-root");
        let sqlite_path = PathBuf::from("/tmp/kamn-migration.sqlite");
        let result =
            migrate_file_snapshots_to_sqlite_parity(missing_root.as_path(), sqlite_path.as_path());
        assert!(
            matches!(
                result,
                Err(SnapshotMigrationError::InvalidStorageRoot { reason_code, .. })
                    if reason_code == REASON_STORAGE_ROOT_INVALID
            ),
            "missing source storage root must fail closed"
        );
    }

    #[test]
    fn unit_migration_rejects_empty_sqlite_path() {
        let root = std::env::temp_dir();
        let result =
            migrate_file_snapshots_to_sqlite_parity(root.as_path(), PathBuf::new().as_path());
        assert!(
            matches!(
                result,
                Err(SnapshotMigrationError::InvalidSqlitePath { reason_code, .. })
                    if reason_code == REASON_SQLITE_PATH_INVALID
            ),
            "empty sqlite path must fail closed"
        );
    }
}
