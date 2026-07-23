use kamn_e2e_harness::build_runtime_receipt_chain_from_actor_paths;
use serde_json::Value;

#[path = "support/pi_transaction_actor_fixture.rs"]
mod pi_transaction_actor_fixture;
use pi_transaction_actor_fixture::{sha, ActorFixture, Overrides};

#[test]
fn spec_c01_builds_service_authority_summary_from_v2_receipts() {
    let fixture = ActorFixture::new();
    fixture.write_all(Overrides::default());

    let raw = build_runtime_receipt_chain_from_actor_paths(&fixture.paths())
        .expect("valid service receipts should build an authority summary");
    let chain: Value = serde_json::from_str(&raw).expect("strict chain JSON");

    assert_eq!(chain["schema_version"], "kamn.service.receipt-chain.v1");
    assert_eq!(actions(&chain), expected_actions());
    assert_eq!(chain["receipt_chain_commitment"], sha('c'));
    assert_sha256(&chain["service_receipt_commitment"]);
    assert!(!raw.contains("transport_response_digests"));
    assert!(!raw.contains("participant_role"));
}

fn assert_sha256(value: &Value) {
    let value = value.as_str().expect("receipt projection commitment");
    assert_eq!(value.len(), 71);
    assert!(value.starts_with("sha256:"));
}

#[test]
fn spec_c02_rejects_missing_duplicate_or_failed_authority() {
    for overrides in [
        Overrides {
            agent_a_include_release: false,
            ..Overrides::default()
        },
        Overrides {
            agent_a_duplicate_fund: true,
            ..Overrides::default()
        },
        Overrides {
            agent_a_release_error: true,
            ..Overrides::default()
        },
    ] {
        assert_authority_error(overrides);
    }
}

#[test]
fn spec_c03_rejects_fact_privacy_and_receipt_drift() {
    for overrides in [
        Overrides {
            agent_b_escrow: "escrow-other",
            ..Overrides::default()
        },
        Overrides {
            agent_c_private: Some(sha('e')),
            ..Overrides::default()
        },
        Overrides {
            agent_a_public_fact_drift: true,
            ..Overrides::default()
        },
    ] {
        assert_authority_error(overrides);
    }
}

#[test]
fn spec_c04_rejects_actor_local_mutation_reordering() {
    let fixture = ActorFixture::new();
    fixture.write_all(Overrides::default());
    fixture.reorder_agent_a_mutations();

    let error = build_runtime_receipt_chain_from_actor_paths(&fixture.paths())
        .expect_err("reordered service authority must fail");
    assert_eq!(error, "PI_SERVICE_AUTHORITY_MISMATCH");
}

fn assert_authority_error(overrides: Overrides) {
    let fixture = ActorFixture::new();
    fixture.write_all(overrides);
    let error = build_runtime_receipt_chain_from_actor_paths(&fixture.paths())
        .expect_err("tampered service authority must fail");
    assert!(
        error == "PI_SERVICE_AUTHORITY_MISMATCH"
            || error == "PI_TRANSACTION_FACT_MISMATCH"
            || error == "PI_VERIFIER_PRIVATE_LEAK",
        "unexpected error: {error}"
    );
}

fn actions(chain: &Value) -> Vec<&str> {
    chain["actor_receipts"]
        .as_array()
        .expect("receipts")
        .iter()
        .map(|receipt| receipt["action"].as_str().expect("action"))
        .collect()
}

fn expected_actions() -> Vec<&'static str> {
    vec![
        "task:create",
        "task:accept",
        "escrow:fund",
        "task:complete",
        "escrow:release-authorize",
    ]
}
