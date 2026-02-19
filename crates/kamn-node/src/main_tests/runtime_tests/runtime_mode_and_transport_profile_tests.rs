#[test]
fn parses_runtime_mode_planning_with_proposals() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "planning".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-1".to_owned(),
        "--proposal".to_owned(),
        "tx-2|did:kamn:agent:bbb|2|state-1".to_owned(),
        "--proposal".to_owned(),
        "tx-1|did:kamn:agent:aaa|1|state-1".to_owned(),
    ];

    let parsed = parse_args(args).expect("planning args should parse");
    assert_eq!(parsed.runtime_mode, RuntimeMode::planning());
    assert_eq!(parsed.expected_state_hash, Some("state-1".to_owned()));
    assert_eq!(parsed.proposals.len(), 2);
}

#[test]
fn parses_runtime_mode_recovery_check_with_rejoin_attempt() {
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
    ];

    let parsed = parse_args(args).expect("recovery-check args should parse");
    assert_eq!(parsed.runtime_mode, RuntimeMode::recovery_check());
    assert_eq!(parsed.expected_state_version, Some(42));
    assert_eq!(parsed.expected_state_hash, Some("state-42".to_owned()));
    assert_eq!(parsed.rejoin_attempts.len(), 1);
}

#[test]
fn parses_runtime_mode_full_with_required_controls() {
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
        "--api-bind".to_owned(),
        "127.0.0.1:19081".to_owned(),
    ];

    let parsed = parse_args(args).expect("full args should parse");
    assert_eq!(parsed.runtime_mode.as_str(), "full");
    assert_eq!(parsed.daemon_max_ticks, Some(3));
    assert_eq!(parsed.daemon_tick_interval_ms, Some(25));
    assert_eq!(parsed.api_bind_addr, Some("127.0.0.1:19081".to_owned()));
}

#[test]
fn unit_select_runtime_transport_profile_for_production_modes_defaults_live() {
    assert_eq!(
        select_runtime_transport_profile_for_runtime_mode(RuntimeMode::full(), true),
        Some(kamn_core::RuntimeTransportProfile::Libp2pLive)
    );
    assert_eq!(
        select_runtime_transport_profile_for_runtime_mode(RuntimeMode::daemon(), true),
        Some(kamn_core::RuntimeTransportProfile::Libp2pLive)
    );
    assert_eq!(
        select_runtime_transport_profile_for_runtime_mode(RuntimeMode::api(), true),
        Some(kamn_core::RuntimeTransportProfile::Libp2pLive)
    );
    assert_eq!(
        select_runtime_transport_profile_for_runtime_mode(RuntimeMode::kolme_live(), true),
        Some(kamn_core::RuntimeTransportProfile::Libp2pLive)
    );
    assert_eq!(
        select_runtime_transport_profile_for_runtime_mode(RuntimeMode::planning(), true),
        None
    );
}

#[test]
fn unit_shutdown_policy_defaults_to_os_signal_path_for_daemon_and_full_on_unix() {
    let expected_default = cfg!(unix);
    assert_eq!(
        crate::should_use_os_signal_shutdown(RuntimeMode::daemon(), false, &[]),
        expected_default
    );
    assert_eq!(
        crate::should_use_os_signal_shutdown(RuntimeMode::full(), false, &[]),
        expected_default
    );
    assert!(!crate::should_use_os_signal_shutdown(
        RuntimeMode::bootstrap(),
        false,
        &[]
    ));
}

#[test]
fn regression_shutdown_policy_prioritizes_explicit_controls_over_defaults() {
    // Regression: #3734
    assert!(crate::should_use_os_signal_shutdown(
        RuntimeMode::daemon(),
        true,
        &[]
    ));
    assert!(!crate::should_use_os_signal_shutdown(
        RuntimeMode::daemon(),
        false,
        &[3]
    ));
}

#[test]
fn functional_production_transport_profile_classifier_rejects_in_memory_fallback() {
    let components = vec![
        "p2p-discovery".to_owned(),
        "p2p-gossip-transport".to_owned(),
        "p2p-transport-profile:in-memory-deterministic".to_owned(),
        "p2p-in-memory-transport-fallback".to_owned(),
    ];
    assert_eq!(
        classify_production_transport_profile_violation(RuntimeMode::full(), true, &components),
        Some("runtime_transport_profile_in_memory_fallback_forbidden")
    );
}

#[test]
fn functional_transport_profile_classifier_rejects_live_and_fallback_profile_pair_conflict() {
    let components = vec![
        "p2p-discovery".to_owned(),
        "p2p-gossip-transport".to_owned(),
        "p2p-transport-profile:libp2p-live".to_owned(),
        "p2p-transport-profile:in-memory-deterministic".to_owned(),
        "p2p-in-memory-transport-fallback".to_owned(),
    ];
    assert_eq!(
        classify_production_transport_profile_violation(RuntimeMode::full(), true, &components),
        Some("runtime_transport_profile_pair_disallowed")
    );
}

