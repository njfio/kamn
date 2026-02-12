use kamn_kolme::{
    is_broadcast_submit_path, is_valid_transport_idempotency_key_input,
    is_valid_transport_wire_payload_input, normalize_broadcast_submit_path_input,
    normalize_transport_idempotency_key_input, parse_authorization_header_value,
    KolmeTransportRequestPolicyError,
};

#[test]
fn functional_parse_authorization_header_value_trims_whitespace() {
    let parsed = parse_authorization_header_value("  Bearer abc123  ")
        .expect("authorization header should parse");
    assert_eq!(parsed, "Bearer abc123");
}

#[test]
fn functional_is_broadcast_submit_path_accepts_query_and_trailing_slash() {
    assert!(is_broadcast_submit_path("/broadcast"));
    assert!(is_broadcast_submit_path("/broadcast/"));
    assert!(is_broadcast_submit_path("/broadcast?mode=sync"));
}

#[test]
fn functional_transport_request_policy_accepts_non_empty_idempotency_key_input() {
    assert!(is_valid_transport_idempotency_key_input(
        "kolme-runtime-commit:op-1"
    ));
}

#[test]
fn functional_transport_request_policy_accepts_non_empty_wire_payload_input() {
    assert!(is_valid_transport_wire_payload_input("operation_id=op-1\n"));
}

#[test]
fn functional_transport_request_policy_normalizes_broadcast_submit_path_input() {
    assert_eq!(
        normalize_broadcast_submit_path_input(" /broadcast "),
        "/broadcast"
    );
}

#[test]
fn functional_transport_request_policy_normalizes_transport_idempotency_key_input() {
    assert_eq!(
        normalize_transport_idempotency_key_input("  kolme-runtime-commit:op-1  "),
        "kolme-runtime-commit:op-1"
    );
}

#[test]
fn regression_issue_1755_parse_authorization_header_value_rejects_empty_header() {
    // Regression: #1755
    let error = parse_authorization_header_value("  ").expect_err("empty header must fail");
    assert_eq!(
        error,
        KolmeTransportRequestPolicyError::InvalidRequest {
            field: "transport_authorization_header",
            reason: "must not be empty",
        }
    );
}

#[test]
fn regression_issue_1755_parse_authorization_header_value_rejects_crlf() {
    // Regression: #1755
    let error = parse_authorization_header_value("Bearer token\r\nX-Injected: 1")
        .expect_err("CR/LF injection must fail");
    assert_eq!(
        error,
        KolmeTransportRequestPolicyError::InvalidRequest {
            field: "transport_authorization_header",
            reason: "must be single-line",
        }
    );
}

#[test]
fn regression_issue_1755_is_broadcast_submit_path_rejects_non_broadcast_paths() {
    // Regression: #1755
    assert!(!is_broadcast_submit_path(""));
    assert!(!is_broadcast_submit_path("/runtime-commit/submit"));
    assert!(!is_broadcast_submit_path("/broadcasting"));
}

#[test]
fn regression_issue_1884_transport_request_policy_rejects_empty_idempotency_key_input() {
    // Regression: #1884
    assert!(!is_valid_transport_idempotency_key_input(" \t "));
}

#[test]
fn regression_issue_1886_transport_request_policy_rejects_empty_wire_payload_input() {
    // Regression: #1886
    assert!(!is_valid_transport_wire_payload_input(" \t "));
}

#[test]
fn regression_issue_1888_transport_request_policy_defaults_empty_submit_path_to_broadcast() {
    // Regression: #1888
    assert_eq!(normalize_broadcast_submit_path_input(" "), "/broadcast");
}

#[test]
fn regression_issue_1912_transport_request_policy_trims_outer_idempotency_whitespace() {
    // Regression: #1912
    assert_eq!(
        normalize_transport_idempotency_key_input("\tkey:1912\t"),
        "key:1912"
    );
}
