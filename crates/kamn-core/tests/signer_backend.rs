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

fn signer_emulator_default_budget_millis() -> u128 {
    if std::env::var_os("CI").is_some() {
        600
    } else {
        300
    }
}

fn signer_emulator_contract_budget_millis() -> u128 {
    match std::env::var("KAMN_SIGNER_EMULATOR_CONTRACT_BUDGET_MS") {
        Ok(value) => value.parse::<u128>().unwrap_or_else(|cause| {
            panic!(
                "invalid KAMN_SIGNER_EMULATOR_CONTRACT_BUDGET_MS value: value={value} cause={cause}"
            )
        }),
        Err(std::env::VarError::NotPresent) => signer_emulator_default_budget_millis(),
        Err(error) => panic!("failed to read KAMN_SIGNER_EMULATOR_CONTRACT_BUDGET_MS: {error}"),
    }
}

fn signer_emulator_within_budget(elapsed_millis: u128, budget_millis: u128) -> bool {
    elapsed_millis <= budget_millis
}

fn signer_emulator_key_id_for_nonce(nonce: u64) -> &'static str {
    if nonce.is_multiple_of(2) {
        "secure:key-ops-perf"
    } else {
        "secure:aws-kms:key-ops-perf"
    }
}

fn signer_emulator_expected_backend_for_nonce(nonce: u64) -> &'static str {
    if nonce.is_multiple_of(2) {
        "secure-mock"
    } else {
        "secure-aws-kms-emulator"
    }
}

