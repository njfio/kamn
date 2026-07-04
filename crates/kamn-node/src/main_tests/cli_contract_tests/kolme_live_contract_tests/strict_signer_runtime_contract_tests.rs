use super::super::*;
use crate::NodeCli;

#[test]
fn regression_kolme_live_strict_signer_contracts_reject_profile_selector_env_mismatch() {
    let _lock = lock_signer_env_guard();
    let _profile_env = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary"));
    let _primary_key = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    assert!(
        matches!(
            build_kolme_live_signer_adapter(Some("ops-primary"), Some("env-local")),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("runtime_signer_profile_selector_mismatch")
        ),
        "strict signer contracts must reject selector/env profile mismatch with deterministic reason code"
    );
}

#[test]
fn integration_runtime_kolme_live_strict_signer_contracts_fail_closed_before_network_on_selector_env_mismatch(
) {
    let _lock = lock_signer_env_guard();
    let _profile_env = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary"));
    let _primary_key = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let (base_url, requests) = spawn_kolme_live_mock_server(selector_env_replies());
    let args = strict_runtime_args(base_url.as_str());
    let parsed = parse_cli(args, "strict kolme-live args should parse");
    assert_strict_mismatch_failure(parsed);
    assert_zero_requests(
        requests,
        "strict selector/env mismatch should fail before any live network request",
    );
}

fn selector_env_replies() -> Vec<MockHttpReply> {
    vec![
        MockHttpReply::ok(r#"{"next_nonce":17,"account_id":"acct-live-processor"}"#),
        MockHttpReply::ok(
            r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"pending"}"#,
        ),
    ]
}

fn strict_runtime_args(base_url: &str) -> Vec<String> {
    with_pairs(
        strict_kolme_live_env_local_args(),
        &[("--kolme-live-base-url", base_url)],
    )
}

fn assert_strict_mismatch_failure(parsed: NodeCli) {
    assert!(
        matches!(
            execute(parsed),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("runtime_signer_profile_selector_mismatch")
        ),
        "runtime must fail closed before network submit with deterministic selector/env mismatch reason code"
    );
}

fn assert_zero_requests(requests: std::sync::Arc<std::sync::Mutex<Vec<String>>>, message: &str) {
    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(recorded_requests.len(), 0, "{message}");
}
