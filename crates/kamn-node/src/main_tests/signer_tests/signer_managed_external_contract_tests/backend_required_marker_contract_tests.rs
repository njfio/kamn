use super::super::*;
use super::support::{assert_reason, managed_env, managed_pubkey_guard, managed_request};

#[test]
fn regression_kolme_live_managed_external_required_marker_rejects_invalid_boolean() {
    let _lock = lock_signer_env_guard();
    let _required_marker_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_REQUIRED",
        Some("invalid-bool"),
    );
    assert!(
        matches!(
            resolve_kolme_live_managed_signer_required_marker(),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("managed_signer_backend_required_invalid")
        ),
        "managed signer required marker must reject non-boolean values"
    );
}

#[test]
fn regression_kolme_live_managed_external_required_marker_forces_backend_command() {
    let _lock = lock_signer_env_guard();
    let _env = required_marker_env();
    let _pubkey_guard = managed_pubkey_guard(
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
        "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
    );
    let request = managed_request("2432-required-marker");
    let error = required_marker_error(&request);
    assert_reason(
        error,
        "managed_signer_backend_required_missing",
        "required marker should fail closed when backend command is absent",
    );
    assert_required_marker_stops_before_nonce(&request);
}

fn required_marker_env() -> Vec<EnvVarGuard> {
    let mut env = managed_env(
        "ops-primary",
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
    );
    env.push(EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_REQUIRED",
        Some("true"),
    ));
    env
}

fn required_marker_error(request: &KolmeRuntimeCommitRequest) -> ConfigError {
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":43,"account_id":"acct-2432-required"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        request,
        None,
        Some("managed-external"),
    )
    .expect_err("managed signer required marker must force backend command contract")
}

fn assert_required_marker_stops_before_nonce(request: &KolmeRuntimeCommitRequest) {
    let (base_url, requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":43,"account_id":"acct-2432-required"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let _ = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        request,
        None,
        Some("managed-external"),
    );
    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        0,
        "required-marker managed-external path must fail before nonce lookup"
    );
}
