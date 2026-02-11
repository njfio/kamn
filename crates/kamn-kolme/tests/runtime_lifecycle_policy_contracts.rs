use kamn_kolme::{
    commit_finality_label, is_terminal_receipt_finality as is_terminal_receipt_finality_contract,
    lifecycle_state_for_finality, lifecycle_state_label,
    require_final_receipt_finality as require_final_receipt_finality_contract,
    KolmeCommitReceiptFinality, RuntimeCommitLifecycleState, RuntimeLifecyclePolicyError,
};

#[test]
fn unit_runtime_lifecycle_policy_maps_finality_to_state_contract() {
    assert_eq!(
        lifecycle_state_for_finality(KolmeCommitReceiptFinality::Pending),
        RuntimeCommitLifecycleState::Pending
    );
    assert_eq!(
        lifecycle_state_for_finality(KolmeCommitReceiptFinality::Final),
        RuntimeCommitLifecycleState::Finalized
    );
    assert_eq!(
        lifecycle_state_for_finality(KolmeCommitReceiptFinality::Failed),
        RuntimeCommitLifecycleState::Failed
    );
}

#[test]
fn functional_runtime_lifecycle_policy_labels_are_deterministic() {
    assert_eq!(
        lifecycle_state_label(RuntimeCommitLifecycleState::Pending),
        "pending"
    );
    assert_eq!(
        lifecycle_state_label(RuntimeCommitLifecycleState::Finalized),
        "finalized"
    );
    assert_eq!(
        lifecycle_state_label(RuntimeCommitLifecycleState::Failed),
        "failed"
    );
    assert_eq!(
        commit_finality_label(KolmeCommitReceiptFinality::Pending),
        "pending"
    );
    assert_eq!(
        commit_finality_label(KolmeCommitReceiptFinality::Final),
        "final"
    );
    assert_eq!(
        commit_finality_label(KolmeCommitReceiptFinality::Failed),
        "failed"
    );
}

#[test]
fn regression_runtime_lifecycle_policy_state_label_drift_remains_fail_closed() {
    // Regression: #1775
    assert_eq!(
        lifecycle_state_label(lifecycle_state_for_finality(
            KolmeCommitReceiptFinality::Final
        )),
        "finalized"
    );
}

#[test]
fn functional_runtime_lifecycle_policy_requires_final_receipt_finality() {
    require_final_receipt_finality_contract(KolmeCommitReceiptFinality::Final)
        .expect("final receipt finality should pass");
}

#[test]
fn regression_issue_1844_runtime_lifecycle_policy_rejects_non_final_receipt_finality() {
    // Regression: #1844
    let error = require_final_receipt_finality_contract(KolmeCommitReceiptFinality::Pending)
        .expect_err("pending receipt finality must fail closed");
    assert_eq!(
        error,
        RuntimeLifecyclePolicyError::NonFinalReceipt {
            finality: KolmeCommitReceiptFinality::Pending,
        }
    );
}

#[test]
fn functional_runtime_lifecycle_policy_detects_terminal_receipt_finality() {
    assert!(!is_terminal_receipt_finality_contract(
        KolmeCommitReceiptFinality::Pending
    ));
    assert!(is_terminal_receipt_finality_contract(
        KolmeCommitReceiptFinality::Final
    ));
}

#[test]
fn regression_issue_1858_runtime_lifecycle_policy_treats_failed_as_terminal() {
    // Regression: #1858
    assert!(is_terminal_receipt_finality_contract(
        KolmeCommitReceiptFinality::Failed
    ));
}
