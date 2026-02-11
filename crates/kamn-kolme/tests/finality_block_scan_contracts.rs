use kamn_kolme::{
    parse_fork_block_txhash, parse_receipt_finality,
    project_failed_block_txhash_receipt as project_failed_block_txhash_receipt_contract,
    project_finalized_block_txhash_receipt as project_finalized_block_txhash_receipt_contract,
    render_block_path, resolve_lookup_upper_bound, validate_block_identity,
    validate_block_path_template, validate_lookup_window, BlockScanReceiptProjection,
    ReceiptFinality,
};

#[test]
fn unit_parse_receipt_finality_supports_runtime_aliases() {
    assert_eq!(
        parse_receipt_finality("pending").expect("pending alias should parse"),
        ReceiptFinality::Pending
    );
    assert_eq!(
        parse_receipt_finality("confirmed").expect("confirmed alias should parse"),
        ReceiptFinality::Final
    );
    assert_eq!(
        parse_receipt_finality("rejected").expect("rejected alias should parse"),
        ReceiptFinality::Failed
    );
}

#[test]
fn functional_validate_lookup_window_rejects_stale_span() {
    let error = validate_lookup_window(40, 44, 3).expect_err("stale span must fail");
    assert_eq!(
        error.to_string(),
        "block fallback window exceeds max lookups: from_height=40 latest_height=44 max_lookups=3"
    );
}

#[test]
fn functional_parse_fork_block_txhash_extracts_required_field() {
    assert_eq!(
        parse_fork_block_txhash(r#"{"height":42,"txhash":"0xabc123"}"#)
            .expect("txhash should parse"),
        "0xabc123"
    );
}

#[test]
fn regression_issue_1720_block_identity_mismatch_fails_closed() {
    // Regression: #1720
    let error = validate_block_identity("kolme-fork-local", "tampered-provider", 42, 99)
        .expect_err("provider/height mismatch must fail");
    assert_eq!(
        error.to_string(),
        "block fallback provider mismatch: expected kolme-fork-local observed tampered-provider"
    );

    assert!(
        validate_block_path_template("/block/{height}").is_ok(),
        "block path template should accept height placeholder"
    );
    assert_eq!(
        render_block_path("/block/{height}", 42).expect("render should work"),
        "/block/42"
    );
}

#[test]
fn functional_resolve_lookup_upper_bound_honors_notification_height_within_window() {
    let upper_bound = resolve_lookup_upper_bound(40, 45, 42);
    assert_eq!(upper_bound, 42);
}

#[test]
fn regression_issue_1840_resolve_lookup_upper_bound_falls_back_to_latest_when_notification_stale() {
    // Regression: #1840
    let upper_bound = resolve_lookup_upper_bound(40, 45, 39);
    assert_eq!(upper_bound, 45);
}

#[test]
fn functional_project_finalized_block_txhash_receipt_maps_commit_id_and_finality() {
    let receipt = project_finalized_block_txhash_receipt_contract("ab12cd34", 72);
    assert_eq!(
        receipt,
        BlockScanReceiptProjection {
            commit_id: "kolme-commit:ab12cd34:h72".to_owned(),
            finality: ReceiptFinality::Final,
        }
    );
}

#[test]
fn regression_issue_1854_project_failed_block_txhash_receipt_uses_heightless_commit_id() {
    // Regression: #1854
    let receipt = project_failed_block_txhash_receipt_contract("ab12cd34");
    assert_eq!(
        receipt,
        BlockScanReceiptProjection {
            commit_id: "kolme-commit:ab12cd34".to_owned(),
            finality: ReceiptFinality::Failed,
        }
    );
}
