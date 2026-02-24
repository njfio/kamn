use kamn_core::signer_backend::CanonicalSecureKeyReference;
use kamn_core::{
    baseline_signature_for_fields, BackendSignature, BaselineTransaction, SignerBackendError,
    SignerBackendRouter, SignerProviderHandshakeMatrix, SignerProviderHandshakeStatus,
    SigningRequest, TransactionGuards, GENESIS_STATE_HASH,
};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

const TEST_SIGNER_PRIVATE_KEY_A_HEX: &str =
    "7f2dcf2ef6bcf53b1af2359954f04eb6d25688fd87cbf09f7f9db4c6522f4c6b";
const TEST_SIGNER_PRIVATE_KEY_B_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

fn signer_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct EnvVarGuard {
    key: &'static str,
    previous: Option<String>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: Option<&str>) -> Self {
        let previous = std::env::var(key).ok();
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.as_deref() {
            std::env::set_var(self.key, previous);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

fn with_default_signer_key_env<T>(run: impl FnOnce() -> T) -> T {
    let _lock = signer_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _generic_key_guard = EnvVarGuard::set(
        "KAMN_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_SIGNER_PRIVATE_KEY_A_HEX),
    );
    let _service_key_guard = EnvVarGuard::set(
        "KAMN_SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_HEX",
        Some(TEST_SIGNER_PRIVATE_KEY_A_HEX),
    );
    run()
}

#[test]
fn functional_secure_backend_signs_and_verifies_when_available() {
    with_default_signer_key_env(|| {
        let router = SignerBackendRouter::default();
        let request = SigningRequest::new(
            "secure:key-ops-1",
            "agent-a",
            1,
            "payload-1",
            GENESIS_STATE_HASH,
        )
        .expect("request should be valid");

        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("secure backend should sign");
        assert_eq!(signed.backend, "secure-mock");

        router
            .verify_with_backend(&signed.backend, &request, &signed.signature)
            .expect("signature should verify");
    });
}

#[test]
fn functional_aws_kms_provider_routes_to_production_adapter_backend() {
    with_default_signer_key_env(|| {
        let router = SignerBackendRouter::default();
        let request = SigningRequest::new(
            "secure:aws-kms:key-ops-1",
            "agent-a",
            1,
            "payload-1",
            GENESIS_STATE_HASH,
        )
        .expect("request should be valid");

        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("secure backend should sign");
        assert_eq!(signed.backend, "secure-aws-kms-emulator");

        router
            .verify_with_backend(&signed.backend, &request, &signed.signature)
            .expect("signature should verify");
    });
}

#[test]
fn functional_router_uses_custom_provider_client_mapping_for_secure_provider() {
    fn custom_provider_client(
        request: &SigningRequest,
        key_reference: &CanonicalSecureKeyReference,
    ) -> Result<BackendSignature, SignerBackendError> {
        Ok(BackendSignature {
            backend: key_reference.provider.backend_name().to_owned(),
            signature: format!(
                "provider-client:{}",
                baseline_signature_for_fields(
                    &request.sender,
                    request.nonce,
                    &request.state_hash,
                    &request.payload,
                )
            ),
        })
    }

    let router = SignerBackendRouter::with_provider_client(
        SignerProviderHandshakeMatrix::with_uniform_availability(true),
        custom_provider_client,
    );
    let request = SigningRequest::new(
        "secure:aws-kms:key-ops-1",
        "agent-a",
        1,
        "payload-1",
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");

    let signed = router
        .sign_with_secure_fallback(&request)
        .expect("provider client should sign through secure router");
    assert_eq!(signed.backend, "secure-aws-kms-emulator");
    assert!(signed
        .signature
        .starts_with("provider-client:sig:deterministic-v1:baseline-v1"));
}

#[test]
fn functional_secure_unavailable_falls_back_to_local_backend() {
    with_default_signer_key_env(|| {
        let router = SignerBackendRouter::with_secure_availability(false);
        let request = SigningRequest::new(
            "secure:key-ops-1",
            "agent-a",
            1,
            "payload-1",
            GENESIS_STATE_HASH,
        )
        .expect("request should be valid");

        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("fallback should sign");
        assert_eq!(signed.backend, "local-software");

        router
            .verify_with_backend(&signed.backend, &request, &signed.signature)
            .expect("fallback signature should verify");
    });
}

#[test]
fn regression_local_backend_signing_requires_explicit_key_material() {
    // Regression: #5913
    let _lock = signer_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _generic_key_guard = EnvVarGuard::set("KAMN_SIGNER_PRIVATE_KEY_HEX", None);
    let _service_key_guard = EnvVarGuard::set("KAMN_SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_HEX", None);
    let _key_specific_guard = EnvVarGuard::set(
        "KAMN_SIGNER_PRIVATE_KEY_HEX__SECURE_KEY_REGRESSION_5913_MISSING",
        None,
    );

    let router = SignerBackendRouter::with_secure_availability(false);
    let request = SigningRequest::new(
        "secure:key-regression-5913-missing",
        "agent-a",
        1,
        "payload-1",
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");

    let result = router.sign_with_secure_fallback(&request);
    assert!(
        matches!(
            result,
            Err(SignerBackendError::MissingSigningKeyMaterial { key_id, .. })
                if key_id == "secure:key-regression-5913-missing"
        ),
        "local signing must fail closed when signer key env is not provisioned"
    );
}

#[test]
fn functional_provider_handshake_matrix_routes_operator_fallback_for_unavailable_provider() {
    with_default_signer_key_env(|| {
        let router = SignerBackendRouter::with_provider_handshake_matrix(
            SignerProviderHandshakeMatrix::with_statuses(
                SignerProviderHandshakeStatus::Available,
                SignerProviderHandshakeStatus::Unavailable,
            ),
        );
        let request = SigningRequest::new(
            "secure:aws-kms:key-ops-1",
            "agent-a",
            1,
            "payload-1",
            GENESIS_STATE_HASH,
        )
        .expect("request should be valid");

        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("operator fallback should sign when provider is unavailable");
        assert_eq!(signed.backend, "local-software");
    });
}

#[test]
fn integration_router_signed_transaction_passes_transaction_guards() {
    with_default_signer_key_env(|| {
        let router = SignerBackendRouter::default();
        let mut guards = TransactionGuards::new();
        let mut tx = BaselineTransaction::signed(
            "tx-sign-1",
            "agent-a",
            1,
            "payload-sign-1",
            guards.expected_state_hash(),
        );

        let request = SigningRequest::for_transaction("secure:key-ops-2", &tx)
            .expect("request should map from transaction");
        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("signing should succeed");
        tx.signature = signed.signature;

        guards
            .validate_and_record(&tx)
            .expect("signed transaction should validate");
    });
}

#[test]
fn integration_aws_kms_signed_transaction_passes_transaction_guards() {
    with_default_signer_key_env(|| {
        let router = SignerBackendRouter::default();
        let mut guards = TransactionGuards::new();
        let mut tx = BaselineTransaction::signed(
            "tx-sign-aws-1",
            "agent-a",
            1,
            "payload-sign-1",
            guards.expected_state_hash(),
        );

        let request = SigningRequest::for_transaction("secure:aws-kms:key-ops-2", &tx)
            .expect("request should map from transaction");
        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("signing should succeed");
        assert_eq!(signed.backend, "secure-aws-kms-emulator");
        tx.signature = signed.signature;

        guards
            .validate_and_record(&tx)
            .expect("signed transaction should validate");
    });
}

#[test]
fn functional_admin_role_key_signs_when_sender_role_matches() {
    with_default_signer_key_env(|| {
        let router = SignerBackendRouter::default();
        let request = SigningRequest::new(
            "secure:aws-kms:role-admin/key-ops-1",
            "admin-agent-a",
            1,
            "payload-1",
            GENESIS_STATE_HASH,
        )
        .expect("request should be valid");

        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("signing should succeed");
        assert_eq!(signed.backend, "secure-aws-kms-emulator");

        router
            .verify_with_backend(&signed.backend, &request, &signed.signature)
            .expect("signature should verify");
    });
}

#[test]
fn regression_role_mismatch_signing_request_is_rejected() {
    // Regression: #619
    let router = SignerBackendRouter::default();
    let request = SigningRequest::new(
        "secure:aws-kms:role-admin/key-ops-1",
        "agent-a",
        1,
        "payload-1",
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");

    assert_eq!(
        router.sign_with_secure_fallback(&request),
        Err(SignerBackendError::KeyRoleMismatch {
            key_role: "admin".to_owned(),
            sender_role: "operator".to_owned(),
            sender: "agent-a".to_owned(),
            key_id: "secure:aws-kms:role-admin/key-ops-1".to_owned(),
        })
    );
}

#[test]
fn regression_admin_key_does_not_fallback_when_secure_provider_unavailable() {
    // Regression: #619
    let router = SignerBackendRouter::with_secure_availability(false);
    let request = SigningRequest::new(
        "secure:aws-kms:role-admin/key-ops-1",
        "admin-agent-a",
        1,
        "payload-1",
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");

    assert_eq!(
        router.sign_with_secure_fallback(&request),
        Err(SignerBackendError::FallbackDeniedByRolePolicy {
            key_role: "admin".to_owned(),
            key_id: "secure:aws-kms:role-admin/key-ops-1".to_owned(),
        })
    );
}

#[test]
fn functional_privileged_roles_deny_fallback_when_provider_unavailable() {
    let router = SignerBackendRouter::with_provider_handshake_matrix(
        SignerProviderHandshakeMatrix::with_statuses(
            SignerProviderHandshakeStatus::Available,
            SignerProviderHandshakeStatus::Unavailable,
        ),
    );
    let privileged_cases = [
        (
            "secure:aws-kms:role-admin/key-ops-1",
            "admin-agent-a",
            "admin",
        ),
        (
            "secure:aws-kms:role-treasury/key-ops-1",
            "treasury-agent-a",
            "treasury",
        ),
        (
            "secure:aws-kms:role-auditor/key-ops-1",
            "auditor-agent-a",
            "auditor",
        ),
    ];

    for (key_id, sender, role) in privileged_cases {
        let request = SigningRequest::new(key_id, sender, 1, "payload-1", GENESIS_STATE_HASH)
            .expect("request should be valid");

        assert_eq!(
            router.sign_with_secure_fallback(&request),
            Err(SignerBackendError::FallbackDeniedByRolePolicy {
                key_role: role.to_owned(),
                key_id: key_id.to_owned(),
            })
        );
    }
}

#[test]
fn regression_unsupported_secure_key_reference_does_not_fallback() {
    // Regression: #160
    let router = SignerBackendRouter::default();
    let request = SigningRequest::new("local:key-1", "agent-a", 1, "payload-1", GENESIS_STATE_HASH)
        .expect("request should be valid");

    assert_eq!(
        router.sign_with_secure_fallback(&request),
        Err(SignerBackendError::UnsupportedKeyReference {
            backend: "secure-mock".to_owned(),
            key_id: "local:key-1".to_owned(),
        })
    );
}

#[test]
fn regression_unknown_secure_provider_is_rejected_without_fallback() {
    // Regression: #619
    let router = SignerBackendRouter::default();
    let request = SigningRequest::new(
        "secure:gcp-kms:key-ops-1",
        "agent-a",
        1,
        "payload-1",
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");

    assert_eq!(
        router.sign_with_secure_fallback(&request),
        Err(SignerBackendError::UnsupportedSecureProvider {
            backend: "secure-mock".to_owned(),
            provider: "gcp-kms".to_owned(),
            key_id: "secure:gcp-kms:key-ops-1".to_owned(),
        })
    );
}

#[test]
fn regression_provider_handshake_policy_block_rejects_without_fallback() {
    // Regression: #677
    let router = SignerBackendRouter::with_provider_handshake_matrix(
        SignerProviderHandshakeMatrix::with_statuses(
            SignerProviderHandshakeStatus::Available,
            SignerProviderHandshakeStatus::PolicyBlocked,
        ),
    );
    let request = SigningRequest::new(
        "secure:aws-kms:key-ops-1",
        "agent-a",
        1,
        "payload-1",
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");

    assert_eq!(
        router.sign_with_secure_fallback(&request),
        Err(SignerBackendError::ProviderHandshakeRejected {
            backend: "secure-aws-kms-emulator".to_owned(),
            failure_class: "policy-blocked".to_owned(),
        })
    );
}

#[test]
fn regression_provider_client_backend_mismatch_is_rejected_without_fallback() {
    // Regression: #986
    fn mismatched_provider_client(
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

    let router = SignerBackendRouter::with_provider_client(
        SignerProviderHandshakeMatrix::with_uniform_availability(true),
        mismatched_provider_client,
    );
    let request = SigningRequest::new(
        "secure:aws-kms:key-ops-1",
        "agent-a",
        1,
        "payload-1",
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");

    assert_eq!(
        router.sign_with_secure_fallback(&request),
        Err(SignerBackendError::ProviderClientBackendMismatch {
            expected_backend: "secure-aws-kms-emulator".to_owned(),
            provided_backend: "secure-mock".to_owned(),
            key_id: "secure:aws-kms:key-ops-1".to_owned(),
        })
    );
}

#[test]
fn integration_signer_backend_accepts_baseline_v1_only_with_explicit_compatibility_switch() {
    let _lock = signer_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _compat_guard = EnvVarGuard::set("KAMN_SIGNER_ALLOW_LEGACY_BASELINE_V1", Some("1"));
    let router = SignerBackendRouter::default();
    let request = SigningRequest::new(
        "secure:key-ops-1",
        "agent-a",
        1,
        "payload-1",
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");
    let baseline_v1_signature =
        baseline_signature_for_fields("agent-a", 1, GENESIS_STATE_HASH, "payload-1");
    assert!(
        router
            .verify_with_backend("secure-mock", &request, baseline_v1_signature.as_str())
            .is_ok(),
        "baseline-v1 signatures should be accepted only when explicit compatibility switch is enabled"
    );
}

#[test]
fn regression_signer_backend_rejects_baseline_v1_signature_by_default() {
    // Regression: #5897
    let router = SignerBackendRouter::default();
    let request = SigningRequest::new(
        "secure:key-ops-1",
        "agent-a",
        1,
        "payload-1",
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");
    let baseline_v1_signature =
        baseline_signature_for_fields("agent-a", 1, GENESIS_STATE_HASH, "payload-1");

    assert!(
        router
            .verify_with_backend("secure-mock", &request, baseline_v1_signature.as_str())
            .is_err(),
        "baseline-v1 signatures must be rejected by default"
    );
}

#[test]
fn regression_local_backend_rejects_tampered_signature() {
    // Regression: #5897
    with_default_signer_key_env(|| {
        let router = SignerBackendRouter::with_secure_availability(false);
        let request = SigningRequest::new(
            "secure:key-ops-1",
            "agent-a",
            1,
            "payload-1",
            GENESIS_STATE_HASH,
        )
        .expect("request should be valid");
        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("local fallback should sign");
        assert_eq!(signed.backend, "local-software");
        let tampered_signature = format!("{}ff", signed.signature);

        assert!(
            router
                .verify_with_backend("local-software", &request, tampered_signature.as_str())
                .is_err(),
            "local backend must reject tampered signatures"
        );
    });
}

#[test]
fn regression_local_backend_rejects_signature_when_verifier_uses_wrong_key() {
    // Regression: #5897
    let _lock = signer_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _compat_guard = EnvVarGuard::set("KAMN_SIGNER_ALLOW_LEGACY_BASELINE_V1", None);
    let router = SignerBackendRouter::with_secure_availability(false);
    let request = SigningRequest::new(
        "secure:key-regression-5897-wrong-key",
        "agent-a",
        1,
        "payload-1",
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");

    let _signing_key_specific_guard = EnvVarGuard::set(
        "KAMN_SIGNER_PRIVATE_KEY_HEX__SECURE_KEY_REGRESSION_5897_WRONG_KEY",
        Some(TEST_SIGNER_PRIVATE_KEY_A_HEX),
    );
    let signed = router
        .sign_with_secure_fallback(&request)
        .expect("local fallback should sign");

    let _verifying_key_specific_guard = EnvVarGuard::set(
        "KAMN_SIGNER_PRIVATE_KEY_HEX__SECURE_KEY_REGRESSION_5897_WRONG_KEY",
        Some(TEST_SIGNER_PRIVATE_KEY_B_HEX),
    );
    assert!(
        router
            .verify_with_backend("local-software", &request, signed.signature.as_str())
            .is_err(),
        "local backend must reject signatures when verifier key material does not match signer key"
    );
}

#[test]
fn regression_local_backend_rejects_baseline_v1_signature_without_compat_switch() {
    // Regression: #5897
    let router = SignerBackendRouter::default();
    let request = SigningRequest::new(
        "secure:key-ops-1",
        "agent-a",
        1,
        "payload-1",
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");
    let baseline_v1_signature =
        baseline_signature_for_fields("agent-a", 1, GENESIS_STATE_HASH, "payload-1");

    assert!(
        router
            .verify_with_backend("local-software", &request, baseline_v1_signature.as_str())
            .is_err(),
        "baseline-v1 must not bypass local backend verification when compat switch is disabled"
    );
}

#[test]
fn regression_secure_provider_backend_mismatch_is_rejected() {
    // Regression: #619
    with_default_signer_key_env(|| {
        let router = SignerBackendRouter::default();
        let request = SigningRequest::new(
            "secure:aws-kms:key-ops-1",
            "agent-a",
            1,
            "payload-1",
            GENESIS_STATE_HASH,
        )
        .expect("request should be valid");
        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("signing should succeed");

        assert_eq!(
            router.verify_with_backend("secure-mock", &request, &signed.signature),
            Err(SignerBackendError::SecureProviderBackendMismatch {
                expected_backend: "secure-aws-kms-emulator".to_owned(),
                provided_backend: "secure-mock".to_owned(),
                key_id: "secure:aws-kms:key-ops-1".to_owned(),
            })
        );
    });
}

#[test]
fn for_transaction_rejects_empty_transaction_id() {
    let tx = BaselineTransaction {
        id: String::new(),
        sender: "agent-a".to_owned(),
        nonce: 1,
        payload: "payload-1".to_owned(),
        state_hash: GENESIS_STATE_HASH.to_owned(),
        signature: "sig:placeholder".to_owned(),
    };

    assert_eq!(
        SigningRequest::for_transaction("secure:key-ops-3", &tx),
        Err(SignerBackendError::EmptyField("transaction_id"))
    );
}

#[test]
fn regression_signing_request_matches_canonical_signature_profile() {
    // Regression: #400
    with_default_signer_key_env(|| {
        let router = SignerBackendRouter::default();
        let request = SigningRequest::new(
            "secure:key-ops-1",
            "agent-a",
            1,
            "payload-1",
            GENESIS_STATE_HASH,
        )
        .expect("request should be valid");

        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("signature should be produced");
        assert!(
            signed.signature.starts_with("sig:secp256k1:baseline-v2:"),
            "default signer path must emit cryptographic baseline-v2 signatures"
        );
    });
}

#[test]
fn regression_signatures_include_profile_identifier_segment() {
    // Regression: #404
    with_default_signer_key_env(|| {
        let router = SignerBackendRouter::default();
        let request = SigningRequest::new(
            "secure:key-ops-1",
            "agent-a",
            1,
            "payload-1",
            GENESIS_STATE_HASH,
        )
        .expect("request should be valid");

        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("signature should be produced");
        assert!(signed.signature.starts_with("sig:secp256k1:baseline-v2:"));
    });
}

#[test]
fn performance_signer_emulator_contract_lane_stays_within_budget() {
    with_default_signer_key_env(|| {
        let router = SignerBackendRouter::default();
        let start = Instant::now();

        for nonce in 1..=256 {
            let key_id = if nonce % 2 == 0 {
                "secure:key-ops-perf"
            } else {
                "secure:aws-kms:key-ops-perf"
            };
            let request = SigningRequest::new(
                key_id,
                "agent-a",
                nonce,
                &format!("payload-perf-{nonce}"),
                GENESIS_STATE_HASH,
            )
            .expect("request should be valid");
            let signed = router
                .sign_with_secure_fallback(&request)
                .expect("signature should be produced");
            let expected_backend = if nonce % 2 == 0 {
                "secure-mock"
            } else {
                "secure-aws-kms-emulator"
            };
            assert_eq!(signed.backend, expected_backend);
        }

        let elapsed_millis = start.elapsed().as_millis();
        let budget_millis = std::env::var("KAMN_SIGNER_EMULATOR_CONTRACT_BUDGET_MS")
            .ok()
            .and_then(|value| value.parse::<u128>().ok())
            .unwrap_or_else(|| {
                if std::env::var_os("CI").is_some() {
                    600
                } else {
                    250
                }
            });
        assert!(
            elapsed_millis < budget_millis,
            "signer emulator contract lane exceeded budget: elapsed={elapsed_millis}ms budget={budget_millis}ms"
        );
    });
}

#[test]
#[ignore = "scheduled provider integration lane"]
fn performance_signer_emulator_bulk_signing_deep_lane() {
    with_default_signer_key_env(|| {
        let secure_router = SignerBackendRouter::default();
        let fallback_router = SignerBackendRouter::with_secure_availability(false);

        for nonce in 1..=5000 {
            let request = SigningRequest::new(
                "secure:key-ops-deep",
                "agent-a",
                nonce,
                &format!("payload-deep-{nonce}"),
                GENESIS_STATE_HASH,
            )
            .expect("request should be valid");

            let (router, expected_backend) = if nonce % 10 == 0 {
                (&fallback_router, "local-software")
            } else {
                (&secure_router, "secure-mock")
            };

            let signed = router
                .sign_with_secure_fallback(&request)
                .expect("signature should be produced");
            assert_eq!(signed.backend, expected_backend);
        }
    });
}
