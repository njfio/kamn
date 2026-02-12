use kamn_kolme::{
    is_valid_http_response_bytes_input, parse_http_response_body, KolmeHttpResponsePolicyError,
};

#[test]
fn functional_parse_http_response_body_returns_body_for_2xx() {
    let body =
        parse_http_response_body(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello".to_vec())
            .expect("response should parse");
    assert_eq!(body, "hello");
}

#[test]
fn functional_http_response_policy_accepts_non_empty_response_bytes_input() {
    assert!(is_valid_http_response_bytes_input(
        b"HTTP/1.1 200 OK\r\n\r\n"
    ));
}

#[test]
fn regression_issue_1741_http_response_parser_fails_closed_on_length_mismatch() {
    // Regression: #1741
    let error =
        parse_http_response_body(b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\nhello".to_vec())
            .expect_err("content-length mismatch must fail");
    assert_eq!(
        error,
        KolmeHttpResponsePolicyError::Malformed {
            reason: "http response content-length mismatch: declared 4, observed 5".to_owned(),
        }
    );
}

#[test]
fn regression_issue_1898_http_response_policy_rejects_empty_response_bytes_input() {
    // Regression: #1898
    assert!(!is_valid_http_response_bytes_input(&[]));
}
