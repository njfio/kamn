use kamn_kolme::{classify_tls_failure_reason, parse_tls_ca_file_env_value, KolmeTlsPolicyError};

#[test]
fn functional_tls_failure_reason_classifier_detects_certificate_errors() {
    let reason = classify_tls_failure_reason(
        "verify error:num=18:self-signed certificate\ncertificate verify failed",
    );
    assert_eq!(reason, "tls certificate verification failed");
}

#[test]
fn regression_issue_1743_tls_ca_env_parser_fails_closed_on_empty_value() {
    // Regression: #1743
    let error = parse_tls_ca_file_env_value(Some("  ")).expect_err("empty CA env value must fail");
    assert_eq!(
        error,
        KolmeTlsPolicyError::Unavailable {
            reason: "KAMN_KOLME_TLS_CA_FILE must not be empty".to_owned(),
        }
    );
}
