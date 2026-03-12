use super::KolmeLiveSignerSelection;
use std::env;
use std::sync::Mutex;

pub(super) const TEST_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";
pub(super) const TEST_PRIVATE_KEY_HEX_SECONDARY: &str =
    "838c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";
pub(super) const TEST_PRIVATE_KEY_ENV: &str = "TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX";

pub(super) fn test_signer_env_lock() -> &'static Mutex<()> {
    crate::signer_test_env_lock()
}

pub(super) fn lock_signer_env_guard() -> std::sync::MutexGuard<'static, ()> {
    match test_signer_env_lock().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

pub(super) struct EnvVarGuard {
    key: &'static str,
    previous: Option<String>,
}

impl EnvVarGuard {
    pub(super) fn set(key: &'static str, value: Option<&str>) -> Self {
        let previous = env::var(key).ok();
        match value {
            Some(value) => env::set_var(key, value),
            None => env::remove_var(key),
        }
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.as_deref() {
            env::set_var(self.key, previous);
        } else {
            env::remove_var(self.key);
        }
    }
}

pub(super) fn test_primary_selection() -> KolmeLiveSignerSelection {
    KolmeLiveSignerSelection {
        profile: "ops-primary",
        key_source: "env-local",
        private_key_env: "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        key_reference_env: "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
    }
}

pub(super) fn test_primary_managed_selection() -> KolmeLiveSignerSelection {
    KolmeLiveSignerSelection {
        profile: "ops-primary",
        key_source: "managed-external",
        private_key_env: "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        key_reference_env: "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
    }
}

pub(super) fn is_zeroized_hex_buffer(value: &str) -> bool {
    value.as_bytes().iter().all(|byte| *byte == 0)
}
