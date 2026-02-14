use kamn_core::{SqliteStoreBackend, SqliteStoreBackendError, SQLITE_STORE_SCHEMA_VERSION};
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_sqlite_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos();
    std::env::temp_dir().join(format!("kamn-{name}-{unique}.db"))
}

#[test]
fn functional_sqlite_store_bootstraps_and_round_trips_entries() {
    let sqlite_path = temp_sqlite_path("sqlite-store-roundtrip");
    let mut backend =
        SqliteStoreBackend::open(&sqlite_path).expect("sqlite backend should initialize");

    assert_eq!(backend.schema_version(), SQLITE_STORE_SCHEMA_VERSION);

    backend
        .put("runtime_snapshot", "snapshot:1", b"payload-v1")
        .expect("put should succeed");
    backend
        .put("runtime_snapshot", "snapshot:2", b"payload-v2")
        .expect("second put should succeed");

    let first = backend
        .get("runtime_snapshot", "snapshot:1")
        .expect("get should succeed");
    assert_eq!(first, Some(b"payload-v1".to_vec()));

    let keys = backend
        .list_keys("runtime_snapshot")
        .expect("list should succeed");
    assert_eq!(keys, vec!["snapshot:1".to_owned(), "snapshot:2".to_owned()]);

    let deleted = backend
        .delete("runtime_snapshot", "snapshot:1")
        .expect("delete should succeed");
    assert!(deleted, "delete should report removed row");
    assert_eq!(
        backend
            .get("runtime_snapshot", "snapshot:1")
            .expect("get after delete should succeed"),
        None
    );

    let _ = fs::remove_file(sqlite_path);
}

#[test]
fn regression_sqlite_store_fails_closed_on_schema_version_mismatch() {
    let sqlite_path = temp_sqlite_path("sqlite-store-schema-mismatch");
    let backend = SqliteStoreBackend::open(&sqlite_path).expect("sqlite backend should initialize");
    drop(backend);
    let connection = Connection::open(&sqlite_path).expect("must open sqlite database");
    connection
        .execute(
            "UPDATE kamn_store_meta SET value = ?1 WHERE key = 'schema_version'",
            [format!("{}", SQLITE_STORE_SCHEMA_VERSION + 1)],
        )
        .expect("must tamper schema version row");
    drop(connection);

    let error = SqliteStoreBackend::open(&sqlite_path)
        .expect_err("schema mismatch must fail closed on open");
    assert!(
        matches!(
            error,
            SqliteStoreBackendError::SchemaVersionMismatch {
                expected: SQLITE_STORE_SCHEMA_VERSION,
                found
            } if found == SQLITE_STORE_SCHEMA_VERSION + 1
        ),
        "schema mismatch should return deterministic typed error"
    );

    let _ = fs::remove_file(sqlite_path);
}
