use kamn_kolme::{
    commit_finality_from_receipt_finality, commit_finality_label, parse_receipt_finality,
    KolmeCommitReceiptFinality, ReceiptFinality,
};

#[test]
fn unit_receipt_to_commit_finality_mapping_contract() {
    assert_eq!(
        commit_finality_from_receipt_finality(ReceiptFinality::Pending),
        KolmeCommitReceiptFinality::Pending
    );
    assert_eq!(
        commit_finality_from_receipt_finality(ReceiptFinality::Final),
        KolmeCommitReceiptFinality::Final
    );
    assert_eq!(
        commit_finality_from_receipt_finality(ReceiptFinality::Failed),
        KolmeCommitReceiptFinality::Failed
    );
}

#[test]
fn functional_receipt_aliases_map_to_deterministic_commit_labels() {
    let accepted = parse_receipt_finality("accepted").expect("accepted alias should parse");
    let finalized = parse_receipt_finality("finalized").expect("finalized alias should parse");
    let failed = parse_receipt_finality("failed").expect("failed alias should parse");

    assert_eq!(
        commit_finality_label(commit_finality_from_receipt_finality(accepted)),
        "pending"
    );
    assert_eq!(
        commit_finality_label(commit_finality_from_receipt_finality(finalized)),
        "final"
    );
    assert_eq!(
        commit_finality_label(commit_finality_from_receipt_finality(failed)),
        "failed"
    );
}

#[test]
fn regression_receipt_to_commit_mapping_remains_parity_locked() {
    // Regression: #1779
    let pending = parse_receipt_finality("pending").expect("pending alias should parse");
    assert_eq!(
        commit_finality_from_receipt_finality(pending),
        KolmeCommitReceiptFinality::Pending
    );
}
