use super::*;

#[test]
fn regression_runtime_kolme_live_rejects_provider_marker_drift() {
    // Regression: #2176
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _env_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let (base_url, requests) = spawn_kolme_live_mock_server(vec![
        MockHttpReply::ok(r#"{"next_nonce":23,"account_id":"acct-2176"}"#),
        MockHttpReply::ok(
            r#"{"status":"submitted","provider":"unexpected-provider","commit_id":"kolme-commit:ab12cd34","finality":"final"}"#,
        ),
    ]);
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        base_url,
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];
    let parsed = parse_args(args).expect("kolme-live args should parse");
    assert!(
        matches!(
            execute(parsed),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("provider marker drift")
        ),
        "runtime must fail closed when provider marker drifts from configured hint"
    );
    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        2,
        "provider drift should fail after nonce lookup and submit response mapping"
    );
}

#[test]
fn regression_runtime_kolme_live_rejects_missing_signer_private_key_env() {
    // Regression: #2220
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _env_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"final"}"#,
    )]);
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        base_url,
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];
    let parsed = parse_args(args).expect("kolme-live args should parse");
    assert!(
        matches!(
            execute(parsed),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX must be set for signer profile ops-primary")
        ),
        "runtime must fail closed when signer private key env is missing"
    );
}

#[test]
fn rejects_missing_role() {
    let args = vec!["kamn-node".to_owned()];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue("--role"))
    );
}

#[test]
fn rejects_planning_without_expected_state_hash() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "planning".to_owned(),
        "--proposal".to_owned(),
        "tx-1|did:kamn:agent:aaa|1|state-1".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue("--expected-state-hash"))
    );
}

#[test]
fn rejects_planning_without_proposal() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "planning".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-1".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue("--proposal"))
    );
}

#[test]
fn rejects_recovery_check_without_expected_state_version() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "recovery-check".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-42".to_owned(),
        "--rejoin-attempt".to_owned(),
        "node-a|42|state-42|resume-1".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue(
            "--expected-state-version"
        ))
    );
}

#[test]
fn rejects_recovery_check_without_expected_state_hash() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "recovery-check".to_owned(),
        "--expected-state-version".to_owned(),
        "42".to_owned(),
        "--rejoin-attempt".to_owned(),
        "node-a|42|state-42|resume-1".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue("--expected-state-hash"))
    );
}

#[test]
fn rejects_recovery_check_without_rejoin_attempt() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "recovery-check".to_owned(),
        "--expected-state-version".to_owned(),
        "42".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-42".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue("--rejoin-attempt"))
    );
}

#[test]
fn rejects_daemon_without_max_ticks() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue("--daemon-max-ticks"))
    );
}

#[test]
fn rejects_daemon_without_tick_interval() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue(
            "--daemon-tick-interval-ms"
        ))
    );
}

#[test]
fn rejects_full_without_max_ticks() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "full".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:19083".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue("--daemon-max-ticks"))
    );
}

#[test]
fn rejects_full_without_api_bind() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "full".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue("--api-bind"))
    );
}

#[test]
fn rejects_daemon_shutdown_signal_without_drain_ticks() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "8".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "3".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "2".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue(
            "--daemon-shutdown-drain-ticks"
        ))
    );
}

#[test]
fn rejects_daemon_shutdown_signal_without_timeout_ticks() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "8".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "3".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "2".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue(
            "--daemon-shutdown-timeout-ticks"
        ))
    );
}

#[test]
fn rejects_daemon_shutdown_controls_without_signal_tick() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "8".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "2".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "4".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue(
            "--daemon-shutdown-signal-tick"
        ))
    );
}

#[test]
fn rejects_kolme_live_without_base_url() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue("--kolme-live-base-url"))
    );
}

#[test]
fn rejects_kolme_live_without_provider_hint() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue(
            "--kolme-live-provider-hint"
        ))
    );
}

#[test]
fn rejects_kolme_live_without_signing_profile() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue(
            "--kolme-live-signing-profile"
        ))
    );
}

#[test]
fn rejects_kolme_live_without_signer_key_source() {
    // Regression: #2626
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue(
            "--kolme-live-signer-key-source"
        ))
    );
}

#[test]
fn rejects_kolme_live_continuous_mode_without_tick_interval() {
    // Regression: #2931
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "2".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue(
            "--daemon-tick-interval-ms"
        ))
    );
}

