use super::*;

const SIGNER_EMULATOR_BUDGET_ENV_KEY: &str = "KAMN_SIGNER_EMULATOR_CONTRACT_BUDGET_MS";
const SIGNER_EMULATOR_DEEP_KEY_ID: &str = "secure:key-ops-deep";
const SIGNER_EMULATOR_AGENT_ID: &str = "agent-a";
const SIGNER_EMULATOR_DEEP_FALLBACK_EVERY: u64 = 10;

fn with_signer_emulator_budget_env<T>(
    budget_override: Option<&str>,
    ci_value: Option<&str>,
    run: impl FnOnce() -> T,
) -> T {
    let _lock = signer_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _budget_guard = EnvVarGuard::set(SIGNER_EMULATOR_BUDGET_ENV_KEY, budget_override);
    let _ci_guard = EnvVarGuard::set("CI", ci_value);
    run()
}

fn signer_emulator_deep_lane_request(nonce: u64) -> SigningRequest {
    SigningRequest::new(
        SIGNER_EMULATOR_DEEP_KEY_ID,
        SIGNER_EMULATOR_AGENT_ID,
        nonce,
        &format!("payload-deep-{nonce}"),
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid")
}

fn signer_emulator_deep_lane_router_and_backend<'a>(
    nonce: u64,
    secure_router: &'a SignerBackendRouter,
    fallback_router: &'a SignerBackendRouter,
) -> (&'a SignerBackendRouter, &'static str) {
    if nonce.is_multiple_of(SIGNER_EMULATOR_DEEP_FALLBACK_EVERY) {
        (fallback_router, "local-software")
    } else {
        (secure_router, "secure-mock")
    }
}

fn assert_signer_emulator_deep_lane_nonce(
    nonce: u64,
    secure_router: &SignerBackendRouter,
    fallback_router: &SignerBackendRouter,
) {
    let request = signer_emulator_deep_lane_request(nonce);
    let (router, expected_backend) =
        signer_emulator_deep_lane_router_and_backend(nonce, secure_router, fallback_router);
    let signed = router
        .sign_with_secure_fallback(&request)
        .expect("signature should be produced");
    assert_eq!(signed.backend, expected_backend);
}

pub(super) fn run_performance_signer_emulator_contract_lane_stays_within_budget() {
    with_default_signer_key_env(|| {
        let router = SignerBackendRouter::default();
        let start = Instant::now();

        for nonce in 1_u64..=256 {
            assert_signer_emulator_contract_signing(&router, nonce);
        }

        let elapsed_millis = start.elapsed().as_millis();
        let budget_millis = signer_emulator_contract_budget_millis();
        assert!(
            signer_emulator_within_budget(elapsed_millis, budget_millis),
            "signer emulator contract lane exceeded budget: elapsed={elapsed_millis}ms budget={budget_millis}ms"
        );
    });
}

pub(super) fn run_regression_signer_emulator_budget_comparator_allows_exact_boundary() {
    assert!(signer_emulator_within_budget(250, 250));
    assert!(!signer_emulator_within_budget(251, 250));
}

pub(super) fn run_regression_signer_emulator_budget_parser_rejects_invalid_override() {
    with_signer_emulator_budget_env(Some("abc"), None, || {
        let result = std::panic::catch_unwind(signer_emulator_contract_budget_millis);
        assert!(
            result.is_err(),
            "invalid budget override must fail loudly instead of silently falling back"
        );
    });
}

pub(super) fn run_regression_signer_emulator_budget_parser_uses_local_default_when_unset() {
    with_signer_emulator_budget_env(None, None, || {
        assert_eq!(signer_emulator_contract_budget_millis(), 300);
    });
}

pub(super) fn run_regression_signer_emulator_budget_parser_uses_ci_default_when_ci_set() {
    with_signer_emulator_budget_env(None, Some("true"), || {
        assert_eq!(signer_emulator_contract_budget_millis(), 600);
    });
}

pub(super) fn run_performance_signer_emulator_bulk_signing_deep_lane() {
    with_default_signer_key_env(|| {
        let secure_router = SignerBackendRouter::default();
        let fallback_router = SignerBackendRouter::with_secure_availability(false);

        for nonce in 1..=5000 {
            assert_signer_emulator_deep_lane_nonce(nonce, &secure_router, &fallback_router);
        }
    });
}
