use super::super::*;

pub(super) struct PreflightMatrixCase {
    pub(super) name: &'static str,
    pub(super) strict_profile: &'static str,
    pub(super) previous_profile: &'static str,
    pub(super) rotation_epoch: &'static str,
    pub(super) previous_rotation_epoch: &'static str,
    pub(super) required_approvals: &'static str,
    pub(super) approved_signers: &'static str,
    pub(super) expected_reason: Option<&'static str>,
}

pub(super) const PREFLIGHT_MATRIX: &[PreflightMatrixCase] = &[
    PreflightMatrixCase {
        name: "linked_non_failover_primary",
        strict_profile: "ops-primary",
        previous_profile: "ops-primary",
        rotation_epoch: "2",
        previous_rotation_epoch: "1",
        required_approvals: "1",
        approved_signers: "ops-primary",
        expected_reason: None,
    },
    PreflightMatrixCase {
        name: "profile_not_approved_non_failover",
        strict_profile: "ops-secondary",
        previous_profile: "ops-secondary",
        rotation_epoch: "2",
        previous_rotation_epoch: "1",
        required_approvals: "1",
        approved_signers: "ops-primary",
        expected_reason: Some("runtime_signer_quorum_linkage_violation"),
    },
    PreflightMatrixCase {
        name: "quorum_shortfall_non_failover",
        strict_profile: "ops-primary",
        previous_profile: "ops-primary",
        rotation_epoch: "2",
        previous_rotation_epoch: "1",
        required_approvals: "2",
        approved_signers: "ops-primary",
        expected_reason: Some("runtime_signer_attestation_quorum_shortfall"),
    },
    PreflightMatrixCase {
        name: "failover_previous_profile_not_approved",
        strict_profile: "ops-secondary",
        previous_profile: "ops-primary",
        rotation_epoch: "2",
        previous_rotation_epoch: "1",
        required_approvals: "2",
        approved_signers: "ops-secondary",
        expected_reason: Some("runtime_signer_failover_attestation_previous_profile_not_approved"),
    },
    PreflightMatrixCase {
        name: "linked_failover_dual_approved",
        strict_profile: "ops-secondary",
        previous_profile: "ops-primary",
        rotation_epoch: "2",
        previous_rotation_epoch: "1",
        required_approvals: "2",
        approved_signers: "ops-primary,ops-secondary",
        expected_reason: None,
    },
];

pub(super) fn profile_key_env_guards(profile: &'static str) -> Vec<EnvVarGuard> {
    vec![
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some(profile)),
        EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
            (profile == "ops-primary").then_some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
        ),
        EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
            (profile == "ops-secondary")
                .then_some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY),
        ),
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None),
    ]
}

pub(super) fn quorum_env_guards(
    previous_profile: &'static str,
    rotation_epoch: &'static str,
    previous_rotation_epoch: &'static str,
    required_approvals: &'static str,
    approved_signers: &'static str,
) -> Vec<EnvVarGuard> {
    let mut guards = vec![
        guard("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE", previous_profile),
        guard("KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH", rotation_epoch),
        guard(
            "KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH",
            previous_rotation_epoch,
        ),
    ];
    guards.extend(quorum_requirement_guards(
        required_approvals,
        approved_signers,
    ));
    guards
}

pub(super) fn assert_preflight_reason(
    profile: &'static str,
    key_source: &'static str,
    reason_code: &'static str,
    failure_context: &'static str,
) {
    let error = enforce_kolme_live_signer_preflight(Some(profile), Some(key_source))
        .expect_err(failure_context);
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains(reason_code)),
        "{failure_context}"
    );
}

pub(super) fn assert_preflight_ready(
    profile: &'static str,
    key_source: &'static str,
    success_context: &'static str,
) {
    let readiness = enforce_kolme_live_signer_preflight(Some(profile), Some(key_source))
        .expect(success_context);
    assert!(readiness.quorum_linked, "{success_context}");
}

pub(super) fn assert_preflight_matrix_case(case: &PreflightMatrixCase) {
    let _ = case.name;
    let _profile_guards = profile_key_env_guards(case.strict_profile);
    let _quorum_guards = quorum_env_guards(
        case.previous_profile,
        case.rotation_epoch,
        case.previous_rotation_epoch,
        case.required_approvals,
        case.approved_signers,
    );
    match case.expected_reason {
        Some(reason_code) => assert_preflight_reason(
            case.strict_profile,
            "env-local",
            reason_code,
            "matrix fail case must fail closed",
        ),
        None => assert_preflight_ready(
            case.strict_profile,
            "env-local",
            "matrix success case must remain ready",
        ),
    }
}

fn guard(name: &'static str, value: &'static str) -> EnvVarGuard {
    EnvVarGuard::set(name, Some(value))
}

fn quorum_requirement_guards(
    required_approvals: &'static str,
    approved_signers: &'static str,
) -> [EnvVarGuard; 2] {
    [
        guard(
            "KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS",
            required_approvals,
        ),
        guard(
            "KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS",
            approved_signers,
        ),
    ]
}
