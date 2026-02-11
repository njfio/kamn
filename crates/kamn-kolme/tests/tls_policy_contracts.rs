use kamn_kolme::{
    classify_tls_failure_reason, parse_tls_ca_file_env_value,
    resolve_tls_ca_file_env_result as resolve_tls_ca_file_env_result_contract, KolmeTlsPolicyError,
};

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

#[test]
fn functional_resolve_tls_ca_file_env_result_supports_present_and_absent_values() {
    let absent = resolve_tls_ca_file_env_result_contract(Err(std::env::VarError::NotPresent))
        .expect("not-present env var should resolve to none");
    assert_eq!(absent, None);

    let present =
        resolve_tls_ca_file_env_result_contract(Ok("/etc/ssl/certs/custom.pem".to_owned()))
            .expect("present env var should resolve");
    assert_eq!(present, Some("/etc/ssl/certs/custom.pem".to_owned()));
}

#[test]
fn regression_issue_1850_tls_env_resolution_fails_closed_on_non_unicode() {
    // Regression: #1850
    let error = resolve_tls_ca_file_env_result_contract(Err(std::env::VarError::NotUnicode(
        std::ffi::OsString::from("invalid"),
    )))
    .expect_err("non-unicode env value must fail closed");
    assert_eq!(
        error,
        KolmeTlsPolicyError::Unavailable {
            reason: "KAMN_KOLME_TLS_CA_FILE must be valid utf-8".to_owned(),
        }
    );
}
