use super::*;

#[test]
fn unit_run_live_s06_proof_verification_probe_rejects_invalid_block_height_env_value() {
    with_env_vars(
        &[("KAMN_E2E_S06_PROOF_BLOCK_HEIGHT", Some("not-a-number"))],
        || {
            let error = run_live_s06_proof_verification_probe()
                .expect_err("invalid block height env value should fail");
            assert!(
                error.contains("invalid block height env value"),
                "probe error should reflect parse failure: {error}",
            );
        },
    );
}

#[test]
fn unit_run_live_s06_proof_verification_probe_accepts_final_verified_receipt() {
    with_env_vars(
        &[
            ("KAMN_ENDPOINT", Some("http://localhost:8080")),
            ("KAMN_KOLME_ENDPOINT", Some("http://localhost:3000")),
            ("KAMN_AGENT_NAME", Some("sdk-driver-test")),
            ("KAMN_E2E_S06_PROOF_FINALITY", Some("final")),
        ],
        || {
            run_live_s06_proof_verification_probe()
                .expect("final verified proof probe should succeed");
        },
    );
}
