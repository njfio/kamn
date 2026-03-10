use crate::support::parse_fixture;

#[test]
fn regression_partial_write_taxonomy_and_matrix_rows_remain_stable() {
    let (metadata, cases) = parse_fixture().expect("fixture should parse");
    assert_metadata(&metadata);
    let observed_case_ids: Vec<String> = cases.into_iter().map(|case| case.case_id).collect();
    assert_case_ids(observed_case_ids);
}

fn assert_metadata(metadata: &crate::support::FixtureMetadata) {
    assert_eq!(
        metadata.schema_version,
        "kamn.runtime.journal-wal-partial-write-fault-matrix.v1"
    );
    assert_eq!(
        metadata.reason_taxonomy_version,
        "kamn.runtime.journal-wal-partial-write-reason-taxonomy.v1"
    );
    assert_eq!(
        metadata.reason_codes_csv,
        "partial_snapshot_file_write_recovered_from_journal,partial_journal_tail_write_fail_closed,partial_snapshot_without_journal_repaired"
    );
    assert_eq!(
        metadata.columns,
        "case_id|store|fault_mode|expected_outcome|expected_marker"
    );
}

fn assert_case_ids(observed_case_ids: Vec<String>) {
    assert_eq!(
        observed_case_ids,
        vec![
            "channel_partial_snapshot_file_write",
            "message_partial_snapshot_file_write",
            "task_partial_snapshot_file_write",
            "channel_partial_journal_tail_write",
            "message_partial_journal_tail_write",
            "task_partial_journal_tail_write",
            "channel_partial_snapshot_without_journal",
            "message_partial_snapshot_without_journal",
            "task_partial_snapshot_without_journal",
        ]
    );
}
