use kamn_core::{
    build_canonical_replay_evidence_bundle, BlockPipelineError, CanonicalCommitRecord,
    CanonicalCommitStore, FileCanonicalCommitStore, NodeRole, SqliteCanonicalCommitStore,
};
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn sample_canonical_record(height: u64, digest: &str, tx_id: &str) -> CanonicalCommitRecord {
    CanonicalCommitRecord {
        block_height: height,
        producer_role: NodeRole::Processor,
        payload_digest: digest.to_owned(),
        transaction_ids: vec![tx_id.to_owned()],
    }
}

fn temp_sqlite_path(tag: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    std::env::temp_dir().join(format!("kamn-canonical-commit-{tag}-{nonce}.sqlite"))
}

#[test]
fn unit_sqlite_canonical_commit_store_rejects_schema_version_mismatch() {
    let path = temp_sqlite_path("schema-mismatch");
    let _ = fs::remove_file(&path);

    let store = SqliteCanonicalCommitStore::new(path.clone()).expect("store should bootstrap");
    drop(store);

    let connection = Connection::open(&path).expect("sqlite file should open");
    connection
        .execute(
            "UPDATE kamn_store_entries SET entry_value = ?1 WHERE namespace = ?2 AND entry_key = ?3",
            (
                b"2".as_slice(),
                "canonical_commit_store_meta",
                "schema_version",
            ),
        )
        .expect("schema version row should be mutable in test");
    drop(connection);

    let result = SqliteCanonicalCommitStore::new(path.clone());
    assert!(
        matches!(result, Err(BlockPipelineError::CommitStore(message)) if message.contains("canonical_commit_store_sqlite_schema_mismatch")),
        "schema mismatch must fail closed with deterministic reason marker"
    );

    let _ = fs::remove_file(path);
}

#[test]
fn functional_sqlite_canonical_commit_store_persists_and_reloads_lineage() {
    let path = temp_sqlite_path("roundtrip");
    let _ = fs::remove_file(&path);
    let mut store = SqliteCanonicalCommitStore::new(path.clone()).expect("store should bootstrap");
    let first = sample_canonical_record(17, "digest-17", "tx-17");
    let second = sample_canonical_record(18, "digest-18", "tx-18");

    store
        .persist_canonical_commit(first.clone())
        .expect("first record should persist");
    store
        .persist_canonical_commit(second.clone())
        .expect("second record should persist");
    assert_eq!(
        store
            .list_canonical_commits()
            .expect("lineage should list after writes"),
        vec![first.clone(), second.clone()]
    );

    let restarted = SqliteCanonicalCommitStore::new(path.clone()).expect("store should reopen");
    assert_eq!(
        restarted
            .list_canonical_commits()
            .expect("lineage should list after restart"),
        vec![first, second]
    );

    let _ = fs::remove_file(path);
}

#[test]
fn integration_sqlite_canonical_commit_store_supports_restart_replay_evidence_validation() {
    let path = temp_sqlite_path("replay");
    let _ = fs::remove_file(&path);

    let mut first_store =
        SqliteCanonicalCommitStore::new(path.clone()).expect("store should bootstrap");
    first_store
        .persist_canonical_commit(sample_canonical_record(71, "digest-71", "tx-71"))
        .expect("first record should persist");
    first_store
        .persist_canonical_commit(sample_canonical_record(72, "digest-72", "tx-72"))
        .expect("second record should persist");
    let pre_restart = first_store
        .list_canonical_commits()
        .expect("pre-restart lineage should load");

    let restarted = SqliteCanonicalCommitStore::new(path.clone()).expect("store should reopen");
    let post_restart = restarted
        .list_canonical_commits()
        .expect("post-restart lineage should load");
    let evidence = build_canonical_replay_evidence_bundle(&pre_restart, &post_restart)
        .expect("restart replay lineage should validate");
    assert_eq!(evidence.continuity_status, "verified");
    assert_eq!(evidence.restart_boundary_block_height, 72);
    assert_eq!(evidence.replay_checkpoint_block_height, 72);

    let _ = fs::remove_file(path);
}

#[test]
fn regression_sqlite_canonical_commit_store_rejects_non_utf8_payload_bytes() {
    // Regression: #3580
    let path = temp_sqlite_path("corrupt-payload");
    let _ = fs::remove_file(&path);

    let mut store = SqliteCanonicalCommitStore::new(path.clone()).expect("store should bootstrap");
    store
        .persist_canonical_commit(sample_canonical_record(201, "digest-201", "tx-201"))
        .expect("record should persist");

    let connection = Connection::open(&path).expect("sqlite file should open");
    connection
        .execute(
            "UPDATE kamn_store_entries SET entry_value = ?1 WHERE namespace = ?2 AND entry_key = ?3",
            (
                vec![0xff, 0xfe, 0xfd],
                "canonical_commit_store_entries",
                "height:00000000000000000201",
            ),
        )
        .expect("test should tamper persisted payload");
    drop(connection);

    let restarted = SqliteCanonicalCommitStore::new(path.clone()).expect("store should reopen");
    let result = restarted.list_canonical_commits();
    assert!(
        matches!(result, Err(BlockPipelineError::CommitStore(message)) if message.contains("canonical_commit_store_sqlite_payload_not_utf8")),
        "non-utf8 payload bytes must fail closed with deterministic reason marker"
    );

    let _ = fs::remove_file(path);
}

#[test]
fn regression_file_store_reference_keeps_parity_with_sqlite_lineage_shape() {
    // Regression: #3580
    let path = temp_sqlite_path("file-store-parity").with_extension("log");
    let _ = fs::remove_file(&path);
    let mut store = FileCanonicalCommitStore::new(path.clone()).expect("file store should build");

    store
        .persist_canonical_commit(sample_canonical_record(1, "digest-1", "tx-1"))
        .expect("first file-backed record should persist");
    store
        .persist_canonical_commit(sample_canonical_record(2, "digest-2", "tx-2"))
        .expect("second file-backed record should persist");

    let records = store
        .list_canonical_commits()
        .expect("file-backed records should list");
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].block_height, 1);
    assert_eq!(records[1].block_height, 2);

    let _ = fs::remove_file(path);
}
