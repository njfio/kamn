#[test]
fn parses_required_role_and_defaults() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
    ];

    let parsed = parse_args(args).expect("args should parse");
    assert_eq!(parsed.profile, None);
    assert_eq!(parsed.role, NodeRole::Processor);
    assert_eq!(parsed.chain_id, "kamn-devnet");
    assert_eq!(parsed.chain_version, "v0.1.0");
    assert_eq!(parsed.storage_dir, "./data");
    assert!(parsed.enable_gossip);
    assert_eq!(parsed.sync_mode, SyncMode::Fast);
    assert_eq!(parsed.runtime_mode, RuntimeMode::bootstrap());
    assert_eq!(parsed.expected_state_hash, None);
    assert_eq!(parsed.expected_state_version, None);
    assert!(parsed.proposals.is_empty());
    assert!(parsed.rejoin_attempts.is_empty());
    assert_eq!(parsed.daemon_max_ticks, None);
    assert_eq!(parsed.daemon_tick_interval_ms, None);
    assert!(parsed.daemon_shutdown_signal_ticks.is_empty());
    assert!(!parsed.daemon_shutdown_os_signals);
    assert_eq!(parsed.daemon_shutdown_drain_ticks, None);
    assert_eq!(parsed.daemon_shutdown_timeout_ticks, None);
    assert_eq!(parsed.daemon_peer_id, None);
    assert!(parsed.daemon_lifecycle_events.is_empty());
    assert_eq!(parsed.kolme_live_base_url, None);
    assert_eq!(parsed.kolme_live_provider_hint, None);
    assert_eq!(parsed.kolme_live_signing_profile, None);
    assert_eq!(parsed.observability_endpoint_bind_addr, None);
    assert_eq!(parsed.observability_endpoint_metrics_path, "/metrics");
    assert_eq!(parsed.observability_endpoint_health_path, "/healthz");
    assert_eq!(parsed.observability_endpoint_max_requests, 1);
    assert_eq!(parsed.observability_endpoint_idle_timeout_ms, 5_000);
    assert_eq!(parsed.output_mode, OutputMode::text());
    assert_eq!(parsed.diagnostics_mode, DiagnosticsMode::basic());
}

#[test]
fn parses_disable_gossip_flag() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "listener".to_owned(),
        "--disable-gossip".to_owned(),
    ];

    let parsed = parse_args(args).expect("args should parse");
    assert_eq!(parsed.role, NodeRole::Listener);
    assert!(!parsed.enable_gossip);
    assert_eq!(parsed.sync_mode, SyncMode::Fast);
}

#[test]
fn parses_sync_mode_flag() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--sync-mode".to_owned(),
        "archive".to_owned(),
    ];

    let parsed = parse_args(args).expect("args should parse");
    assert_eq!(parsed.sync_mode, SyncMode::Archive);
}

#[test]
fn parses_output_mode_json_flag() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed = parse_args(args).expect("args should parse");
    assert_eq!(parsed.output_mode, OutputMode::json());
}

#[test]
fn parses_diagnostics_snapshot_flag() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--diagnostics".to_owned(),
        "snapshot".to_owned(),
    ];

    let parsed = parse_args(args).expect("diagnostics args should parse");
    assert_eq!(parsed.diagnostics_mode, DiagnosticsMode::snapshot());
}

#[test]
fn unit_kolme_live_local_signer_override_marker_defaults_false() {
    let _signer_lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _override_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING", None);
    assert!(!resolve_kolme_live_allow_local_signer_testing_override()
        .expect("override marker should default false when unset"));
}

#[test]
fn unit_kolme_live_local_signer_override_marker_parses_boolean_values() {
    let _signer_lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");

    {
        let _true_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING", Some("true"));
        assert!(resolve_kolme_live_allow_local_signer_testing_override()
            .expect("true override marker should parse"));
    }
    {
        let _false_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING", Some("false"));
        assert!(!resolve_kolme_live_allow_local_signer_testing_override()
            .expect("false override marker should parse"));
    }
}

#[test]
fn regression_kolme_live_local_signer_override_marker_rejects_invalid_value() {
    let _signer_lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _override_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING", Some("maybe"));
    let error = resolve_kolme_live_allow_local_signer_testing_override()
        .expect_err("invalid override marker value must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("legacy_local_signer_path_override_invalid")),
        "invalid local signer override marker must fail with deterministic reason code"
    );
}

