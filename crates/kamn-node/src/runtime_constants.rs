pub(crate) const KOLME_LIVE_PROVIDER_CONTRACT: &str = "KolmeRuntimeCommitLiveProvider";
pub(crate) const KOLME_LIVE_SIGNING_PROFILE: &str = "kolme-fork-secp256k1-v1";
pub(crate) const KOLME_IN_MEMORY_PROVIDER_MARKER: &str = "InMemoryKolmeRuntimeCommitClient";
pub(crate) const KOLME_LIVE_TRANSPORT_TIMEOUT_SECONDS: u64 = 30;
pub(crate) const KOLME_LIVE_FINALITY_STATUS_PATH: &str = "/runtime-commit/status";
pub(crate) const KOLME_LIVE_FINALITY_MAX_ATTEMPTS: u32 = 2;
pub(crate) const KOLME_LIVE_NONCE_PATH: &str = "/get-next-nonce";
pub(crate) const KOLME_LIVE_SIGNER_PROFILE_ENV: &str = "KAMN_KOLME_LIVE_SIGNER_PROFILE";
pub(crate) const KOLME_LIVE_SIGNER_PROFILE_PRIMARY: &str = "ops-primary";
pub(crate) const KOLME_LIVE_SIGNER_PROFILE_SECONDARY: &str = "ops-secondary";
pub(crate) const KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL: &str = "env-local";
pub(crate) const KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL: &str = "managed-external";
pub(crate) const KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_ENV: &str =
    "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX";
pub(crate) const KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY_ENV: &str =
    "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY";
pub(crate) const KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK_ENV: &str =
    "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK";
pub(crate) const KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_ENV: &str =
    "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX";
pub(crate) const KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY_ENV: &str =
    "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY";
pub(crate) const KOLME_LIVE_SIGNER_KEY_REF_ENV: &str = "KAMN_KOLME_LIVE_SIGNER_KEY_REF";
pub(crate) const KOLME_LIVE_SIGNER_KEY_REF_SECONDARY_ENV: &str =
    "KAMN_KOLME_LIVE_SIGNER_KEY_REF_SECONDARY";
pub(crate) const KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV: &str =
    "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND";
pub(crate) const KOLME_LIVE_MANAGED_SIGNER_REQUIRED_ENV: &str =
    "KAMN_KOLME_LIVE_MANAGED_SIGNER_REQUIRED";
pub(crate) const KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING_ENV: &str =
    "KAMN_KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING";
pub(crate) const KOLME_LIVE_ENV_LOCAL_SIGNER_KEY_SOURCE_FORBIDDEN_REASON_CODE: &str =
    "production_signer_key_source_env_local_forbidden";
pub(crate) const KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_ENV: &str =
    "KAMN_KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS";
pub(crate) const KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_DEFAULT: u64 = 5;
pub(crate) const KOLME_LIVE_MANAGED_SIGNER_POLL_INTERVAL_MILLIS: u64 = 10;
pub(crate) const KOLME_LIVE_NATIVE_CREATED_AT: &str = "2026-02-12T00:00:00Z";
pub(crate) const FULL_RUNTIME_BOOTSTRAP_COMPONENT_SEQUENCE: [&str; 4] =
    ["daemon", "api", "transport", "kolme-commit"];

#[cfg(test)]
pub(crate) fn daemon_test_env_lock() -> &'static std::sync::Mutex<()> {
    use std::sync::{Mutex, OnceLock};

    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[cfg(test)]
pub(crate) fn signer_test_env_lock() -> &'static std::sync::Mutex<()> {
    use std::sync::{Mutex, OnceLock};

    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}
