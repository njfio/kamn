use super::super::*;
use crate::NodeCli;

#[test]
fn regression_runtime_kolme_live_rejects_provider_marker_drift() {
    let _lock = lock_signer_env_guard();
    let _profile_env = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_env = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let (base_url, requests) = spawn_kolme_live_mock_server(provider_drift_replies());
    let args = runtime_kolme_live_args(base_url.as_str());
    let parsed = parse_cli(args, "kolme-live provider drift args should parse");
    assert_provider_drift_failure(parsed);
    assert_request_count(requests, 2, "provider drift should fail after nonce lookup and submit response mapping");
}

#[test]
fn regression_runtime_kolme_live_rejects_missing_signer_private_key_env() {
    let _lock = lock_signer_env_guard();
    let _profile_env = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_env = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![submit_final_reply()]);
    let parsed = parse_cli(
        runtime_kolme_live_args(base_url.as_str()),
        "kolme-live missing signer key args should parse",
    );
    assert!(
        matches!(
            execute(parsed),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX must be set for signer profile ops-primary")
        ),
        "runtime must fail closed when signer private key env is missing"
    );
}

fn runtime_kolme_live_args(base_url: &str) -> Vec<String> {
    with_pairs(
        kolme_live_declared_args(),
        &[
            ("--kolme-live-base-url", base_url),
            ("--kolme-live-signer-key-source", "env-local"),
            ("--output", "json"),
        ],
    )
}

fn provider_drift_replies() -> Vec<MockHttpReply> {
    vec![
        MockHttpReply::ok(r#"{"next_nonce":23,"account_id":"acct-2176"}"#),
        MockHttpReply::ok(
            r#"{"status":"submitted","provider":"unexpected-provider","commit_id":"kolme-commit:ab12cd34","finality":"final"}"#,
        ),
    ]
}

fn submit_final_reply() -> MockHttpReply {
    MockHttpReply::ok(
        r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"final"}"#,
    )
}

fn assert_provider_drift_failure(parsed: NodeCli) {
    assert!(
        matches!(
            execute(parsed),
            Err(ConfigError::RuntimeKolmeLive(message)) if message.contains("provider marker drift")
        ),
        "runtime must fail closed when provider marker drifts from configured hint"
    );
}

fn assert_request_count(requests: std::sync::Arc<std::sync::Mutex<Vec<String>>>, expected: usize, message: &str) {
    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(recorded_requests.len(), expected, "{message}");
}
