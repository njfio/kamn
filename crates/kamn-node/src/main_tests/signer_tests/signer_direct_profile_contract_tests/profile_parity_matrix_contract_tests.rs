use super::super::*;
use super::support::set_env_vars;

struct SignerProfileParityCase {
    env: &'static [(&'static str, Option<&'static str>)],
    profile: &'static str,
    key_source: &'static str,
    private_key_env: &'static str,
    expect_ready: bool,
}

const PRIMARY_ENV_LOCAL_CASE: SignerProfileParityCase = SignerProfileParityCase {
    env: &[
        ("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary")),
        (
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
            Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
        ),
        ("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY", None),
        ("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None),
        ("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE", None),
        ("KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH", None),
        ("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH", None),
        ("KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS", None),
        ("KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS", None),
    ],
    profile: "ops-primary",
    key_source: "env-local",
    private_key_env: "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
    expect_ready: true,
};

const SECONDARY_ENV_LOCAL_CASE: SignerProfileParityCase = SignerProfileParityCase {
    env: &[
        ("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary")),
        ("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None),
        (
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
            Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY),
        ),
        ("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None),
        ("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE", None),
        ("KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH", None),
        ("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH", None),
        ("KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS", None),
        ("KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS", None),
    ],
    profile: "ops-secondary",
    key_source: "env-local",
    private_key_env: "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
    expect_ready: true,
};

const PRIMARY_MANAGED_CASE: SignerProfileParityCase = SignerProfileParityCase {
    env: &[
        ("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary")),
        (
            "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
            Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE),
        ),
        ("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None),
        ("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None),
        (
            "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
            Some("printf 'managed-signer-ready\\n'"),
        ),
        ("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE", None),
        ("KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH", None),
        ("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH", None),
        ("KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS", None),
        ("KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS", None),
    ],
    profile: "ops-primary",
    key_source: "managed-external",
    private_key_env: "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
    expect_ready: true,
};

const SECONDARY_MANAGED_CASE: SignerProfileParityCase = SignerProfileParityCase {
    env: &[
        ("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary")),
        (
            "KAMN_KOLME_LIVE_SIGNER_KEY_REF_SECONDARY",
            Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE_SECONDARY),
        ),
        ("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY", None),
        ("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None),
        (
            "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
            Some("printf 'managed-signer-ready\\n'"),
        ),
        ("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE", None),
        ("KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH", None),
        ("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH", None),
        ("KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS", None),
        ("KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS", None),
    ],
    profile: "ops-secondary",
    key_source: "managed-external",
    private_key_env: "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
    expect_ready: false,
};

#[test]
fn functional_signer_migration_profile_key_source_parity_matrix() {
    let _lock = lock_signer_env_guard();
    assert_env_local_case(&PRIMARY_ENV_LOCAL_CASE);
    assert_env_local_case(&SECONDARY_ENV_LOCAL_CASE);
    assert_managed_case(
        &PRIMARY_MANAGED_CASE,
        "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
    );
    assert_managed_case(
        &SECONDARY_MANAGED_CASE,
        "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY",
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE_SECONDARY,
    );
}

fn assert_env_local_case(case: &SignerProfileParityCase) {
    let _env = set_env_vars(case.env);
    let (_adapter, selection) =
        build_kolme_live_signer_adapter(Some(case.profile), Some(case.key_source))
            .expect("env-local path should remain parity-stable");
    assert_eq!(selection.profile, case.profile);
    assert_eq!(selection.key_source, case.key_source);
    assert_eq!(selection.private_key_env, case.private_key_env);
}

fn assert_managed_case(
    case: &SignerProfileParityCase,
    pubkey_env: &'static str,
    key_reference: &str,
) {
    let pubkey = managed_signer_public_key_hex(key_reference);
    let _env = set_env_vars(case.env);
    let _pubkey_guard = EnvVarGuard::set(pubkey_env, Some(pubkey.as_str()));
    if case.expect_ready {
        assert_managed_ready(case);
        return;
    }
    assert_managed_disallowed(case);
}

fn assert_managed_ready(case: &SignerProfileParityCase) {
    let readiness = enforce_kolme_live_signer_preflight(Some(case.profile), Some(case.key_source))
        .expect("managed-external path should remain parity-stable");
    assert!(readiness.quorum_linked);
}

fn assert_managed_disallowed(case: &SignerProfileParityCase) {
    let error = enforce_kolme_live_signer_preflight(Some(case.profile), Some(case.key_source))
        .expect_err("secondary managed-external path must remain disallowed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("runtime_signer_key_source_profile_pair_disallowed")),
        "secondary managed-external pair must preserve disallowed reason marker"
    );
}
