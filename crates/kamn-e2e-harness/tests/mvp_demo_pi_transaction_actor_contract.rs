use kamn_e2e_harness::verify_pi_transaction_actor_paths;

#[path = "support/pi_transaction_actor_fixture.rs"]
mod pi_transaction_actor_fixture;
use pi_transaction_actor_fixture::{sha, ActorFixture, Overrides};

const SERVICE_AUTHORITY_ERROR: &str = "PI_SERVICE_AUTHORITY_MISMATCH";
const TRANSPORT_PROVENANCE_ERROR: &str = "PI_TRANSPORT_PROVENANCE_INVALID";

#[test]
fn spec_c01_rust_verifier_accepts_three_runtime_bound_pi_actors() {
    let fixture = ActorFixture::new();
    fixture.write_bound_v2_all();

    let summary = verify_pi_transaction_actor_paths(&fixture.paths())
        .expect("three independent actor artifacts should verify");
    assert!(summary.contains(r#""receipt_chain_commitment":"sha256:"#));
    assert!(summary.contains(r#""public_commitment":"sha256:"#));
}
#[test]
fn spec_c02_rust_verifier_rejects_reused_process_and_identity() {
    for (overrides, expected) in [
        (
            Overrides {
                agent_c_pid: 101,
                ..Overrides::default()
            },
            TRANSPORT_PROVENANCE_ERROR,
        ),
        (
            Overrides {
                agent_c_did: "kamn:did:a",
                ..Overrides::default()
            },
            SERVICE_AUTHORITY_ERROR,
        ),
    ] {
        let fixture = ActorFixture::new();
        write_bound_actor_fixture(&fixture, overrides);
        let error = verify_pi_transaction_actor_paths(&fixture.paths())
            .expect_err("reused actor identity must fail");
        assert_eq!(error, expected);
    }
}

#[test]
fn spec_c03_rust_verifier_rejects_runtime_privacy_and_shared_fact_drift() {
    for overrides in [
        Overrides {
            agent_c_projection: sha('f'),
            ..Overrides::default()
        },
        Overrides {
            agent_c_private: Some(sha('e')),
            ..Overrides::default()
        },
        Overrides {
            agent_b_escrow: "escrow-other",
            ..Overrides::default()
        },
        Overrides {
            agent_a_handoff_authorized: true,
            ..Overrides::default()
        },
    ] {
        let fixture = ActorFixture::new();
        write_bound_actor_fixture(&fixture, overrides);
        let error = verify_pi_transaction_actor_paths(&fixture.paths())
            .expect_err("tampered actor evidence must fail");
        assert_eq!(error, SERVICE_AUTHORITY_ERROR);
    }
}

#[test]
fn spec_c04_rust_verifier_rejects_missing_operation_and_type_confusion() {
    for overrides in [
        Overrides {
            agent_a_include_release: false,
            ..Overrides::default()
        },
        Overrides {
            agent_a_handoff_as_string: true,
            ..Overrides::default()
        },
    ] {
        let fixture = ActorFixture::new();
        write_bound_actor_fixture(&fixture, overrides);
        let error = verify_pi_transaction_actor_paths(&fixture.paths())
            .expect_err("incomplete or type-confused actor evidence must fail");
        assert!(error.contains("PI_"), "unexpected error: {error}");
    }
}

fn write_bound_actor_fixture(fixture: &ActorFixture, overrides: Overrides) {
    fixture.write_bound_v2(overrides);
}
