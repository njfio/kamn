use kamn_kolme::{parse_commit_receipt_finality, KolmeCommitReceiptFinality, ReceiptFinalityError};

#[test]
fn unit_commit_finality_parse_maps_aliases_to_commit_finality_contract() {
    assert_eq!(
        parse_commit_receipt_finality("pending").expect("pending should parse"),
        KolmeCommitReceiptFinality::Pending
    );
    assert_eq!(
        parse_commit_receipt_finality("finalized").expect("finalized should parse"),
        KolmeCommitReceiptFinality::Final
    );
    assert_eq!(
        parse_commit_receipt_finality("failed").expect("failed should parse"),
        KolmeCommitReceiptFinality::Failed
    );
}

#[test]
fn functional_commit_finality_parse_accepts_confirmed_alias() {
    assert_eq!(
        parse_commit_receipt_finality("confirmed").expect("confirmed alias should parse"),
        KolmeCommitReceiptFinality::Final
    );
}

#[test]
fn regression_commit_finality_parse_rejects_unknown_values_fail_closed() {
    // Regression: #1783
    assert_eq!(
        parse_commit_receipt_finality("settled"),
        Err(ReceiptFinalityError::InvalidFinalityValue(
            "settled".to_owned()
        ))
    );
}