#[test]
fn functional_transport_profile_classifier_rejects_fallback_marker_without_profile_pair() {
    let components = vec![
        "p2p-discovery".to_owned(),
        "p2p-gossip-transport".to_owned(),
        "p2p-in-memory-transport-fallback".to_owned(),
    ];
    assert_eq!(
        classify_production_transport_profile_violation(RuntimeMode::planning(), true, &components),
        Some("runtime_transport_profile_fallback_marker_without_in_memory_profile")
    );
}

#[test]
fn functional_transport_profile_classifier_accepts_planning_in_memory_profile_pair() {
    let components = vec![
        "p2p-discovery".to_owned(),
        "p2p-gossip-transport".to_owned(),
        "p2p-transport-profile:in-memory-deterministic".to_owned(),
        "p2p-in-memory-transport-fallback".to_owned(),
    ];
    assert_eq!(
        classify_production_transport_profile_violation(RuntimeMode::planning(), true, &components),
        None
    );
}

#[test]
fn regression_transport_profile_pair_disallowed_reason_code_is_stable() {
    // Regression: #3880
    let components = vec![
        "p2p-transport-profile:libp2p-live".to_owned(),
        "p2p-transport-profile:in-memory-deterministic".to_owned(),
        "p2p-in-memory-transport-fallback".to_owned(),
    ];
    assert_eq!(
        classify_production_transport_profile_violation(RuntimeMode::daemon(), true, &components),
        Some("runtime_transport_profile_pair_disallowed")
    );
}

#[test]
fn regression_transport_profile_fallback_marker_linkage_reason_code_is_stable() {
    // Regression: #3880
    let components = vec!["p2p-in-memory-transport-fallback".to_owned()];
    assert_eq!(
        classify_production_transport_profile_violation(RuntimeMode::planning(), true, &components),
        Some("runtime_transport_profile_fallback_marker_without_in_memory_profile")
    );
}

#[test]
fn functional_production_transport_profile_classifier_rejects_contract_only_compile_mode() {
    let components = vec![
        "p2p-discovery".to_owned(),
        "p2p-gossip-transport".to_owned(),
        "p2p-transport-profile:libp2p-live".to_owned(),
        "p2p-live-libp2p-provider".to_owned(),
        "p2p-live-libp2p-provider:contract-only".to_owned(),
    ];
    assert_eq!(
        classify_production_transport_profile_violation(RuntimeMode::full(), true, &components),
        Some("runtime_transport_profile_compile_mode_not_native")
    );
}

#[test]
fn regression_production_transport_policy_error_detail_includes_remediation_guidance() {
    // Regression: #3673
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "full".to_owned(),
        "--disable-gossip".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "5".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:19093".to_owned(),
    ])
    .expect("full args should parse");

    let error = execute(parsed).expect_err("production transport policy must fail closed");
    match error {
        ConfigError::RuntimeStoreCompatibility {
            reason_code,
            detail,
            ..
        } => {
            assert_eq!(
                reason_code,
                "runtime_transport_profile_gossip_disabled_for_production"
            );
            assert!(
                detail.contains("remove --disable-gossip"),
                "detail should include actionable gossip remediation guidance"
            );
            assert!(
                detail.contains("or use non-production runtime modes"),
                "detail should include non-production fallback guidance"
            );
        }
        other => panic!("expected runtime store compatibility failure, found {other:?}"),
    }
}

#[test]
fn integration_runtime_full_uses_live_transport_profile_components_by_default() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "full".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "5".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:19091".to_owned(),
    ])
    .expect("full args should parse");

    let report = execute(parsed).expect("full execution should succeed");
    assert!(report
        .components
        .contains(&"p2p-transport-profile:libp2p-live".to_owned()));
    assert!(report
        .components
        .contains(&"p2p-live-libp2p-provider".to_owned()));
    assert!(report
        .components
        .contains(&"p2p-live-libp2p-provider:native".to_owned()));
    assert!(!report
        .components
        .contains(&"p2p-in-memory-transport-fallback".to_owned()));
}

#[test]
fn regression_production_transport_profile_in_memory_rejection_reason_code_is_stable() {
    // Regression: #3634
    let components = vec![
        "p2p-discovery".to_owned(),
        "p2p-gossip-transport".to_owned(),
        "p2p-in-memory-transport-fallback".to_owned(),
    ];
    assert_eq!(
        classify_production_transport_profile_violation(RuntimeMode::daemon(), true, &components),
        Some("runtime_transport_profile_fallback_marker_without_in_memory_profile")
    );
}

#[test]
fn performance_runtime_full_live_transport_profile_startup_stays_within_local_budget() {
    let started = std::time::Instant::now();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "full".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "5".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:19092".to_owned(),
    ])
    .expect("full args should parse");

    let report = execute(parsed).expect("full execution should succeed");
    assert_eq!(report.runtime_mode, "full");
    assert!(
        started.elapsed() <= std::time::Duration::from_secs(2),
        "full runtime live transport profile startup exceeded local budget"
    );
}

