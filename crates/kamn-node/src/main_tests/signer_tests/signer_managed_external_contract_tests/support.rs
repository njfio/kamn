use super::super::support::managed_external_core_signer_env_guards;
use super::super::*;
use crate::signer::KolmeLiveSignerSelection;

pub(super) fn managed_request(suffix: &str) -> KolmeRuntimeCommitRequest {
    KolmeRuntimeCommitRequest::deterministic(
        format!("op-node-live-{suffix}").as_str(),
        format!("state:node-live-{suffix}").as_str(),
        format!("kamn:did:agent:node-live-{suffix}").as_str(),
        1,
        format!("payload:node-live-{suffix}").as_str(),
    )
    .expect("request should build")
}

pub(super) fn managed_env(
    profile: &str,
    key_ref_env: &'static str,
    key_reference: &str,
) -> Vec<EnvVarGuard> {
    vec![
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some(profile)),
        EnvVarGuard::set(key_ref_env, Some(key_reference)),
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None),
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None),
    ]
}

pub(super) fn managed_pubkey_guard(key_reference: &str, env_name: &'static str) -> EnvVarGuard {
    let pubkey = managed_signer_public_key_hex(key_reference);
    EnvVarGuard::set(env_name, Some(pubkey.as_str()))
}

pub(super) fn primary_managed_core_env() -> (Vec<EnvVarGuard>, (EnvVarGuard, EnvVarGuard)) {
    managed_env_with_core(
        "ops-primary",
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
    )
}

pub(super) fn primary_managed_pubkey_guard() -> EnvVarGuard {
    managed_pubkey_guard(
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
        "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
    )
}

pub(super) fn backend_command_guard(
    request: &KolmeRuntimeCommitRequest,
    nonce: u64,
    signer_pubkey_hex: Option<&str>,
) -> EnvVarGuard {
    let backend_command = managed_backend_command(
        request,
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
        nonce,
        signer_pubkey_hex,
    );
    EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
        Some(backend_command.as_str()),
    )
}

pub(super) fn managed_backend_command(
    request: &KolmeRuntimeCommitRequest,
    key_reference: &str,
    nonce: u64,
    signer_pubkey_hex: Option<&str>,
) -> String {
    let signing_key = build_kolme_live_managed_signing_key(key_reference)
        .expect("managed signing key should derive");
    let managed_pubkey = managed_signer_public_key_hex(key_reference);
    let canonical_message =
        render_kolme_live_native_direct_message(request, managed_pubkey.as_str(), nonce)
            .expect("canonical message should render");
    let (signature, recovery_id) = signing_key
        .sign_recoverable(canonical_message.as_bytes())
        .expect("managed signing key should sign canonical message");
    render_backend_output(
        encode_kolme_hex_lower(signature.to_bytes().as_ref()).as_str(),
        recovery_id.to_byte(),
        signer_pubkey_hex,
    )
}

pub(super) fn render_backend_output(
    signature_hex: &str,
    recovery_id: u8,
    signer_pubkey_hex: Option<&str>,
) -> String {
    match signer_pubkey_hex {
        Some(pubkey) => format!(
            "printf 'signature_hex={}\\nrecovery_id={}\\nsigner_public_key_hex={}\\n'",
            signature_hex, recovery_id, pubkey,
        ),
        None => format!(
            "printf 'signature_hex={}\\nrecovery_id={}\\n'",
            signature_hex, recovery_id,
        ),
    }
}

pub(super) fn managed_build_error(
    request: &KolmeRuntimeCommitRequest,
    nonce: u64,
    account_id: &str,
    profile: Option<&str>,
    key_source: Option<&str>,
) -> ConfigError {
    let reply = format!(r#"{{"next_nonce":{nonce},"account_id":"{account_id}"}}"#);
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(&reply)]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        request,
        profile,
        key_source,
    )
    .expect_err("managed-external path must fail closed")
}

pub(super) fn managed_payload_and_request_count(
    request: &KolmeRuntimeCommitRequest,
    nonce: u64,
    account_id: &str,
    profile: Option<&str>,
    key_source: Option<&str>,
) -> (String, KolmeLiveSignerSelection, usize) {
    let reply = format!(r#"{{"next_nonce":{nonce},"account_id":"{account_id}"}}"#);
    let (base_url, requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(&reply)]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let (payload, selection) = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        request,
        profile,
        key_source,
    )
    .expect("managed-external signing should succeed through secure backend route");
    let count = requests.lock().expect("request mutex should lock").len();
    (payload, selection, count)
}

pub(super) fn assert_reason(error: ConfigError, reason_code: &str, context: &str) {
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains(reason_code)),
        "{context}"
    );
}

pub(super) fn assert_provider_reason(
    request: &KolmeRuntimeCommitRequest,
    availability: bool,
    expected_signer_public_key_hex: &str,
    reason_code: &str,
    context: &str,
) {
    let error = sign_kolme_live_managed_external_message(
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
        request,
        1,
        "payload:managed-signature",
        SignerProviderHandshakeMatrix::with_uniform_availability(availability),
        expected_signer_public_key_hex,
    )
    .expect_err(context);
    assert_reason(error, reason_code, context);
}

pub(super) fn managed_env_with_core(
    profile: &str,
    key_ref_env: &'static str,
    key_reference: &str,
) -> (Vec<EnvVarGuard>, (EnvVarGuard, EnvVarGuard)) {
    let core = managed_external_core_signer_env_guards();
    let env = managed_env(profile, key_ref_env, key_reference);
    (env, core)
}
