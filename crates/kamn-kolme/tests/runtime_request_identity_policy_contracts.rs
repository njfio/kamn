use kamn_kolme::{
    deterministic_runtime_commit_id, deterministic_runtime_commit_idempotency_key,
    is_valid_runtime_commit_id_request, is_valid_runtime_operation_id_input,
    is_valid_runtime_payload_hash_input, is_valid_runtime_state_root_input,
};

#[test]
fn unit_runtime_request_identity_policy_idempotency_key_contract() {
    let idempotency_key = deterministic_runtime_commit_idempotency_key(
        " operation-123 ",
        " state:abc ",
        " did:kamn:agent:alpha ",
        7,
        " payload-hash ",
    );
    assert_eq!(
        idempotency_key,
        "kolme-runtime-commit:operation-123:state:abc:did:kamn:agent:alpha:7:12"
    );
}

#[test]
fn functional_runtime_request_identity_policy_commit_id_contract() {
    let commit_id = deterministic_runtime_commit_id("op-9", "did:agent:beta", 11, "hash:xyz");
    assert_eq!(commit_id, "kolme-commit:op-9:did:agent:beta:11:8");
}

#[test]
fn regression_runtime_request_identity_policy_payload_length_drift_remains_fail_closed() {
    // Regression: #1777
    let left = deterministic_runtime_commit_id("op-x", "did:agent", 3, "abc");
    let right = deterministic_runtime_commit_id("op-x", "did:agent", 3, "xyz");
    assert_eq!(left, right);
}

#[test]
fn functional_runtime_request_identity_policy_accepts_non_empty_commit_id_request() {
    assert!(is_valid_runtime_commit_id_request(
        "kolme-commit:op-9:did:agent:beta:11:8"
    ));
}

#[test]
fn regression_issue_1862_runtime_request_identity_policy_rejects_empty_commit_id_request() {
    // Regression: #1862
    assert!(!is_valid_runtime_commit_id_request(""));
    assert!(!is_valid_runtime_commit_id_request("   "));
}

#[test]
fn functional_runtime_request_identity_policy_accepts_non_empty_request_fields() {
    assert!(is_valid_runtime_operation_id_input("op-9"));
    assert!(is_valid_runtime_state_root_input("state:beta"));
    assert!(is_valid_runtime_payload_hash_input("payload:beta"));
}

#[test]
fn regression_issue_1892_runtime_request_identity_policy_rejects_empty_request_fields() {
    // Regression: #1892
    assert!(!is_valid_runtime_operation_id_input(" "));
    assert!(!is_valid_runtime_state_root_input(" "));
    assert!(!is_valid_runtime_payload_hash_input(" "));
}
