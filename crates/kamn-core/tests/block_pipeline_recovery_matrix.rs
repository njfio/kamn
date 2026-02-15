use kamn_core::{
    build_canonical_replay_evidence_bundle, BlockPipelineError, CanonicalCommitRecord,
    CanonicalCommitStore, FileCanonicalCommitStore, NodeRole, SqliteCanonicalCommitStore,
};
use rusqlite::Connection;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn sample_canonical_record(height: u64, digest: &str, tx_id: &str) -> CanonicalCommitRecord {
    CanonicalCommitRecord {
        block_height: height,
        producer_role: NodeRole::Processor,
        payload_digest: digest.to_owned(),
        transaction_ids: vec![tx_id.to_owned()],
    }
}

fn temp_path(tag: &str, extension: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    std::env::temp_dir().join(format!("kamn-canonical-recovery-{tag}-{nonce}.{extension}"))
}

#[test]
fn unit_block_pipeline_error_reason_code_parses_commit_store_marker() {
    let error = BlockPipelineError::CommitStore(
        "canonical commit sqlite schema mismatch: expected 1, found 2 (canonical_commit_store_sqlite_schema_mismatch)"
            .to_owned(),
    );
    assert_eq!(
        error.reason_code(),
        "canonical_commit_store_sqlite_schema_mismatch"
    );
}

#[test]
fn functional_recovery_matrix_validates_restart_behavior_for_file_and_sqlite_stores() {
    let file_path = temp_path("functional-file", "log");
    let sqlite_path = temp_path("functional-sqlite", "sqlite");
    let _ = fs::remove_file(&file_path);
    let _ = fs::remove_file(&sqlite_path);

    let mut file_store =
        FileCanonicalCommitStore::new(file_path.clone()).expect("file store should build");
    let mut sqlite_store =
        SqliteCanonicalCommitStore::new(sqlite_path.clone()).expect("sqlite store should build");

    for record in [
        sample_canonical_record(31, "digest-31", "tx-31"),
        sample_canonical_record(32, "digest-32", "tx-32"),
    ] {
        file_store
            .persist_canonical_commit(record.clone())
            .expect("file record should persist");
        sqlite_store
            .persist_canonical_commit(record)
            .expect("sqlite record should persist");
    }

    let restarted_file =
        FileCanonicalCommitStore::new(file_path.clone()).expect("file store should reopen");
    let restarted_sqlite =
        SqliteCanonicalCommitStore::new(sqlite_path.clone()).expect("sqlite store should reopen");

    let file_pre = file_store
        .list_canonical_commits()
        .expect("file pre-restart lineage should load");
    let file_post = restarted_file
        .list_canonical_commits()
        .expect("file post-restart lineage should load");
    let sqlite_pre = sqlite_store
        .list_canonical_commits()
        .expect("sqlite pre-restart lineage should load");
    let sqlite_post = restarted_sqlite
        .list_canonical_commits()
        .expect("sqlite post-restart lineage should load");

    let file_evidence = build_canonical_replay_evidence_bundle(&file_pre, &file_post)
        .expect("file replay evidence should validate");
    let sqlite_evidence = build_canonical_replay_evidence_bundle(&sqlite_pre, &sqlite_post)
        .expect("sqlite replay evidence should validate");
    assert_eq!(file_evidence.continuity_status, "verified");
    assert_eq!(sqlite_evidence.continuity_status, "verified");

    let _ = fs::remove_file(file_path);
    let _ = fs::remove_file(sqlite_path);
}

#[test]
fn integration_recovery_matrix_rejects_stale_file_tail_with_stable_reason_code() {
    let file_path = temp_path("stale-tail", "log");
    let _ = fs::remove_file(&file_path);

    let mut store = FileCanonicalCommitStore::new(file_path.clone()).expect("store should build");
    store
        .persist_canonical_commit(sample_canonical_record(90, "digest-90", "tx-90"))
        .expect("baseline record should persist");

    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(&file_path)
        .expect("file should open for stale-tail tamper");
    file.write_all(b"stale-tail-without-delimiters\n")
        .expect("stale tail should append");

    let restarted = FileCanonicalCommitStore::new(file_path.clone()).expect("store should reopen");
    let error = restarted
        .list_canonical_commits()
        .expect_err("stale tail must fail closed");
    assert_eq!(
        error.reason_code(),
        "canonical_commit_store_record_malformed"
    );

    let _ = fs::remove_file(file_path);
}

#[test]
fn regression_recovery_matrix_rejects_sqlite_schema_artifact_drift_with_reason_codes() {
    // Regression: #3581
    let path = temp_path("schema-artifact-drift", "sqlite");
    let _ = fs::remove_file(&path);

    let mut store = SqliteCanonicalCommitStore::new(path.clone()).expect("store should build");
    store
        .persist_canonical_commit(sample_canonical_record(120, "digest-120", "tx-120"))
        .expect("record should persist");

    let connection = Connection::open(&path).expect("sqlite database should open");
    connection
        .execute(
            "DELETE FROM kamn_store_entries WHERE namespace = ?1 AND entry_key = ?2",
            ("canonical_commit_store_meta", "schema_version"),
        )
        .expect("schema marker row should delete");
    drop(connection);

    let missing_schema_error = SqliteCanonicalCommitStore::new(path.clone())
        .expect_err("missing schema marker must fail closed");
    assert_eq!(
        missing_schema_error.reason_code(),
        "canonical_commit_store_sqlite_schema_missing"
    );

    let repaired_store = SqliteCanonicalCommitStore::new(path.clone())
        .expect_err("schema marker remains missing while entries exist");
    assert_eq!(
        repaired_store.reason_code(),
        "canonical_commit_store_sqlite_schema_missing"
    );

    let _ = fs::remove_file(path);
}

#[test]
fn performance_recovery_matrix_replay_validation_stays_within_local_budget() {
    let path = temp_path("performance", "sqlite");
    let _ = fs::remove_file(&path);

    let mut store = SqliteCanonicalCommitStore::new(path.clone()).expect("store should build");
    for index in 1..=256 {
        store
            .persist_canonical_commit(sample_canonical_record(
                index,
                format!("digest-{index}").as_str(),
                format!("tx-{index}").as_str(),
            ))
            .expect("record should persist");
    }
    let pre_restart = store
        .list_canonical_commits()
        .expect("pre-restart lineage should load");
    let restarted = SqliteCanonicalCommitStore::new(path.clone()).expect("store should reopen");
    let post_restart = restarted
        .list_canonical_commits()
        .expect("post-restart lineage should load");

    let start = Instant::now();
    let evidence = build_canonical_replay_evidence_bundle(&pre_restart, &post_restart)
        .expect("replay evidence should validate");
    assert_eq!(evidence.post_restart_commit_count, 256);
    assert!(
        start.elapsed() <= Duration::from_secs(2),
        "recovery replay validation exceeded local budget"
    );

    let _ = fs::remove_file(path);
}
