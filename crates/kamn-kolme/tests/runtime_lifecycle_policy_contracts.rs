use kamn_kolme::{
    commit_finality_label, lifecycle_state_for_finality, lifecycle_state_label,
    KolmeCommitReceiptFinality, RuntimeCommitLifecycleState,
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
