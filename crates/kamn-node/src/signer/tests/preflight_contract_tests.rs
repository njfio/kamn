use super::support::{lock_signer_env_guard, test_primary_selection, EnvVarGuard};
use super::{
    evaluate_kolme_live_signer_preflight_readiness, ConfigError, Duration, Instant,
    KolmeLiveSignerSelection,
};

#[test]
fn unit_signer_preflight_defaults_to_single_signer_quorum_ready() {
    let _lock = lock_signer_env_guard();
    let _previous_profile = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE", None);
    let _rotation_epoch = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH", None);
    let _previous_rotation_epoch =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH", None);
    let _required_approvals =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS", None);
    let _approved_signers =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS", None);

    let readiness = evaluate_kolme_live_signer_preflight_readiness(&test_primary_selection())
        .expect("default single-signer preflight should be ready");
    assert_eq!(readiness.previous_profile, "ops-primary");
    assert!(!readiness.failover_active);
    assert_eq!(readiness.rotation_epoch, 1);
    assert_eq!(readiness.previous_rotation_epoch, 1);
    assert_eq!(readiness.quorum_linkage_contract_version, "v1");
    assert_eq!(readiness.quorum_required_approvals, 1);
    assert_eq!(readiness.quorum_approved_signers_count, 1);
    assert!(readiness.quorum_profile_linked);
    assert!(readiness.quorum_satisfied);
    assert!(readiness.quorum_linked);
}

#[test]
fn regression_signer_preflight_rejects_stale_failover_rotation_epoch() {
    let _lock = lock_signer_env_guard();
    let _previous_profile = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE",
        Some("ops-primary"),
    );
    let _rotation_epoch = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH", Some("2"));
    let _previous_rotation_epoch =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH", Some("2"));
    let _required_approvals = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS",
        Some("2"),
    );
    let _approved_signers = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS",
        Some("ops-primary,ops-secondary"),
    );
    let selection = KolmeLiveSignerSelection {
        profile: "ops-secondary",
        key_source: "env-local",
        private_key_env: "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
        key_reference_env: "KAMN_KOLME_LIVE_SIGNER_KEY_REF_SECONDARY",
    };
    let error = evaluate_kolme_live_signer_preflight_readiness(&selection)
        .expect_err("stale failover rotation epoch must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("runtime_signer_rotation_epoch_stale"))
    );
}

#[test]
fn regression_signer_preflight_rejects_non_failover_rotation_epoch_regression() {
    let _lock = lock_signer_env_guard();
    let _previous_profile = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE",
        Some("ops-primary"),
    );
    let _rotation_epoch = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH", Some("1"));
    let _previous_rotation_epoch =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH", Some("2"));
    let _required_approvals = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS",
        Some("1"),
    );
    let _approved_signers = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS",
        Some("ops-primary"),
    );
    let error = evaluate_kolme_live_signer_preflight_readiness(&test_primary_selection())
        .expect_err("non-failover rotation epoch regression must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("runtime_signer_rotation_epoch_regressed"))
    );
}

#[test]
fn regression_signer_preflight_rejects_disallowed_secondary_managed_external_pair() {
    let _lock = lock_signer_env_guard();
    let _previous_profile = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE",
        Some("ops-secondary"),
    );
    let _rotation_epoch = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH", Some("1"));
    let _previous_rotation_epoch =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH", Some("1"));
    let _required_approvals = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS",
        Some("1"),
    );
    let _approved_signers = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS",
        Some("ops-secondary"),
    );
    let selection = KolmeLiveSignerSelection {
        profile: "ops-secondary",
        key_source: "managed-external",
        private_key_env: "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
        key_reference_env: "KAMN_KOLME_LIVE_SIGNER_KEY_REF_SECONDARY",
    };
    let error = evaluate_kolme_live_signer_preflight_readiness(&selection)
        .expect_err("secondary managed-external pair must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("runtime_signer_key_source_profile_pair_disallowed"))
    );
}

#[test]
fn functional_signer_preflight_rejects_quorum_shortfall() {
    let _lock = lock_signer_env_guard();
    let _previous_profile = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE",
        Some("ops-primary"),
    );
    let _rotation_epoch = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH", Some("1"));
    let _previous_rotation_epoch =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH", Some("1"));
    let _required_approvals = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS",
        Some("2"),
    );
    let _approved_signers = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS",
        Some("ops-primary"),
    );
    let error = evaluate_kolme_live_signer_preflight_readiness(&test_primary_selection())
        .expect_err("quorum shortfall must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("runtime_signer_attestation_quorum_shortfall"))
    );
}

#[test]
fn performance_signer_preflight_readiness_stays_bounded() {
    let _lock = lock_signer_env_guard();
    let _previous_profile = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE",
        Some("ops-primary"),
    );
    let _rotation_epoch = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH", Some("1"));
    let _previous_rotation_epoch =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH", Some("1"));
    let _required_approvals = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS",
        Some("1"),
    );
    let _approved_signers = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS",
        Some("ops-primary"),
    );
    let selection = test_primary_selection();
    let started = Instant::now();
    for _ in 0..5_000 {
        let readiness = evaluate_kolme_live_signer_preflight_readiness(&selection)
            .expect("preflight readiness must remain stable");
        assert!(readiness.quorum_linked);
    }
    assert!(started.elapsed() < Duration::from_secs(2));
}