#[test]
fn rejects_kolme_live_continuous_mode_without_max_ticks() {
    // Regression: #2931
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue("--daemon-max-ticks"))
    );
}

#[test]
fn rejects_kolme_live_strict_signer_contracts_without_signer_profile_selector() {
    // Regression: #2246
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-strict-signer-contracts".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue(
            "--kolme-live-signer-profile"
        ))
    );
}

#[test]
fn rejects_kolme_live_strict_signer_contracts_without_key_source() {
    // Regression: #2246
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-strict-signer-contracts".to_owned(),
        "--kolme-live-signer-profile".to_owned(),
        "ops-primary".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue(
            "--kolme-live-signer-key-source"
        ))
    );
}

#[test]
fn parses_kolme_live_strict_signer_contracts_with_managed_external_key_source() {
    // Regression: #2322
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-strict-signer-contracts".to_owned(),
        "--kolme-live-signer-profile".to_owned(),
        "ops-primary".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "managed-external".to_owned(),
    ];
    parse_args(args)
        .expect("strict signer contract declarations should parse managed-external markers");
}

#[test]
fn parses_kolme_live_strict_signer_contracts_with_explicit_declarations() {
    // Regression: #2246
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-strict-signer-contracts".to_owned(),
        "--kolme-live-signer-profile".to_owned(),
        "ops-primary".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
    ];
    parse_args(args).expect("strict signer contract declarations should parse");
}

#[test]
fn rejects_kolme_live_strict_signer_contracts_with_empty_signer_profile_selector() {
    // Regression: #2247
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-strict-signer-contracts".to_owned(),
        "--kolme-live-signer-profile".to_owned(),
        " ".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
    ];
    assert!(
        matches!(
            parse_args(args),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("--kolme-live-signer-profile must not be empty")
        ),
        "strict signer contracts must reject empty signer profile selector"
    );
}

#[test]
fn rejects_kolme_live_strict_signer_contracts_with_empty_key_source() {
    // Regression: #2247
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-strict-signer-contracts".to_owned(),
        "--kolme-live-signer-profile".to_owned(),
        "ops-primary".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        " ".to_owned(),
    ];
    assert!(
        matches!(
            parse_args(args),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("--kolme-live-signer-key-source must not be empty")
        ),
        "strict signer contracts must reject empty key-source declaration"
    );
}

#[test]
fn regression_kolme_live_strict_signer_contracts_reject_profile_selector_env_mismatch() {
    // Regression: #2247
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary"));
    let _primary_key_guard = EnvVarGuard::set(
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
    // Regression: #2247
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary"));
    let _primary_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let (base_url, requests) = spawn_kolme_live_mock_server(vec![
        MockHttpReply::ok(r#"{"next_nonce":17,"account_id":"acct-live-processor"}"#),
        MockHttpReply::ok(
            r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"pending"}"#,
        ),
    ]);
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        base_url,
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-strict-signer-contracts".to_owned(),
        "--kolme-live-signer-profile".to_owned(),
        "ops-primary".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
    ];
    let parsed = parse_args(args).expect("strict kolme-live args should parse");
    assert!(
        matches!(
            execute(parsed),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("runtime_signer_profile_selector_mismatch")
        ),
        "runtime must fail closed before network submit with deterministic selector/env mismatch reason code"
    );
    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        0,
        "strict selector/env mismatch should fail before any live network request"
    );
}

