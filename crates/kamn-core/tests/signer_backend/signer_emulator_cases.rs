use super::*;

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
    let _lock = signer_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _budget_guard = EnvVarGuard::set("KAMN_SIGNER_EMULATOR_CONTRACT_BUDGET_MS", Some("abc"));

    let result = std::panic::catch_unwind(signer_emulator_contract_budget_millis);
    assert!(
        result.is_err(),
        "invalid budget override must fail loudly instead of silently falling back"
    );
}

pub(super) fn run_regression_signer_emulator_budget_parser_uses_local_default_when_unset() {
    let _lock = signer_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _budget_guard = EnvVarGuard::set("KAMN_SIGNER_EMULATOR_CONTRACT_BUDGET_MS", None);
    let _ci_guard = EnvVarGuard::set("CI", None);

    assert_eq!(signer_emulator_contract_budget_millis(), 300);
}

pub(super) fn run_regression_signer_emulator_budget_parser_uses_ci_default_when_ci_set() {
    let _lock = signer_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _budget_guard = EnvVarGuard::set("KAMN_SIGNER_EMULATOR_CONTRACT_BUDGET_MS", None);
    let _ci_guard = EnvVarGuard::set("CI", Some("true"));

    assert_eq!(signer_emulator_contract_budget_millis(), 600);
}

pub(super) fn run_performance_signer_emulator_bulk_signing_deep_lane() {
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
