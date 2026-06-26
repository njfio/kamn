use std::fs;
use std::sync::Mutex;

use kamn_core::SignerProviderHandshakeMatrix;

use super::super::sign_kolme_live_managed_external_message;
use super::support::{
    deterministic_managed_backend_fixture, lock_managed_backend_env, managed_backend_env_lock,
    managed_backend_selection, managed_external_core_signer_env_guards,
    managed_signer_command_guard, managed_signer_printf_command, managed_signer_printf_script,
    managed_signer_timeout_guard, sign_with_managed_backend_adapter, unique_temp_path,
    write_managed_signer_script, EnvVarGuard, TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
};

fn install_printf_backend_command(fixture: &super::support::ManagedBackendFixture) -> EnvVarGuard {
    managed_signer_command_guard(
        managed_signer_printf_command(
            fixture.signature_hex.as_str(),
            fixture.recovery_id,
            fixture.signer_public_key_hex.as_str(),
        )
        .as_str(),
    )
}

fn remove_file_if_present(path: &std::path::Path) {
    let _ = fs::remove_file(path);
}

fn build_injection_backend_command(
    fixture: &super::support::ManagedBackendFixture,
    marker_path: &std::path::Path,
) -> (std::path::PathBuf, String) {
    let script_body = managed_signer_printf_script(
        fixture.signature_hex.as_str(),
        fixture.recovery_id,
        fixture.signer_public_key_hex.as_str(),
    );
    let script_path = write_managed_signer_script(script_body.as_str());
    let backend_command = format!(
        "/bin/sh {} ; touch {}",
        script_path.display(),
        marker_path.display()
    );
    (script_path, backend_command)
}

fn assert_matching_signature(
    observed_signature_hex: String,
    observed_recovery_id: u8,
    fixture: &super::support::ManagedBackendFixture,
) {
    assert_eq!(observed_signature_hex, fixture.signature_hex);
    assert_eq!(observed_recovery_id, fixture.recovery_id);
}

fn execute_managed_backend_signing(
    fixture: &super::support::ManagedBackendFixture,
) -> Result<(String, u8), kamn_core::ConfigError> {
    sign_kolme_live_managed_external_message(
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
        &fixture.request,
        1,
        fixture.canonical_message.as_str(),
        SignerProviderHandshakeMatrix::with_uniform_availability(true),
        fixture.signer_public_key_hex.as_str(),
    )
}

#[test]
fn unit_managed_key_source_adapter_emits_deterministic_provenance_marker() {
    let _lock = lock_managed_backend_env();
    let (_core_signer_key_guard, _core_service_key_guard) =
        managed_external_core_signer_env_guards();
    let fixture = deterministic_managed_backend_fixture("adapter-provenance");
    let _backend_command_guard = install_printf_backend_command(&fixture);

    let selection = managed_backend_selection();
    let output = sign_with_managed_backend_adapter(&fixture, 55)
        .expect("managed adapter should emit deterministic provenance marker");
    assert_eq!(output.provenance_marker.profile, selection.profile);
    assert_eq!(output.provenance_marker.key_source, selection.key_source);
    assert_eq!(
        output.provenance_marker.key_reference_env,
        selection.key_reference_env
    );
    assert_eq!(
        output.provenance_marker.signer_public_key_hex,
        fixture.signer_public_key_hex
    );
}

#[test]
fn regression_managed_backend_env_lock_aliases_shared_signer_lock() {
    let managed_lock = managed_backend_env_lock() as *const Mutex<()>;
    let shared_lock = crate::signer_test_env_lock() as *const Mutex<()>;
    assert_eq!(
        managed_lock, shared_lock,
        "managed backend tests must share signer env lock domain"
    );
}

#[test]
fn regression_managed_external_backend_command_injection_payload_is_not_interpreted() {
    let _lock = lock_managed_backend_env();
    let (_core_signer_key_guard, _core_service_key_guard) =
        managed_external_core_signer_env_guards();

    let fixture = deterministic_managed_backend_fixture("injection-contract");
    let marker_path = unique_temp_path("managed-signer-injection-marker", ".txt");
    remove_file_if_present(marker_path.as_path());
    let (script_path, backend_command) =
        build_injection_backend_command(&fixture, marker_path.as_path());
    let _backend_command_guard = managed_signer_command_guard(backend_command.as_str());
    let _backend_timeout_guard = managed_signer_timeout_guard();

    let (observed_signature_hex, observed_recovery_id) = execute_managed_backend_signing(&fixture)
        .expect("managed backend command injection payload must not execute");
    assert_matching_signature(observed_signature_hex, observed_recovery_id, &fixture);
    assert!(
        !marker_path.exists(),
        "shell injection payload must not be interpreted by managed backend command execution"
    );

    remove_file_if_present(script_path.as_path());
    remove_file_if_present(marker_path.as_path());
}

#[test]
fn regression_managed_external_backend_scrubs_signer_secret_env_for_child_process() {
    let _lock = lock_managed_backend_env();
    let (_core_signer_key_guard, _core_service_key_guard) =
        managed_external_core_signer_env_guards();

    let fixture = deterministic_managed_backend_fixture("env-scrub-contract");
    let script_body = format!(
        "if [ -n \"$KAMN_SIGNER_PRIVATE_KEY_HEX\" ] || [ -n \"$KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX\" ]; then\n  echo 'managed signer secret env leaked' >&2\n  exit 91\nfi\nprintf 'signature_hex={}\\nrecovery_id={}\\nsigner_public_key_hex={}\\n'\n",
        fixture.signature_hex,
        fixture.recovery_id,
        fixture.signer_public_key_hex,
    );
    let script_path = write_managed_signer_script(script_body.as_str());
    let backend_command = format!("/bin/sh {}", script_path.display());
    let _backend_command_guard = managed_signer_command_guard(backend_command.as_str());
    let _backend_timeout_guard = managed_signer_timeout_guard();

    let (observed_signature_hex, observed_recovery_id) = execute_managed_backend_signing(&fixture)
        .expect("managed backend child process must not receive signer secret env values");
    assert_matching_signature(observed_signature_hex, observed_recovery_id, &fixture);

    remove_file_if_present(script_path.as_path());
}