#[test]
fn regression_3599_startup_signer_mode_negative_matrix_corpus() {
    // Regression: #3599
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let mut covered_cases: Vec<&'static str> = Vec::new();
    let expected_cases = vec![
        "strict-missing-signer-profile",
        "strict-missing-signer-key-source",
        "daemon-shutdown-controls-missing-signal",
        "strict-selector-env-mismatch-preflight",
        "fallback-secret-preflight",
    ];

    {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "kolme-live".to_owned(),
            "--kolme-live-base-url".to_owned(),
            "http://127.0.0.1:3000".to_owned(),
            "--kolme-live-provider-hint".to_owned(),
            "kolme-fork-local".to_owned(),
            "--kolme-live-signing-profile".to_owned(),
            "kolme-fork-secp256k1-v1".to_owned(),
            "--kolme-live-strict-signer-contracts".to_owned(),
            "--kolme-live-signer-key-source".to_owned(),
            "env-local".to_owned(),
        ];
        assert!(
            matches!(
                parse_args(args),
                Err(ConfigError::MissingArgumentValue(
                    "--kolme-live-signer-profile"
                ))
            ),
            "matrix case strict-missing-signer-profile must fail closed"
        );
        covered_cases.push("strict-missing-signer-profile");
    }

    {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "kolme-live".to_owned(),
            "--kolme-live-base-url".to_owned(),
            "http://127.0.0.1:3000".to_owned(),
            "--kolme-live-provider-hint".to_owned(),
            "kolme-fork-local".to_owned(),
            "--kolme-live-signing-profile".to_owned(),
            "kolme-fork-secp256k1-v1".to_owned(),
            "--kolme-live-strict-signer-contracts".to_owned(),
            "--kolme-live-signer-profile".to_owned(),
            "ops-primary".to_owned(),
        ];
        assert!(
            matches!(
                parse_args(args),
                Err(ConfigError::MissingArgumentValue(
                    "--kolme-live-signer-key-source"
                ))
            ),
            "matrix case strict-missing-signer-key-source must fail closed"
        );
        covered_cases.push("strict-missing-signer-key-source");
    }

    {
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "daemon".to_owned(),
            "--daemon-max-ticks".to_owned(),
            "12".to_owned(),
            "--daemon-tick-interval-ms".to_owned(),
            "5".to_owned(),
            "--daemon-shutdown-drain-ticks".to_owned(),
            "2".to_owned(),
            "--daemon-shutdown-timeout-ticks".to_owned(),
            "4".to_owned(),
        ];
        assert!(
            matches!(
                parse_args(args),
                Err(ConfigError::MissingArgumentValue(
                    "--daemon-shutdown-signal-tick"
                ))
            ),
            "matrix case daemon-shutdown-controls-missing-signal must fail closed"
        );
        covered_cases.push("daemon-shutdown-controls-missing-signal");
    }

    {
        let _profile_env_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary"));
        let _primary_key_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
            Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
        );
        let _fallback_key_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
        let _override_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING", None);
        let (base_url, requests) = spawn_kolme_live_mock_server(vec![
            MockHttpReply::ok(r#"{"next_nonce":17,"account_id":"acct-live-processor"}"#),
            MockHttpReply::ok(
                r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"pending"}"#,
            ),
        ]);
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "kolme-live".to_owned(),
            "--kolme-live-base-url".to_owned(),
            base_url,
            "--kolme-live-provider-hint".to_owned(),
            "kolme-fork-local".to_owned(),
            "--kolme-live-signing-profile".to_owned(),
            "kolme-fork-secp256k1-v1".to_owned(),
            "--kolme-live-strict-signer-contracts".to_owned(),
            "--kolme-live-signer-profile".to_owned(),
            "ops-primary".to_owned(),
            "--kolme-live-signer-key-source".to_owned(),
            "env-local".to_owned(),
        ];
        let parsed = parse_args(args).expect("strict mismatch args should parse");
        assert!(
            matches!(
                execute(parsed),
                Err(ConfigError::RuntimeKolmeLive(message))
                if message.contains("strict signer profile mismatch")
            ),
            "matrix case strict-selector-env-mismatch-preflight must fail closed before network"
        );
        let recorded_requests = requests.lock().expect("request mutex should lock");
        assert_eq!(
            recorded_requests.len(),
            0,
            "strict selector/env mismatch must fail before network"
        );
        covered_cases.push("strict-selector-env-mismatch-preflight");
    }

    {
        let _profile_env_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
        let _primary_key_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
            Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
        );
        let _fallback_key_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK",
            Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY),
        );
        let (base_url, requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
            r#"{"next_nonce":17,"account_id":"acct-live-processor"}"#,
        )]);
        let args = vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "kolme-live".to_owned(),
            "--kolme-live-base-url".to_owned(),
            base_url,
            "--kolme-live-provider-hint".to_owned(),
            "kolme-fork-local".to_owned(),
            "--kolme-live-signing-profile".to_owned(),
            "kolme-fork-secp256k1-v1".to_owned(),
            "--kolme-live-strict-signer-contracts".to_owned(),
            "--kolme-live-signer-profile".to_owned(),
            "ops-primary".to_owned(),
            "--kolme-live-signer-key-source".to_owned(),
            "env-local".to_owned(),
        ];
        let parsed = parse_args(args).expect("fallback-secret args should parse");
        assert!(
            matches!(
                execute(parsed),
                Err(ConfigError::RuntimeKolmeLive(message))
                if message.contains("fallback_signer_secret_present_violation")
            ),
            "matrix case fallback-secret-preflight must fail closed with deterministic reason code"
        );
        let recorded_requests = requests.lock().expect("request mutex should lock");
        assert_eq!(
            recorded_requests.len(),
            0,
            "fallback secret path must fail before network dispatch"
        );
        covered_cases.push("fallback-secret-preflight");
    }

    assert_eq!(
        covered_cases, expected_cases,
        "startup_negative_matrix_policy_marker_missing"
    );
}

