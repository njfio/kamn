use super::super::*;

#[test]
fn regression_3599_startup_signer_mode_negative_matrix_corpus() {
    let _lock = lock_signer_env_guard();
    let covered_cases = vec![
        strict_missing_signer_profile_case(),
        strict_missing_signer_key_source_case(),
        daemon_shutdown_controls_missing_signal_case(),
        strict_selector_env_mismatch_preflight_case(),
        fallback_secret_preflight_case(),
    ];
    let expected_cases = vec![
        "strict-missing-signer-profile",
        "strict-missing-signer-key-source",
        "daemon-shutdown-controls-missing-signal",
        "strict-selector-env-mismatch-preflight",
        "fallback-secret-preflight",
    ];
    assert_eq!(covered_cases, expected_cases, "startup_negative_matrix_policy_marker_missing");
}

fn strict_missing_signer_profile_case() -> &'static str {
    assert_parse_error(
        with_pairs(strict_kolme_live_args(), &[("--kolme-live-signer-key-source", "env-local")]),
        ConfigError::MissingArgumentValue("--kolme-live-signer-profile"),
    );
    "strict-missing-signer-profile"
}

fn strict_missing_signer_key_source_case() -> &'static str {
    assert_parse_error(
        with_pairs(strict_kolme_live_args(), &[("--kolme-live-signer-profile", "ops-primary")]),
        ConfigError::MissingArgumentValue("--kolme-live-signer-key-source"),
    );
    "strict-missing-signer-key-source"
}

fn daemon_shutdown_controls_missing_signal_case() -> &'static str {
    assert_parse_error(
        with_pairs(
            daemon_args(),
            &[
                ("--daemon-max-ticks", "12"),
                ("--daemon-tick-interval-ms", "5"),
                ("--daemon-shutdown-drain-ticks", "2"),
                ("--daemon-shutdown-timeout-ticks", "4"),
            ],
        ),
        ConfigError::MissingArgumentValue("--daemon-shutdown-signal-tick"),
    );
    "daemon-shutdown-controls-missing-signal"
}

fn strict_selector_env_mismatch_preflight_case() -> &'static str {
    let _profile_env = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary"));
    let _primary_key = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let _fallback_key = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
    let _override = EnvVarGuard::set("KAMN_KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING", None);
    let (base_url, requests) = spawn_kolme_live_mock_server(selector_env_replies());
    let parsed = parse_cli(strict_runtime_args(base_url.as_str()), "strict mismatch args should parse");
    assert!(
        matches!(
            execute(parsed),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("strict signer profile mismatch")
        ),
        "matrix case strict-selector-env-mismatch-preflight must fail closed before network"
    );
    assert_no_live_requests(requests, "strict selector/env mismatch must fail before network");
    "strict-selector-env-mismatch-preflight"
}

fn fallback_secret_preflight_case() -> &'static str {
    let _profile_env = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _primary_key = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let _fallback_key = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY),
    );
    let (base_url, requests) = spawn_kolme_live_mock_server(vec![
        MockHttpReply::ok(r#"{"next_nonce":17,"account_id":"acct-live-processor"}"#),
    ]);
    let parsed = parse_cli(strict_runtime_args(base_url.as_str()), "fallback-secret args should parse");
    assert!(
        matches!(
            execute(parsed),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("fallback_signer_secret_present_violation")
        ),
        "matrix case fallback-secret-preflight must fail closed with deterministic reason code"
    );
    assert_no_live_requests(requests, "fallback secret path must fail before network dispatch");
    "fallback-secret-preflight"
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

fn assert_no_live_requests(requests: std::sync::Arc<std::sync::Mutex<Vec<String>>>, message: &str) {
    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(recorded_requests.len(), 0, "{message}");
}
