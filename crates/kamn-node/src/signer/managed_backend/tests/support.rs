use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use kamn_core::KolmeRuntimeCommitRequest;

pub(super) const TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE: &str =
    "secure:aws-kms:role-operator/key-live-ops-primary";
pub(super) const TEST_CORE_SIGNER_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

pub(super) fn managed_external_core_signer_env_guards() -> (EnvVarGuard, EnvVarGuard) {
    (
        EnvVarGuard::set(
            "KAMN_SIGNER_PRIVATE_KEY_HEX",
            Some(TEST_CORE_SIGNER_PRIVATE_KEY_HEX),
        ),
        EnvVarGuard::set(
            "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX",
            Some(TEST_CORE_SIGNER_PRIVATE_KEY_HEX),
        ),
    )
}

pub(super) fn managed_backend_env_lock() -> &'static Mutex<()> {
    crate::signer_test_env_lock()
}

pub(super) fn unique_temp_path(prefix: &str, suffix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    env::temp_dir().join(format!("{prefix}-{}-{nanos}{suffix}", std::process::id()))
}

pub(super) fn write_managed_signer_script(script_body: &str) -> PathBuf {
    let script_path = unique_temp_path("managed-signer-script", ".sh");
    fs::write(script_path.as_path(), script_body).expect("managed signer script should write");
    script_path
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

pub(super) fn deterministic_request(suffix: &str) -> KolmeRuntimeCommitRequest {
    KolmeRuntimeCommitRequest::deterministic(
        format!("op-node-live-{suffix}").as_str(),
        format!("state:node-live-{suffix}").as_str(),
        format!("kamn:did:agent:node-live-{suffix}").as_str(),
        1,
        format!("payload:node-live-{suffix}").as_str(),
    )
    .expect("request should build")
}
