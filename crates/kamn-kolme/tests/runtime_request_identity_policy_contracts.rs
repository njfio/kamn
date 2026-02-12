use kamn_kolme::{
    are_runtime_commit_request_fields_single_line, deterministic_runtime_commit_id,
    deterministic_runtime_commit_idempotency_key, is_canonical_runtime_commit_signed_message,
    is_valid_runtime_commit_id_request, is_valid_runtime_nonce_input,
    is_valid_runtime_operation_id_input, is_valid_runtime_payload_hash_input,
    is_valid_runtime_state_root_input, normalize_runtime_commit_signed_envelope_fields,
    render_runtime_commit_wire_payload,
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
fn functional_runtime_request_identity_policy_accepts_positive_nonce_input() {
    assert!(is_valid_runtime_nonce_input(1));
}

#[test]
fn regression_issue_1892_runtime_request_identity_policy_rejects_empty_request_fields() {
    // Regression: #1892
    assert!(!is_valid_runtime_operation_id_input(" "));
    assert!(!is_valid_runtime_state_root_input(" "));
    assert!(!is_valid_runtime_payload_hash_input(" "));
}

#[test]
fn functional_runtime_request_identity_policy_accepts_single_line_request_fields() {
    assert!(are_runtime_commit_request_fields_single_line(
        "op-9",
        "state:beta",
        "payload:beta"
    ));
}

#[test]
fn functional_runtime_request_identity_policy_accepts_canonical_signed_message_match() {
    let canonical = "operation_id=op-9\nstate_root=state:beta\n";
    assert!(is_canonical_runtime_commit_signed_message(
        canonical, canonical
    ));
}

#[test]
fn functional_runtime_request_identity_policy_normalizes_signed_envelope_fields() {
    let normalized = normalize_runtime_commit_signed_envelope_fields(
        " signer-key-2 ",
        " message-body ",
        " signature-body ",
    );
    assert_eq!(
        normalized,
        (
            "signer-key-2".to_owned(),
            "message-body".to_owned(),
            "signature-body".to_owned(),
        )
    );
}

#[test]
fn functional_runtime_request_identity_policy_renders_runtime_commit_wire_payload() {
    let payload = render_runtime_commit_wire_payload(
        "op-12",
        "state:delta",
        "did:kamn:agent:delta",
        8,
        "payload:delta",
        "idempotency:delta",
    );
    assert_eq!(
        payload,
        "operation_id=op-12\nstate_root=state:delta\nactor_did=did:kamn:agent:delta\nnonce=8\npayload_hash=payload:delta\nidempotency_key=idempotency:delta\n"
    );
}

#[test]
fn regression_issue_1894_runtime_request_identity_policy_rejects_multiline_request_fields() {
    // Regression: #1894
    assert!(!are_runtime_commit_request_fields_single_line(
        "op-9\nwrapped",
        "state:beta",
        "payload:beta"
    ));
    assert!(!are_runtime_commit_request_fields_single_line(
        "op-9",
        "state:beta\nwrapped",
        "payload:beta"
    ));
    assert!(!are_runtime_commit_request_fields_single_line(
        "op-9",
        "state:beta",
        "payload:beta\nwrapped"
    ));
}

#[test]
fn regression_issue_1900_runtime_request_identity_policy_rejects_zero_nonce_input() {
    // Regression: #1900
    assert!(!is_valid_runtime_nonce_input(0));
}

#[test]
fn regression_issue_1902_runtime_request_identity_policy_rejects_noncanonical_signed_message() {
    // Regression: #1902
    let canonical = "operation_id=op-9\nstate_root=state:beta\n";
    let tampered = "operation_id=op-9\nstate_root=state:tampered\n";
    assert!(!is_canonical_runtime_commit_signed_message(
        canonical, tampered
    ));
}

#[test]
fn regression_issue_1904_runtime_request_identity_policy_preserves_inner_whitespace() {
    // Regression: #1904
    let normalized = normalize_runtime_commit_signed_envelope_fields(
        "\t signer-key-2\t",
        " message value with spaces ",
        "\n signature value \n",
    );
    assert_eq!(
        normalized,
        (
            "signer-key-2".to_owned(),
            "message value with spaces".to_owned(),
            "signature value".to_owned(),
        )
    );
}

#[test]
fn regression_issue_1906_runtime_request_identity_policy_wire_payload_field_order_is_stable() {
    // Regression: #1906
    let payload = render_runtime_commit_wire_payload(
        "op-12",
        "state:delta",
        "did:kamn:agent:delta",
        8,
        "payload:delta",
        "idempotency:delta",
    );
    assert!(payload.starts_with("operation_id=op-12\nstate_root=state:delta\n"));
    assert!(payload.ends_with("idempotency_key=idempotency:delta\n"));
}
