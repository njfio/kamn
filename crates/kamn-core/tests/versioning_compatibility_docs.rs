const POLICY: &str = include_str!("../../../docs/foundation/versioning-compatibility-matrix.md");

#[test]
fn policy_defines_semantic_versions_for_chain_app_and_sdks() {
    assert!(POLICY.contains("## Semantic Versioning Policy"));
    assert!(POLICY.contains("Chain protocol version follows MAJOR.MINOR.PATCH."));
    assert!(POLICY.contains("App-state schema version follows MAJOR.MINOR.PATCH."));
    assert!(POLICY.contains("SDK versions (Rust, Python, TypeScript) follow MAJOR.MINOR.PATCH."));
}

#[test]
fn policy_contains_compatibility_matrix_with_upgrade_and_downgrade_expectations() {
    assert!(POLICY.contains("## Compatibility Matrix"));
    assert!(POLICY.contains("| Chain Protocol | App-State Schema | Node Binary | SDK Family | Upgrade Expectation | Downgrade Expectation |"));
    assert!(POLICY.contains("Same major version upgrade: supported with migration plan."));
    assert!(
        POLICY.contains("Cross-major upgrade: requires governance approval and staged rollout.")
    );
    assert!(POLICY.contains("Downgrade across major versions: blocked."));
}

#[test]
fn policy_defines_support_and_deprecation_windows() {
    assert!(POLICY.contains("## Support and Deprecation Windows"));
    assert!(POLICY.contains("Current minor (N) and previous minor (N-1) are supported."));
    assert!(POLICY.contains("Anything older than N-1 is deprecated and no-go for new rollouts."));
}

#[test]
fn policy_defines_governance_parameter_compatibility_contract() {
    assert!(POLICY.contains("## Governance Parameter Compatibility Policy"));
    assert!(POLICY.contains("| Parameter Key | Allowed Range | Minimum Supported Version |"));
    assert!(POLICY.contains("| `listener.quorum` | `[1, 7]` | `1.0.0` |"));
    assert!(POLICY.contains("| `watchdog.delivery_ratio_bps` | `[9000, 9999]` | `1.1.0` |"));
}

#[test]
fn regression_requires_incompatible_downgrade_no_go_rule() {
    // Regression: #175
    assert!(POLICY.contains("Incompatible downgrade decision: NO-GO."));
    assert!(POLICY.contains(
        "Referenced by governance workflow: docs/foundation/release-gonogo-checklist.md"
    ));
    assert!(POLICY.contains(
        "Referenced by migration/rollback workflow: docs/foundation/upgrade-rollback-runbook.md"
    ));
}
