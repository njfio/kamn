use kamn_e2e_harness::verify_pi_transaction_actor_paths;

#[path = "support/pi_transaction_actor_fixture.rs"]
mod pi_transaction_actor_fixture;
use pi_transaction_actor_fixture::{sha, ActorFixture, Overrides};
#[test]
fn spec_c01_rust_verifier_accepts_three_runtime_bound_pi_actors() {
    let fixture = ActorFixture::new();
    fixture.write_all(Overrides::default());

    let summary = verify_pi_transaction_actor_paths(&fixture.paths())
        .expect("three independent actor artifacts should verify");
    assert!(summary.contains("task-live-7099"));
    assert!(summary.contains("escrow-live-7099"));
    assert!(summary.contains("devnet-signature-7099"));
}
#[test]
fn spec_c02_rust_verifier_rejects_reused_process_and_identity() {
    for overrides in [
        Overrides {
            agent_c_pid: 101,
            ..Overrides::default()
        },
        Overrides {
            agent_c_did: "kamn:did:a",
            ..Overrides::default()
        },
    ] {
        let fixture = ActorFixture::new();
        fixture.write_all(overrides);
        let error = verify_pi_transaction_actor_paths(&fixture.paths())
            .expect_err("reused actor identity must fail");
        assert!(error.contains("PI_ACTOR_"));
    }
}

#[test]
fn spec_c03_rust_verifier_rejects_runtime_privacy_and_shared_fact_drift() {
    for (overrides, code) in [
        (
            Overrides {
                agent_c_projection: sha('f'),
                ..Overrides::default()
            },
            "PI_RUNTIME_RECEIPT_MISMATCH",
        ),
        (
            Overrides {
                agent_c_private: Some(sha('e')),
                ..Overrides::default()
            },
            "PI_VERIFIER_PRIVATE_LEAK",
        ),
        (
            Overrides {
                agent_b_escrow: "escrow-other",
                ..Overrides::default()
            },
            "PI_TRANSACTION_FACT_MISMATCH",
        ),
        (
            Overrides {
                agent_a_handoff_authorized: true,
                ..Overrides::default()
            },
            "PI_HANDOFF_AUTHORIZATION_FORBIDDEN",
        ),
    ] {
        let fixture = ActorFixture::new();
        fixture.write_all(overrides);
        let error = verify_pi_transaction_actor_paths(&fixture.paths())
            .expect_err("tampered actor evidence must fail");
        assert!(error.contains(code), "unexpected error: {error}");
    }
}

#[test]
fn spec_c04_rust_verifier_rejects_missing_operation_and_type_confusion() {
    for overrides in [
        Overrides { agent_a_include_release: false, ..Overrides::default() },
        Overrides { agent_a_handoff_as_string: true, ..Overrides::default() },
    ] {
        let fixture = ActorFixture::new();
        fixture.write_all(overrides);
        let error = verify_pi_transaction_actor_paths(&fixture.paths())
            .expect_err("incomplete or type-confused actor evidence must fail");
        assert!(error.contains("PI_"), "unexpected error: {error}");
    }
}
