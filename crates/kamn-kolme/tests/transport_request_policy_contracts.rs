use kamn_kolme::{
    is_broadcast_submit_path, is_valid_transport_idempotency_key_input,
    parse_authorization_header_value, KolmeTransportRequestPolicyError,
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
