use std::fs;
use std::sync::Mutex;

use kamn_core::{SignerProviderHandshakeMatrix};

use super::super::{sign_kolme_live_managed_external_message, ManagedExternalKeySourceAdapter};
use super::super::super::{
    build_kolme_live_managed_signing_key, encode_kolme_hex_lower,
    KolmeLiveManagedKeySourceAdapter, KolmeLiveSignerSelection,
};
use super::support::{
    deterministic_request, managed_backend_env_lock, managed_external_core_signer_env_guards,
    unique_temp_path, write_managed_signer_script, EnvVarGuard,
    TEST_CORE_SIGNER_PRIVATE_KEY_HEX, TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
};

#[test]
fn unit_managed_key_source_adapter_emits_deterministic_provenance_marker() {
    let _lock = managed_backend_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _core_signer_key_guard = EnvVarGuard::set(
        "KAMN_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_CORE_SIGNER_PRIVATE_KEY_HEX),
    );
    let _core_service_key_guard = EnvVarGuard::set(
        "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX",
        Some(TEST_CORE_SIGNER_PRIVATE_KEY_HEX),
    );
    let request = deterministic_request("3955-provenance");
    let canonical_message = "{\"managed\":\"adapter-provenance\"}";
    let signing_key =
        build_kolme_live_managed_signing_key(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE)
            .expect("managed signing key should derive");
    let managed_pubkey = encode_kolme_hex_lower(
        signing_key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    );
    let (backend_signature, backend_recovery_id) = signing_key
        .sign_recoverable(canonical_message.as_bytes())
        .expect("managed signing key should sign canonical message");
    let backend_command = format!(
        "printf 'signature_hex={}\\nrecovery_id={}\\nsigner_public_key_hex={}\\n'",
        encode_kolme_hex_lower(backend_signature.to_bytes().as_ref()),
        backend_recovery_id.to_byte(),
        managed_pubkey
    );
    let _backend_command_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
        Some(backend_command.as_str()),
    );

    let selection = KolmeLiveSignerSelection {
        profile: "ops-primary",
        key_source: "managed-external",
        private_key_env: "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        key_reference_env: "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
    };
    let adapter = ManagedExternalKeySourceAdapter::with_provider_handshake_matrix(
        SignerProviderHandshakeMatrix::with_uniform_availability(true),
    );
    let output = KolmeLiveManagedKeySourceAdapter::sign_message(
        &adapter,
        &selection,
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
        &request,
        55,
        canonical_message,
        managed_pubkey.as_str(),
    )
    .expect("managed adapter should emit deterministic provenance marker");
    assert_eq!(output.provenance_marker.profile, selection.profile);
    assert_eq!(output.provenance_marker.key_source, selection.key_source);
    assert_eq!(output.provenance_marker.key_reference_env, selection.key_reference_env);
    assert_eq!(output.provenance_marker.signer_public_key_hex, managed_pubkey);
}

#[test]
fn regression_managed_backend_env_lock_aliases_shared_signer_lock() {
    let managed_lock = managed_backend_env_lock() as *const Mutex<()>;
    let shared_lock = crate::signer_test_env_lock() as *const Mutex<()>;
    assert_eq!(managed_lock, shared_lock, "managed backend tests must share signer env lock domain");
}

#[test]
fn regression_managed_external_backend_command_injection_payload_is_not_interpreted() {
    let _lock = managed_backend_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let (_core_signer_key_guard, _core_service_key_guard) = managed_external_core_signer_env_guards();

    let request = deterministic_request("5931-injection");
    let canonical_message = "{\"managed\":\"injection-contract\"}";
    let signing_key = build_kolme_live_managed_signing_key(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE)
        .expect("managed signing key should derive");
    let managed_pubkey = encode_kolme_hex_lower(
        signing_key.verifying_key().to_encoded_point(true).as_bytes(),
    );
    let (backend_signature, backend_recovery_id) = signing_key
        .sign_recoverable(canonical_message.as_bytes())
        .expect("managed signing key should sign canonical message");
    let signature_hex = encode_kolme_hex_lower(backend_signature.to_bytes().as_ref());

    let script_body = format!(
        "printf 'signature_hex={}\\nrecovery_id={}\\nsigner_public_key_hex={}\\n'\n",
        signature_hex,
        backend_recovery_id.to_byte(),
        managed_pubkey,
    );
    let script_path = write_managed_signer_script(script_body.as_str());
    let marker_path = unique_temp_path("managed-signer-injection-marker", ".txt");
    let _ = fs::remove_file(marker_path.as_path());
    let backend_command = format!("/bin/sh {} ; touch {}", script_path.display(), marker_path.display());
    let _backend_command_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
        Some(backend_command.as_str()),
    );
    let _backend_timeout_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS", Some("3"));

    let (observed_signature_hex, observed_recovery_id) = sign_kolme_live_managed_external_message(
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
        &request,
        1,
        canonical_message,
        SignerProviderHandshakeMatrix::with_uniform_availability(true),
        managed_pubkey.as_str(),
    )
    .expect("managed backend command injection payload must not execute");
    assert_eq!(observed_signature_hex, signature_hex);
    assert_eq!(observed_recovery_id, backend_recovery_id.to_byte());
    assert!(
        !marker_path.exists(),
        "shell injection payload must not be interpreted by managed backend command execution"
    );

    let _ = fs::remove_file(script_path.as_path());
    let _ = fs::remove_file(marker_path.as_path());
}

#[test]
fn regression_managed_external_backend_scrubs_signer_secret_env_for_child_process() {
    let _lock = managed_backend_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let (_core_signer_key_guard, _core_service_key_guard) = managed_external_core_signer_env_guards();

    let request = deterministic_request("5931-env-scrub");
    let canonical_message = "{\"managed\":\"env-scrub-contract\"}";
    let signing_key = build_kolme_live_managed_signing_key(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE)
        .expect("managed signing key should derive");
    let managed_pubkey = encode_kolme_hex_lower(
        signing_key.verifying_key().to_encoded_point(true).as_bytes(),
    );
    let (backend_signature, backend_recovery_id) = signing_key
        .sign_recoverable(canonical_message.as_bytes())
        .expect("managed signing key should sign canonical message");
    let signature_hex = encode_kolme_hex_lower(backend_signature.to_bytes().as_ref());

    let script_body = format!(
        "if [ -n \"$KAMN_SIGNER_PRIVATE_KEY_HEX\" ] || [ -n \"$KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX\" ]; then\n  echo 'managed signer secret env leaked' >&2\n  exit 91\nfi\nprintf 'signature_hex={}\\nrecovery_id={}\\nsigner_public_key_hex={}\\n'\n",
        signature_hex,
        backend_recovery_id.to_byte(),
        managed_pubkey,
    );
    let script_path = write_managed_signer_script(script_body.as_str());
    let backend_command = format!("/bin/sh {}", script_path.display());
    let _backend_command_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
        Some(backend_command.as_str()),
    );
    let _backend_timeout_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS", Some("3"));

    let (observed_signature_hex, observed_recovery_id) = sign_kolme_live_managed_external_message(
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
        &request,
        1,
        canonical_message,
        SignerProviderHandshakeMatrix::with_uniform_availability(true),
        managed_pubkey.as_str(),
    )
    .expect("managed backend child process must not receive signer secret env values");
    assert_eq!(observed_signature_hex, signature_hex);
    assert_eq!(observed_recovery_id, backend_recovery_id.to_byte());

    let _ = fs::remove_file(script_path.as_path());
}
