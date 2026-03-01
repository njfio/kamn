use kamn_snapshot_journal::{
    parse_snapshot_journal_record, parse_snapshot_journal_record_checked, SnapshotJournalParseError,
};

#[test]
fn integration_checked_parse_accepts_valid_record() {
    let line = r#"{"schema_version":"kamn.snapshot-journal.entry.v1","payload_hex":"616263"}"#;
    let payload_hex =
        parse_snapshot_journal_record_checked(line).expect("expected valid checked parse");
    assert_eq!(payload_hex, "616263");
    assert_eq!(
        parse_snapshot_journal_record(line),
        Some("616263".to_owned())
    );
}

#[test]
fn integration_checked_parse_rejects_invalid_json() {
    let line = "not-json";
    assert_eq!(
        parse_snapshot_journal_record_checked(line),
        Err(SnapshotJournalParseError::InvalidJson)
    );
    assert!(parse_snapshot_journal_record(line).is_none());
}

#[test]
fn integration_checked_parse_rejects_schema_mismatch() {
    let line = r#"{"schema_version":"kamn.snapshot-journal.entry.v2","payload_hex":"616263"}"#;
    assert_eq!(
        parse_snapshot_journal_record_checked(line),
        Err(SnapshotJournalParseError::SchemaVersionMismatch)
    );
    assert!(parse_snapshot_journal_record(line).is_none());
}

#[test]
fn integration_checked_parse_rejects_empty_payload_hex() {
    let line = r#"{"schema_version":"kamn.snapshot-journal.entry.v1","payload_hex":""}"#;
    assert_eq!(
        parse_snapshot_journal_record_checked(line),
        Err(SnapshotJournalParseError::MissingPayloadHex)
    );
    assert!(parse_snapshot_journal_record(line).is_none());
}
