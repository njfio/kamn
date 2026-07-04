use super::super::*;

#[test]
fn rejects_kolme_live_strict_signer_contracts_without_signer_profile_selector() {
    assert_parse_error(
        with_pairs(
            strict_kolme_live_args(),
            &[("--kolme-live-signer-key-source", "env-local")],
        ),
        ConfigError::MissingArgumentValue("--kolme-live-signer-profile"),
    );
}

#[test]
fn rejects_kolme_live_strict_signer_contracts_without_key_source() {
    assert_parse_error(
        with_pairs(
            strict_kolme_live_args(),
            &[("--kolme-live-signer-profile", "ops-primary")],
        ),
        ConfigError::MissingArgumentValue("--kolme-live-signer-key-source"),
    );
}

#[test]
fn parses_kolme_live_strict_signer_contracts_with_managed_external_key_source() {
    parse_args(managed_external_strict_kolme_live_args())
        .expect("strict signer contract declarations should parse managed-external markers");
}

#[test]
fn parses_kolme_live_strict_signer_contracts_with_explicit_declarations() {
    parse_args(strict_kolme_live_env_local_args())
        .expect("strict signer contract declarations should parse");
}

#[test]
fn rejects_kolme_live_strict_signer_contracts_with_empty_signer_profile_selector() {
    let args = with_pairs(
        strict_kolme_live_args(),
        &[
            ("--kolme-live-signer-profile", " "),
            ("--kolme-live-signer-key-source", "env-local"),
        ],
    );
    assert_runtime_kolme_live_error(args, "--kolme-live-signer-profile must not be empty");
}

#[test]
fn rejects_kolme_live_strict_signer_contracts_with_empty_key_source() {
    let args = with_pairs(
        strict_kolme_live_args(),
        &[
            ("--kolme-live-signer-profile", "ops-primary"),
            ("--kolme-live-signer-key-source", " "),
        ],
    );
    assert_runtime_kolme_live_error(args, "--kolme-live-signer-key-source must not be empty");
}

fn assert_runtime_kolme_live_error(args: Vec<String>, needle: &str) {
    assert!(
        matches!(
            parse_args(args),
            Err(ConfigError::RuntimeKolmeLive(message)) if message.contains(needle)
        ),
        "strict signer contracts must fail closed with expected runtime kolm-live error: {needle}"
    );
}