fn assert_signer_emulator_contract_signing(router: &SignerBackendRouter, nonce: u64) {
    let request = SigningRequest::new(
        signer_emulator_key_id_for_nonce(nonce),
        "agent-a",
        nonce,
        &format!("payload-perf-{nonce}"),
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");
    let signed = router
        .sign_with_secure_fallback(&request)
        .expect("signature should be produced");
    assert_eq!(
        signed.backend,
        signer_emulator_expected_backend_for_nonce(nonce)
    );
}

#[path = "signer_backend/signer_core_cases.rs"]
mod signer_core_cases;
#[path = "signer_backend/signer_emulator_cases.rs"]
mod signer_emulator_cases;
#[path = "signer_backend/signer_provider_cases.rs"]
mod signer_provider_cases;
#[path = "signer_backend/signer_request_cases.rs"]
mod signer_request_cases;
#[path = "signer_backend/signer_signature_cases.rs"]
mod signer_signature_cases;

#[test]
fn functional_secure_backend_signs_and_verifies_when_available() {
    signer_core_cases::run_functional_secure_backend_signs_and_verifies_when_available();
}

#[test]
fn functional_aws_kms_provider_routes_to_production_adapter_backend() {
    signer_core_cases::run_functional_aws_kms_provider_routes_to_production_adapter_backend();
}

#[test]
fn functional_router_uses_custom_provider_client_mapping_for_secure_provider() {
    signer_core_cases::run_functional_router_uses_custom_provider_client_mapping_for_secure_provider();
}

#[test]
fn functional_secure_unavailable_falls_back_to_local_backend() {
    signer_core_cases::run_functional_secure_unavailable_falls_back_to_local_backend();
}

#[test]
fn regression_local_backend_signing_requires_explicit_key_material() {
    signer_core_cases::run_regression_local_backend_signing_requires_explicit_key_material();
}

#[test]
fn functional_provider_handshake_matrix_routes_operator_fallback_for_unavailable_provider() {
    signer_provider_cases::run_functional_provider_handshake_matrix_routes_operator_fallback_for_unavailable_provider();
}

#[test]
fn integration_router_signed_transaction_passes_transaction_guards() {
    signer_core_cases::run_integration_router_signed_transaction_passes_transaction_guards();
}

#[test]
fn integration_aws_kms_signed_transaction_passes_transaction_guards() {
    signer_core_cases::run_integration_aws_kms_signed_transaction_passes_transaction_guards();
}

#[test]
fn functional_admin_role_key_signs_when_sender_role_matches() {
    signer_provider_cases::run_functional_admin_role_key_signs_when_sender_role_matches();
}

#[test]
fn regression_role_mismatch_signing_request_is_rejected() {
    signer_provider_cases::run_regression_role_mismatch_signing_request_is_rejected();
}

#[test]
fn regression_admin_key_does_not_fallback_when_secure_provider_unavailable() {
    signer_provider_cases::run_regression_admin_key_does_not_fallback_when_secure_provider_unavailable();
}

#[test]
fn functional_privileged_roles_deny_fallback_when_provider_unavailable() {
    signer_provider_cases::run_functional_privileged_roles_deny_fallback_when_provider_unavailable(
    );
}

#[test]
fn regression_unsupported_secure_key_reference_does_not_fallback() {
    signer_core_cases::run_regression_unsupported_secure_key_reference_does_not_fallback();
}

#[test]
fn regression_unknown_secure_provider_is_rejected_without_fallback() {
    signer_provider_cases::run_regression_unknown_secure_provider_is_rejected_without_fallback();
}

#[test]
fn regression_provider_handshake_policy_block_rejects_without_fallback() {
    signer_provider_cases::run_regression_provider_handshake_policy_block_rejects_without_fallback(
    );
}

#[test]
fn regression_provider_client_backend_mismatch_is_rejected_without_fallback() {
    signer_provider_cases::run_regression_provider_client_backend_mismatch_is_rejected_without_fallback();
}

#[test]
fn integration_signer_backend_accepts_baseline_v1_only_with_explicit_compatibility_switch() {
    signer_signature_cases::run_integration_signer_backend_accepts_baseline_v1_only_with_explicit_compatibility_switch();
}

#[test]
fn regression_signer_backend_rejects_baseline_v1_signature_by_default() {
    signer_signature_cases::run_regression_signer_backend_rejects_baseline_v1_signature_by_default(
    );
}

#[test]
fn regression_local_backend_rejects_tampered_signature() {
    signer_signature_cases::run_regression_local_backend_rejects_tampered_signature();
}

#[test]
fn regression_local_backend_rejects_signature_when_verifier_uses_wrong_key() {
    signer_signature_cases::run_regression_local_backend_rejects_signature_when_verifier_uses_wrong_key();
}

#[test]
fn regression_local_backend_rejects_baseline_v1_signature_without_compat_switch() {
    signer_signature_cases::run_regression_local_backend_rejects_baseline_v1_signature_without_compat_switch();
}

#[test]
fn regression_secure_provider_backend_mismatch_is_rejected() {
    signer_provider_cases::run_regression_secure_provider_backend_mismatch_is_rejected();
}

#[test]
fn for_transaction_rejects_empty_transaction_id() {
    signer_request_cases::run_for_transaction_rejects_empty_transaction_id();
}

#[test]
fn regression_signing_request_matches_canonical_signature_profile() {
    signer_request_cases::run_regression_signing_request_matches_canonical_signature_profile();
}

#[test]
fn regression_signatures_include_profile_identifier_segment() {
    signer_request_cases::run_regression_signatures_include_profile_identifier_segment();
}

#[test]
fn performance_signer_emulator_contract_lane_stays_within_budget() {
    signer_emulator_cases::run_performance_signer_emulator_contract_lane_stays_within_budget();
}

#[test]
fn regression_signer_emulator_budget_comparator_allows_exact_boundary() {
    signer_emulator_cases::run_regression_signer_emulator_budget_comparator_allows_exact_boundary();
}

#[test]
fn regression_signer_emulator_budget_parser_rejects_invalid_override() {
    signer_emulator_cases::run_regression_signer_emulator_budget_parser_rejects_invalid_override();
}

#[test]
fn regression_signer_emulator_budget_parser_uses_local_default_when_unset() {
    signer_emulator_cases::run_regression_signer_emulator_budget_parser_uses_local_default_when_unset();
}

#[test]
fn regression_signer_emulator_budget_parser_uses_ci_default_when_ci_set() {
    signer_emulator_cases::run_regression_signer_emulator_budget_parser_uses_ci_default_when_ci_set(
    );
}

#[test]
#[ignore = "scheduled provider integration lane"]
fn performance_signer_emulator_bulk_signing_deep_lane() {
    signer_emulator_cases::run_performance_signer_emulator_bulk_signing_deep_lane();
}