#[test]
fn rejects_kolme_live_with_invalid_signing_profile() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "synthetic-signing-profile".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidKolmeLiveSigningProfile(
            "synthetic-signing-profile".to_owned()
        ))
    );
}

#[test]
fn rejects_kolme_live_with_in_memory_provider_hint_marker() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "InMemoryKolmeRuntimeCommitClient".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidKolmeLiveProviderHint(
            "InMemoryKolmeRuntimeCommitClient".to_owned()
        ))
    );
}

#[test]
fn rejects_unknown_argument() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "approver".to_owned(),
        "--unknown".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::UnknownArgument("--unknown".to_owned()))
    );
}

#[test]
fn rejects_invalid_output_mode() {
    // Regression: #307
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "approver".to_owned(),
        "--output".to_owned(),
        "yaml".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidOutputMode("yaml".to_owned()))
    );
}

#[test]
fn rejects_invalid_profile_value() {
    // Regression: #310
    let args = vec![
        "kamn-node".to_owned(),
        "--profile".to_owned(),
        "local-unknown".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidNodeProfile("local-unknown".to_owned()))
    );
}

#[test]
fn rejects_invalid_runtime_mode() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "service".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidRuntimeMode("service".to_owned()))
    );
}

#[test]
fn rejects_malformed_proposal_argument() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "planning".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-1".to_owned(),
        "--proposal".to_owned(),
        "tx-1|did:kamn:agent:aaa|state-1".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidProposalArgument(
            "tx-1|did:kamn:agent:aaa|state-1".to_owned()
        ))
    );
}

#[test]
fn rejects_malformed_rejoin_attempt_argument() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "recovery-check".to_owned(),
        "--expected-state-version".to_owned(),
        "42".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-42".to_owned(),
        "--rejoin-attempt".to_owned(),
        "node-a|42|state-42".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidRejoinAttemptArgument(
            "node-a|42|state-42".to_owned()
        ))
    );
}

#[test]
fn rejects_invalid_expected_state_version_argument() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "recovery-check".to_owned(),
        "--expected-state-version".to_owned(),
        "0".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-42".to_owned(),
        "--rejoin-attempt".to_owned(),
        "node-a|42|state-42|resume-1".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidExpectedStateVersion("0".to_owned()))
    );
}

#[test]
fn rejects_invalid_daemon_control_argument() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "abc".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidDaemonControlArgument("abc".to_owned()))
    );
}

#[test]
fn rejects_invalid_daemon_lifecycle_event_argument() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-peer-id".to_owned(),
        "peer-alpha".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "resume".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidDaemonLifecycleEvent(
            "resume".to_owned()
        ))
    );
}

#[test]
fn rejects_invalid_diagnostics_mode() {
    // Regression: #313
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--diagnostics".to_owned(),
        "extended".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidDiagnosticsMode("extended".to_owned()))
    );
}

#[test]
fn regression_runtime_planning_rejects_duplicate_candidate_ids() {
    // Regression: #335
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "planning".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-1".to_owned(),
        "--proposal".to_owned(),
        "tx-1|did:kamn:agent:aaa|1|state-1".to_owned(),
        "--proposal".to_owned(),
        "tx-1|did:kamn:agent:bbb|2|state-1".to_owned(),
    ];
    let parsed = parse_args(args).expect("planning args should parse");
    assert_eq!(
        execute(parsed),
        Err(ConfigError::RuntimePlanner(
            "duplicate proposal candidate id: tx-1".to_owned()
        ))
    );
}

#[test]
fn regression_runtime_planning_rejects_stale_state_hash() {
    // Regression: #335
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "planning".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-1".to_owned(),
        "--proposal".to_owned(),
        "tx-1|did:kamn:agent:aaa|1|state-2".to_owned(),
    ];
    let parsed = parse_args(args).expect("planning args should parse");
    assert_eq!(
        execute(parsed),
        Err(ConfigError::RuntimePlanner(
            "proposal candidate state hash mismatch: expected state-1, found state-2".to_owned()
        ))
    );
}

