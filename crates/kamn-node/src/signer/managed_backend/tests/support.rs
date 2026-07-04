use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use kamn_core::KolmeRuntimeCommitRequest;

use crate::signer::{
    build_kolme_live_managed_signing_key, encode_kolme_hex_lower, KolmeLiveManagedKeySourceAdapter,
    KolmeLiveManagedKeySourceAdapterOutput, KolmeLiveSignerSelection,
    ManagedExternalKeySourceAdapter,
};
use kamn_core::SignerProviderHandshakeMatrix;

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

pub(super) fn lock_managed_backend_env() -> std::sync::MutexGuard<'static, ()> {
    managed_backend_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

pub(super) fn unique_temp_path(prefix: &str, suffix: &str) -> PathBuf {
    let process_id = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    env::temp_dir().join(format!("{prefix}-{process_id}-{nanos}{suffix}"))
}

pub(super) fn write_managed_signer_script(script_body: &str) -> PathBuf {
    let script_path = unique_temp_path("managed-signer-script", ".sh");
    fs::write(script_path.as_path(), script_body).expect("managed signer script should write");
    script_path
}

pub(super) struct ManagedBackendFixture {
    pub(super) request: KolmeRuntimeCommitRequest,
    pub(super) canonical_message: String,
    pub(super) signer_public_key_hex: String,
    pub(super) signature_hex: String,
    pub(super) recovery_id: u8,
}

pub(super) fn deterministic_managed_backend_fixture(suffix: &str) -> ManagedBackendFixture {
    let request = deterministic_request(suffix);
    let canonical_message = format!("{{\"managed\":\"{suffix}\"}}");
    let signing_key = build_kolme_live_managed_signing_key(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE)
        .expect("managed signing key should derive");
    let signer_public_key_hex = encode_kolme_hex_lower(
        signing_key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    );
    let (signature, recovery_id) = signing_key
        .sign_recoverable(canonical_message.as_bytes())
        .expect("managed signing key should sign canonical message");
    ManagedBackendFixture {
        request,
        canonical_message,
        signer_public_key_hex,
        signature_hex: encode_kolme_hex_lower(signature.to_bytes().as_ref()),
        recovery_id: recovery_id.to_byte(),
    }
}

pub(super) fn managed_signer_printf_command(
    signature_hex: &str,
    recovery_id: u8,
    signer_public_key_hex: &str,
) -> String {
    format!(
        "printf 'signature_hex={signature_hex}\\nrecovery_id={recovery_id}\\nsigner_public_key_hex={signer_public_key_hex}\\n'"
    )
}

pub(super) fn managed_signer_printf_script(
    signature_hex: &str,
    recovery_id: u8,
    signer_public_key_hex: &str,
) -> String {
    let command = managed_signer_printf_command(signature_hex, recovery_id, signer_public_key_hex);
    format!("{command}\n")
}

pub(super) fn managed_signer_command_guard(command: &str) -> EnvVarGuard {
    EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND", Some(command))
}

pub(super) fn managed_signer_timeout_guard() -> EnvVarGuard {
    EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS", Some("3"))
}

pub(super) fn managed_backend_selection() -> KolmeLiveSignerSelection {
    KolmeLiveSignerSelection {
        profile: "ops-primary",
        key_source: "managed-external",
        private_key_env: "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        key_reference_env: "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
    }
}

pub(super) fn managed_backend_adapter() -> ManagedExternalKeySourceAdapter {
    ManagedExternalKeySourceAdapter::with_provider_handshake_matrix(
        SignerProviderHandshakeMatrix::with_uniform_availability(true),
    )
}

pub(super) fn sign_with_managed_backend_adapter(
    fixture: &ManagedBackendFixture,
    nonce: u64,
) -> Result<KolmeLiveManagedKeySourceAdapterOutput, kamn_core::ConfigError> {
    let selection = managed_backend_selection();
    KolmeLiveManagedKeySourceAdapter::sign_message(
        &managed_backend_adapter(),
        &selection,
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
        &fixture.request,
        nonce,
        fixture.canonical_message.as_str(),
        fixture.signer_public_key_hex.as_str(),
    )
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
