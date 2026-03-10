use super::super::*;

#[test]
fn integration_kolme_live_signer_preflight_quorum_profile_matrix_paths() {
    // Regression: #3957
    let _lock = lock_signer_env_guard();

    struct Case {
        name: &'static str,
        strict_profile: &'static str,
        previous_profile: &'static str,
        rotation_epoch: &'static str,
        previous_rotation_epoch: &'static str,
        required_approvals: &'static str,
        approved_signers: &'static str,
        expected_reason: Option<&'static str>,
    }

    let matrix = [
        Case {
            name: "linked_non_failover_primary",
            strict_profile: "ops-primary",
            previous_profile: "ops-primary",
            rotation_epoch: "2",
            previous_rotation_epoch: "1",
            required_approvals: "1",
            approved_signers: "ops-primary",
            expected_reason: None,
        },
        Case {
            name: "profile_not_approved_non_failover",
            strict_profile: "ops-secondary",
            previous_profile: "ops-secondary",
            rotation_epoch: "2",
            previous_rotation_epoch: "1",
            required_approvals: "1",
            approved_signers: "ops-primary",
            expected_reason: Some("runtime_signer_quorum_linkage_violation"),
        },
        Case {
            name: "quorum_shortfall_non_failover",
            strict_profile: "ops-primary",
            previous_profile: "ops-primary",
            rotation_epoch: "2",
            previous_rotation_epoch: "1",
            required_approvals: "2",
            approved_signers: "ops-primary",
            expected_reason: Some("runtime_signer_attestation_quorum_shortfall"),
        },
        Case {
            name: "failover_previous_profile_not_approved",
            strict_profile: "ops-secondary",
            previous_profile: "ops-primary",
            rotation_epoch: "2",
            previous_rotation_epoch: "1",
            required_approvals: "2",
            approved_signers: "ops-secondary",
            expected_reason: Some(
                "runtime_signer_failover_attestation_previous_profile_not_approved",
            ),
        },
        Case {
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

    for case in matrix {
        let _profile_env_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some(case.strict_profile));
        let _primary_key_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
            if case.strict_profile == "ops-primary" {
                Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX)
            } else {
                None
            },
        );
        let _secondary_key_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
            if case.strict_profile == "ops-secondary" {
                Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY)
            } else {
                None
            },
        );
        let _fallback_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
        let _previous_profile_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE",
            Some(case.previous_profile),
        );
        let _rotation_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH",
            Some(case.rotation_epoch),
        );
        let _previous_rotation_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH",
            Some(case.previous_rotation_epoch),
        );
        let _required_approvals_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS",
            Some(case.required_approvals),
        );
        let _approved_signers_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS",
            Some(case.approved_signers),
        );

        match case.expected_reason {
            Some(reason_code) => {
                let error = enforce_kolme_live_signer_preflight(
                    Some(case.strict_profile),
                    Some("env-local"),
                )
                .expect_err("matrix fail case must fail closed");
                assert!(
                    matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains(reason_code)),
                    "matrix case '{}' must preserve reason code {}",
                    case.name,
                    reason_code
                );
            }
            None => {
                let readiness = enforce_kolme_live_signer_preflight(
                    Some(case.strict_profile),
                    Some("env-local"),
                )
                .expect("matrix success case must remain ready");
                assert!(
                    readiness.quorum_linked,
                    "matrix case '{}' must preserve quorum_linked=true",
                    case.name
                );
            }
        }
    }
}