#[test]
fn regression_runtime_recovery_rejects_replay_resume_token() {
    // Regression: #336
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "recovery-check".to_owned(),
        "--expected-state-version".to_owned(),
        "42".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-42".to_owned(),
        "--rejoin-attempt".to_owned(),
        "node-a|42|state-42|resume-1".to_owned(),
        "--rejoin-attempt".to_owned(),
        "node-a|42|state-42|resume-1".to_owned(),
    ];
    let parsed = parse_args(args).expect("recovery-check args should parse");
    assert_eq!(
        execute(parsed),
        Err(ConfigError::RuntimeRecovery(
            "rejoin resume token replayed: resume-1".to_owned()
        ))
    );
}

#[test]
fn regression_runtime_recovery_rejects_state_version_mismatch() {
    // Regression: #336
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "recovery-check".to_owned(),
        "--expected-state-version".to_owned(),
        "42".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-42".to_owned(),
        "--rejoin-attempt".to_owned(),
        "node-a|43|state-43|resume-1".to_owned(),
    ];
    let parsed = parse_args(args).expect("recovery-check args should parse");
    assert_eq!(
        execute(parsed),
        Err(ConfigError::RuntimeRecovery(
            "rejoin state version mismatch: expected 42, found 43".to_owned()
        ))
    );
}

#[test]
fn regression_runtime_recovery_rejects_state_hash_mismatch() {
    // Regression: #336
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "recovery-check".to_owned(),
        "--expected-state-version".to_owned(),
        "42".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-42".to_owned(),
        "--rejoin-attempt".to_owned(),
        "node-a|42|state-41|resume-1".to_owned(),
    ];
    let parsed = parse_args(args).expect("recovery-check args should parse");
    assert_eq!(
        execute(parsed),
        Err(ConfigError::RuntimeRecovery(
            "rejoin state hash mismatch: expected state-42, found state-41".to_owned()
        ))
    );
}

#[test]
fn regression_runtime_daemon_rejects_zero_tick_budget() {
    // Regression: #348
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "0".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidDaemonControlArgument("0".to_owned()))
    );
}

#[test]
fn regression_runtime_observability_endpoint_rejects_custom_path_without_bind_address() {
    // Regression: #2830
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--observability-endpoint-metrics-path".to_owned(),
        "/runtime/metrics".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue(
            "--observability-endpoint-bind"
        ))
    );
}

#[test]
fn regression_runtime_daemon_rejects_invalid_lifecycle_transition() {
    // Regression: #349
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-peer-id".to_owned(),
        "peer-alpha".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "handshake-succeeded".to_owned(),
    ];
    let parsed = parse_args(args).expect("daemon args should parse");
    assert_eq!(
        execute(parsed),
        Err(ConfigError::RuntimeDaemonLifecycle(
            "invalid peer lifecycle transition from Disconnected via HandshakeSucceeded".to_owned()
        ))
    );
}

#[test]
fn regression_runtime_daemon_ignores_replayed_and_late_shutdown_signals() {
    // Regression: #2674
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "8".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "3".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "7".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "11".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "2".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "4".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];
    let parsed = parse_args(args).expect("daemon replay args should parse");
    let report = execute(parsed).expect("daemon replay execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"daemon_executed_ticks\":5"));
    assert!(rendered.contains(
        "\"daemon_completion_reason\":\"graceful-shutdown:signal@3;drain_ticks=2;timeout_ticks=4;ignored_signals=2\""
    ));
}

#[test]
fn performance_runtime_daemon_shutdown_drain_stays_bounded_by_timeout_budget() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "9".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "8".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "5".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "1".to_owned(),
    ];
    let parsed = parse_args(args).expect("daemon bounded shutdown args should parse");
    let report = execute(parsed).expect("daemon bounded shutdown should succeed");
    let executed_ticks = report
        .daemon_executed_ticks
        .expect("daemon execution must report executed ticks");
    assert!(
        executed_ticks <= 9,
        "shutdown drain execution must remain within max tick budget"
    );
    assert_eq!(
        report.daemon_completion_reason.as_deref(),
        Some("graceful-shutdown-timeout:signal@8;drain_ticks=5;timeout_ticks=1;ignored_signals=0")
    );
}
