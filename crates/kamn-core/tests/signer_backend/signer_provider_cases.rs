use super::*;

const PROVIDER_HANDSHAKE_KEY_ID: &str = "secure:aws-kms:key-ops-1";
const PROVIDER_ADMIN_KEY_ID: &str = "secure:aws-kms:role-admin/key-ops-1";
const PROVIDER_UNKNOWN_KEY_ID: &str = "secure:gcp-kms:key-ops-1";
const PROVIDER_OPERATOR_SENDER: &str = "agent-a";
const PROVIDER_ADMIN_SENDER: &str = "admin-agent-a";
const PROVIDER_DEFAULT_NONCE: u64 = 1;
const PROVIDER_DEFAULT_PAYLOAD: &str = "payload-1";

fn signer_provider_request(key_id: &str, sender: &str) -> SigningRequest {
    SigningRequest::new(
        key_id,
        sender,
        PROVIDER_DEFAULT_NONCE,
        PROVIDER_DEFAULT_PAYLOAD,
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid")
}

fn signer_provider_unavailable_router() -> SignerBackendRouter {
    SignerBackendRouter::with_provider_handshake_matrix(SignerProviderHandshakeMatrix::with_statuses(
        SignerProviderHandshakeStatus::Available,
        SignerProviderHandshakeStatus::Unavailable,
    ))
}

fn signer_provider_policy_block_router() -> SignerBackendRouter {
    SignerBackendRouter::with_provider_handshake_matrix(SignerProviderHandshakeMatrix::with_statuses(
        SignerProviderHandshakeStatus::Available,
        SignerProviderHandshakeStatus::PolicyBlocked,
    ))
}

fn signer_provider_client_mismatch(
    request: &SigningRequest,
    _key_reference: &CanonicalSecureKeyReference,
) -> Result<BackendSignature, SignerBackendError> {
    Ok(BackendSignature {
        backend: "secure-mock".to_owned(),
        signature: baseline_signature_for_fields(
            &request.sender,
            request.nonce,
            &request.state_hash,
            &request.payload,
        ),
    })
}

fn assert_provider_fallback_denied(router: &SignerBackendRouter, key_id: &str, sender: &str, role: &str) {
    let request = signer_provider_request(key_id, sender);
    assert_eq!(
        router.sign_with_secure_fallback(&request),
        Err(SignerBackendError::FallbackDeniedByRolePolicy {
            key_role: role.to_owned(),
            key_id: key_id.to_owned(),
        })
    );
}

pub(super) fn run_functional_provider_handshake_matrix_routes_operator_fallback_for_unavailable_provider() {
    with_default_signer_key_env(|| {
        let router = signer_provider_unavailable_router();
        let request = signer_provider_request(PROVIDER_HANDSHAKE_KEY_ID, PROVIDER_OPERATOR_SENDER);
        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("operator fallback should sign when provider is unavailable");
        assert_eq!(signed.backend, "local-software");
    });
}

pub(super) fn run_functional_admin_role_key_signs_when_sender_role_matches() {
    with_default_signer_key_env(|| {
        let router = SignerBackendRouter::default();
        let request = signer_provider_request(PROVIDER_ADMIN_KEY_ID, PROVIDER_ADMIN_SENDER);
        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("signing should succeed");
        assert_eq!(signed.backend, "secure-aws-kms-emulator");
        router
            .verify_with_backend(&signed.backend, &request, &signed.signature)
            .expect("signature should verify");
    });
}

pub(super) fn run_regression_role_mismatch_signing_request_is_rejected() {
    // Regression: #619
    let router = SignerBackendRouter::default();
    let request = signer_provider_request(PROVIDER_ADMIN_KEY_ID, PROVIDER_OPERATOR_SENDER);
    assert_eq!(
        router.sign_with_secure_fallback(&request),
        Err(SignerBackendError::KeyRoleMismatch {
            key_role: "admin".to_owned(),
            sender_role: "operator".to_owned(),
            sender: PROVIDER_OPERATOR_SENDER.to_owned(),
            key_id: PROVIDER_ADMIN_KEY_ID.to_owned(),
        })
    );
}

pub(super) fn run_regression_admin_key_does_not_fallback_when_secure_provider_unavailable() {
    // Regression: #619
    let router = SignerBackendRouter::with_secure_availability(false);
    let request = signer_provider_request(PROVIDER_ADMIN_KEY_ID, PROVIDER_ADMIN_SENDER);
    assert_eq!(
        router.sign_with_secure_fallback(&request),
        Err(SignerBackendError::FallbackDeniedByRolePolicy {
            key_role: "admin".to_owned(),
            key_id: PROVIDER_ADMIN_KEY_ID.to_owned(),
        })
    );
}

pub(super) fn run_functional_privileged_roles_deny_fallback_when_provider_unavailable() {
    let router = signer_provider_unavailable_router();
    let privileged_cases = [
        (PROVIDER_ADMIN_KEY_ID, PROVIDER_ADMIN_SENDER, "admin"),
        ("secure:aws-kms:role-treasury/key-ops-1", "treasury-agent-a", "treasury"),
        ("secure:aws-kms:role-auditor/key-ops-1", "auditor-agent-a", "auditor"),
    ];
    for (key_id, sender, role) in privileged_cases {
        assert_provider_fallback_denied(&router, key_id, sender, role);
    }
}

pub(super) fn run_regression_unknown_secure_provider_is_rejected_without_fallback() {
    // Regression: #619
    let router = SignerBackendRouter::default();
    let request = signer_provider_request(PROVIDER_UNKNOWN_KEY_ID, PROVIDER_OPERATOR_SENDER);
    assert_eq!(
        router.sign_with_secure_fallback(&request),
        Err(SignerBackendError::UnsupportedSecureProvider {
            backend: "secure-mock".to_owned(),
            provider: "gcp-kms".to_owned(),
            key_id: PROVIDER_UNKNOWN_KEY_ID.to_owned(),
        })
    );
}

pub(super) fn run_regression_provider_handshake_policy_block_rejects_without_fallback() {
    // Regression: #677
    let router = signer_provider_policy_block_router();
    let request = signer_provider_request(PROVIDER_HANDSHAKE_KEY_ID, PROVIDER_OPERATOR_SENDER);
    assert_eq!(
        router.sign_with_secure_fallback(&request),
        Err(SignerBackendError::ProviderHandshakeRejected {
            backend: "secure-aws-kms-emulator".to_owned(),
            failure_class: "policy-blocked".to_owned(),
        })
    );
}

pub(super) fn run_regression_provider_client_backend_mismatch_is_rejected_without_fallback() {
    // Regression: #986
    let router = SignerBackendRouter::with_provider_client(
        SignerProviderHandshakeMatrix::with_uniform_availability(true),
        signer_provider_client_mismatch,
    );
    let request = signer_provider_request(PROVIDER_HANDSHAKE_KEY_ID, PROVIDER_OPERATOR_SENDER);
    assert_eq!(
        router.sign_with_secure_fallback(&request),
        Err(SignerBackendError::ProviderClientBackendMismatch {
            expected_backend: "secure-aws-kms-emulator".to_owned(),
            provided_backend: "secure-mock".to_owned(),
            key_id: PROVIDER_HANDSHAKE_KEY_ID.to_owned(),
        })
    );
}

pub(super) fn run_regression_secure_provider_backend_mismatch_is_rejected() {
    // Regression: #619
    with_default_signer_key_env(|| {
        let router = SignerBackendRouter::default();
        let request = signer_provider_request(PROVIDER_HANDSHAKE_KEY_ID, PROVIDER_OPERATOR_SENDER);
        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("signing should succeed");
        assert_eq!(
            router.verify_with_backend("secure-mock", &request, &signed.signature),
            Err(SignerBackendError::SecureProviderBackendMismatch {
                expected_backend: "secure-aws-kms-emulator".to_owned(),
                provided_backend: "secure-mock".to_owned(),
                key_id: PROVIDER_HANDSHAKE_KEY_ID.to_owned(),
            })
        );
    });
}