#[test]
fn unit_kolme_live_signer_contract_policy_allows_strict_override_or_test_build() {
    enforce_kolme_live_signer_contract_policy(true, false, false)
        .expect("strict signer contracts should satisfy policy");
    enforce_kolme_live_signer_contract_policy(false, true, false)
        .expect("explicit local testing override should satisfy policy");
    enforce_kolme_live_signer_contract_policy(false, false, true)
        .expect("test build should bypass strict signer policy guard");
}

#[test]
fn regression_kolme_live_signer_contract_policy_rejects_legacy_local_path_in_production() {
    let error = enforce_kolme_live_signer_contract_policy(false, false, false)
        .expect_err("production runtime must reject legacy local signer path");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("legacy_local_signer_path_forbidden")),
        "production policy enforcement must emit deterministic legacy local signer rejection marker"
    );
}

#[test]
fn unit_kolme_live_signer_key_source_policy_classifier_matrix() {
    assert_eq!(
        classify_kolme_live_signer_key_source_policy_violation(
            false,
            Some("env-local"),
            false,
            false,
        ),
        None
    );
    assert_eq!(
        classify_kolme_live_signer_key_source_policy_violation(
            true,
            Some("managed-external"),
            false,
            false,
        ),
        None
    );
    assert_eq!(
        classify_kolme_live_signer_key_source_policy_violation(
            true,
            Some("env-local"),
            false,
            false,
        ),
        Some("production_signer_key_source_env_local_forbidden")
    );
    assert_eq!(
        classify_kolme_live_signer_key_source_policy_violation(
            true,
            Some("env-local"),
            true,
            false,
        ),
        None
    );
    assert_eq!(
        classify_kolme_live_signer_key_source_policy_violation(
            true,
            Some("env-local"),
            false,
            true,
        ),
        None
    );
}

#[test]
pub(super) fn functional_kolme_live_strict_env_local_key_source_rejects_with_reason_code() {
    let parsed = parse_args(vec![
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
    ])
    .expect("strict args should parse");
    let error = enforce_kolme_live_signer_key_source_policy(
        parsed.kolme_live_strict_signer_contracts,
        parsed.kolme_live_signer_key_source.as_deref(),
        false,
        false,
    )
    .expect_err("strict env-local key source must fail closed for production-targeted runs");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("production_signer_key_source_env_local_forbidden")),
        "strict env-local key source policy must emit deterministic reason code"
    );
}

#[test]
pub(super) fn functional_kolme_live_strict_env_local_key_source_allows_with_local_override() {
    let parsed = parse_args(vec![
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
    ])
    .expect("strict args should parse");
    enforce_kolme_live_signer_key_source_policy(
        parsed.kolme_live_strict_signer_contracts,
        parsed.kolme_live_signer_key_source.as_deref(),
        true,
        false,
    )
    .expect("explicit local override should allow strict env-local key source");
}

#[test]
pub(super) fn integration_kolme_live_strict_managed_external_key_source_policy_passes() {
    let parsed = parse_args(vec![
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
    ])
    .expect("strict managed-external args should parse");
    enforce_kolme_live_signer_key_source_policy(
        parsed.kolme_live_strict_signer_contracts,
        parsed.kolme_live_signer_key_source.as_deref(),
        false,
        false,
    )
    .expect("strict managed-external key source should satisfy production policy");
}

#[test]
fn regression_runtime_kolme_live_honors_declared_managed_external_key_source_without_strict_flag() {
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let _fallback_key_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
    let _key_ref_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE),
    );
    let _signer_public_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
        Some(managed_signer_public_key_hex(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE).as_str()),
    );
    let _managed_command_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND", None);

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:39000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-signer-profile".to_owned(),
        "ops-primary".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "managed-external".to_owned(),
    ])
    .expect("kolme-live args should parse");
    let error = execute(parsed).expect_err(
        "declared managed-external key source must be honored without strict flag in local/test execution",
    );
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_backend_required_missing")),
        "declared managed-external key source must not silently fall back to env-local signer path"
    );
}
