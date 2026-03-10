use super::*;

pub(super) fn managed_external_core_signer_env_guards() -> (EnvVarGuard, EnvVarGuard) {
    (
        EnvVarGuard::set(
            "KAMN_SIGNER_PRIVATE_KEY_HEX",
            Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
        ),
        EnvVarGuard::set(
            "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX",
            Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
        ),
    )
}
