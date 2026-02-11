use kamn_kolme::{
    parse_receipt_finality, render_block_path, validate_block_identity,
    validate_block_path_template, validate_lookup_window, ReceiptFinality,
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
