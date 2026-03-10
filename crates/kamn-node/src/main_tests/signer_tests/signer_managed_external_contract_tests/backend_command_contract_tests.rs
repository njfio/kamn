use super::super::*;
use super::support::{assert_reason, managed_env, managed_pubkey_guard, managed_request};

#[test]
fn regression_kolme_live_managed_external_strict_contracts_require_backend_command_marker() {
    let _lock = lock_signer_env_guard();
    let _env = strict_backend_command_env();
    let _pubkey_guard = managed_pubkey_guard(
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
        "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
    );
    let request = managed_request("2432-missing-backend-command");
    let error = strict_backend_command_error(&request);
    assert_reason(
        error,
        "managed_signer_backend_required_missing",
        "strict managed-external runtime path must fail closed with deterministic missing backend reason code",
    );
    assert_zero_nonce_requests(&request, 42, "acct-2432");
}

#[test]
fn regression_kolme_live_managed_external_requires_backend_command_without_required_marker() {
    let _lock = lock_signer_env_guard();
    let _env = permissive_backend_command_env();
    let _pubkey_guard = managed_pubkey_guard(
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
        "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
    );
    let request = managed_request("2505-command-required");
    let error = permissive_backend_command_error(&request);
    assert_reason(
        error,
        "managed_signer_backend_required_missing",
        "managed-external signer mode without backend command must fail closed",
    );
}

fn strict_backend_command_env() -> Vec<EnvVarGuard> {
    managed_env(
        "ops-primary",
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
    )
}

fn permissive_backend_command_env() -> Vec<EnvVarGuard> {
    let mut env = strict_backend_command_env();
    env.push(EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_REQUIRED",
        None,
    ));
    env
}

fn strict_backend_command_error(request: &KolmeRuntimeCommitRequest) -> ConfigError {
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":42,"account_id":"acct-2432"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        request,
        Some("ops-primary"),
        Some("managed-external"),
    )
    .expect_err("strict managed-external signer contracts must require backend command marker")
}

fn permissive_backend_command_error(request: &KolmeRuntimeCommitRequest) -> ConfigError {
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":44,"account_id":"acct-2505-required"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        request,
        None,
        Some("managed-external"),
    )
    .expect_err("managed-external signer mode must require backend command marker")
}

fn assert_zero_nonce_requests(request: &KolmeRuntimeCommitRequest, nonce: u64, account_id: &str) {
    let reply = format!(r#"{{"next_nonce":{nonce},"account_id":"{account_id}"}}"#);
    let (base_url, requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(&reply)]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let _ = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        request,
        Some("ops-primary"),
        Some("managed-external"),
    );
    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        0,
        "managed-external missing backend command must fail before nonce lookup"
    );
}
