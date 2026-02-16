const SIGNER_SOURCE: &str = include_str!("../src/signer.rs");
const KOLME_RUNTIME_COMMIT_DOC: &str =
    include_str!("../../../docs/architecture/kolme-runtime-commit.md");
const CI_STRATEGY_DOC: &str = include_str!("../../../docs/ci/strategy.md");

#[test]
fn source_declares_signer_decode_zeroization_markers() {
    let decoded_zeroize_count = SIGNER_SOURCE.matches("decoded.zeroize();").count();
    assert!(
        decoded_zeroize_count >= 2,
        "signer source must keep explicit decoded buffer zeroization calls"
    );
    assert!(
        SIGNER_SOURCE.contains("private_key_bytes.zeroize();"),
        "signer source must keep private key byte buffer zeroization"
    );
    assert!(
        SIGNER_SOURCE.contains("private_key_hex.zeroize();"),
        "signer source must keep private key hex buffer zeroization"
    );
    assert!(
        SIGNER_SOURCE
            .contains("regression_signer_private_key_decode_failure_redacts_sensitive_input"),
        "signer source must keep decode-failure redaction regression coverage"
    );
}

#[test]
fn docs_runtime_commit_declares_signer_decode_zeroization_contracts() {
    assert!(
        KOLME_RUNTIME_COMMIT_DOC.contains("### Signer Key Decode Zeroization Guarantees"),
        "runtime-commit docs must declare signer decode zeroization section"
    );
    assert!(
        KOLME_RUNTIME_COMMIT_DOC.contains("signer_decode_zeroization_contract_version=v1"),
        "runtime-commit docs must declare signer decode zeroization version marker"
    );
    assert!(
        KOLME_RUNTIME_COMMIT_DOC
            .contains("signer_decode_error_redaction_policy=raw_private_key_value_never_emitted"),
        "runtime-commit docs must declare signer decode redaction policy marker"
    );
    assert!(
        KOLME_RUNTIME_COMMIT_DOC.contains(
            "cargo test -p kamn-node --test signer_secret_hygiene_contract -- --nocapture"
        ),
        "runtime-commit docs must declare signer secret-hygiene contract command"
    );
}

#[test]
fn docs_ci_strategy_declares_signer_secret_redaction_regression_policy() {
    assert!(
        CI_STRATEGY_DOC.contains("### Signer Secret Redaction Regression Guard"),
        "ci strategy docs must declare signer secret redaction guard section"
    );
    assert!(
        CI_STRATEGY_DOC.contains("signer_secret_redaction_regression_guard_status=active"),
        "ci strategy docs must declare signer redaction guard status marker"
    );
    assert!(
        CI_STRATEGY_DOC
            .contains("signer_secret_redaction_policy=raw_private_key_value_never_emitted"),
        "ci strategy docs must declare signer redaction policy marker"
    );
    assert!(
        CI_STRATEGY_DOC.contains(
            "cargo test -p kamn-node signer::tests::regression_signer_private_key_decode_failure_redacts_sensitive_input -- --exact --nocapture"
        ),
        "ci strategy docs must include decode-failure redaction regression command"
    );
}
