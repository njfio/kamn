use super::super::*;

#[test]
fn functional_signer_migration_profile_key_source_parity_matrix() {
    // Regression: #3766
    let _lock = lock_signer_env_guard();

    {
        let _profile_env_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
        let _primary_key_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
            Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
        );
        let _secondary_key_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY", None);
        let _fallback_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
        let _previous_profile_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE", None);
        let _rotation_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH", None);
        let _previous_rotation_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH", None);
        let _required_approvals_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS", None);
        let _approved_signers_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS", None);

        let (_adapter, selection) =
            build_kolme_live_signer_adapter(Some("ops-primary"), Some("env-local"))
                .expect("primary env-local path should remain parity-stable");
        assert_eq!(selection.profile, "ops-primary");
        assert_eq!(selection.key_source, "env-local");
        assert_eq!(
            selection.private_key_env,
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX"
        );
    }

    {
        let _profile_env_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary"));
        let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
        let _secondary_key_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
            Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY),
        );
        let _fallback_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
        let _previous_profile_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE", None);
        let _rotation_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH", None);
        let _previous_rotation_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH", None);
        let _required_approvals_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS", None);
        let _approved_signers_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS", None);

        let (_adapter, selection) =
            build_kolme_live_signer_adapter(Some("ops-secondary"), Some("env-local"))
                .expect("secondary env-local path should remain parity-stable");
        assert_eq!(selection.profile, "ops-secondary");
        assert_eq!(selection.key_source, "env-local");
        assert_eq!(
            selection.private_key_env,
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY"
        );
    }

    {
        let _profile_env_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
        let _key_ref_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
            Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE),
        );
        let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
        let _fallback_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
        let _managed_command_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
            Some("printf 'managed-signer-ready\\n'"),
        );
        let managed_pubkey = managed_signer_public_key_hex(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE);
        let _managed_pubkey_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
            Some(managed_pubkey.as_str()),
        );
        let _previous_profile_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE", None);
        let _rotation_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH", None);
        let _previous_rotation_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH", None);
        let _required_approvals_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS", None);
        let _approved_signers_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS", None);

        let readiness =
            enforce_kolme_live_signer_preflight(Some("ops-primary"), Some("managed-external"))
                .expect("primary managed-external path should remain parity-stable");
        assert!(readiness.quorum_linked);
    }

    {
        let _profile_env_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary"));
        let _key_ref_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_KEY_REF_SECONDARY",
            Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE_SECONDARY),
        );
        let _secondary_key_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY", None);
        let _fallback_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
        let _managed_command_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
            Some("printf 'managed-signer-ready\\n'"),
        );
        let managed_pubkey =
            managed_signer_public_key_hex(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE_SECONDARY);
        let _managed_pubkey_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY",
            Some(managed_pubkey.as_str()),
        );
        let _previous_profile_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE", None);
        let _rotation_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH", None);
        let _previous_rotation_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH", None);
        let _required_approvals_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS", None);
        let _approved_signers_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS", None);

        let error =
            enforce_kolme_live_signer_preflight(Some("ops-secondary"), Some("managed-external"))
                .expect_err("secondary managed-external path must remain disallowed");
        assert!(
            matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("runtime_signer_key_source_profile_pair_disallowed")),
            "secondary managed-external pair must preserve disallowed reason marker"
        );
    }
}
