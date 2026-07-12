use kamn_e2e_harness::build_runtime_receipt_chain_from_actor_paths;
use serde_json::Value;

#[path = "support/pi_transaction_actor_fixture.rs"]
mod pi_transaction_actor_fixture;
use pi_transaction_actor_fixture::{sha, ActorFixture, Overrides};

#[test]
fn spec_c01_builds_ordered_chain_from_exact_runtime_receipts() {
    let fixture = ActorFixture::new();
    fixture.write_all(Overrides::default());

    let raw = build_runtime_receipt_chain_from_actor_paths(&fixture.paths())
        .expect("valid actor receipts should build a runtime chain");
    let chain: Value = serde_json::from_str(&raw).expect("strict chain JSON");

    assert_eq!(chain["schema_version"], "kamn.mvp.runtime-receipt-chain.v1");
    assert_eq!(actions(&chain), expected_actions());
    assert!(steps(&chain).iter().all(has_runtime_source));
    assert!(!raw.contains("agent_a_registered"));
    assert!(!raw.contains("private_receipt_digest"));
}

#[test]
fn spec_c02_missing_runtime_step_fails_with_chain_reason() {
    let fixture = ActorFixture::new();
    fixture.write_all(Overrides {
        agent_a_include_release: false,
        ..Overrides::default()
    });

    assert_chain_error(&fixture, "RUNTIME_RECEIPT_CHAIN_STEP_MISSING");
}

#[test]
fn spec_c03_shared_fact_drift_fails_with_chain_reason() {
    let fixture = ActorFixture::new();
    fixture.write_all(Overrides {
        agent_b_escrow: "escrow-other",
        ..Overrides::default()
    });

    assert_chain_error(&fixture, "RUNTIME_RECEIPT_CHAIN_FACT_MISMATCH");
}

#[test]
fn spec_c04_verifier_private_data_fails_with_chain_reason() {
    let fixture = ActorFixture::new();
    fixture.write_all(Overrides {
        agent_c_private: Some(sha('e')),
        ..Overrides::default()
    });

    assert_chain_error(&fixture, "RUNTIME_RECEIPT_CHAIN_VERIFIER_PRIVATE_LEAK");
}

#[test]
fn spec_c05_duplicate_successful_mutation_fails_closed() {
    let fixture = ActorFixture::new();
    fixture.write_all(Overrides {
        agent_a_duplicate_fund: true,
        ..Overrides::default()
    });

    assert_chain_error(&fixture, "RUNTIME_RECEIPT_CHAIN_STEP_DUPLICATED");
}

#[test]
fn spec_c06_failed_release_cannot_satisfy_success_step() {
    let fixture = ActorFixture::new();
    fixture.write_all(Overrides {
        agent_a_release_error: true,
        ..Overrides::default()
    });

    assert_chain_error(&fixture, "RUNTIME_RECEIPT_CHAIN_OUTCOME_INVALID");
}

#[test]
fn spec_c07_receipt_digest_drift_fails_closed() {
    let fixture = ActorFixture::new();
    fixture.write_all(Overrides {
        agent_a_receipt_digest_mismatch: true,
        ..Overrides::default()
    });

    assert_chain_error(&fixture, "RUNTIME_RECEIPT_CHAIN_DIGEST_MISMATCH");
}

#[test]
fn spec_c08_public_receipt_fact_drift_fails_closed() {
    let fixture = ActorFixture::new();
    fixture.write_all(Overrides {
        agent_a_public_fact_drift: true,
        ..Overrides::default()
    });

    assert_chain_error(&fixture, "RUNTIME_RECEIPT_CHAIN_FACT_MISMATCH");
}

#[test]
fn spec_c09_actor_local_mutation_order_fails_closed() {
    let fixture = ActorFixture::new();
    fixture.write_all(Overrides::default());
    fixture.reorder_agent_a_mutations();

    assert_chain_error(&fixture, "RUNTIME_RECEIPT_CHAIN_ORDER_INVALID");
}

fn assert_chain_error(fixture: &ActorFixture, code: &str) {
    let error = build_runtime_receipt_chain_from_actor_paths(&fixture.paths())
        .expect_err("tampered runtime evidence must fail");
    assert_eq!(error, code);
}

fn actions(chain: &Value) -> Vec<&str> {
    steps(chain)
        .iter()
        .map(|step| step["action"].as_str().expect("action"))
        .collect()
}

fn steps(chain: &Value) -> &[Value] {
    chain["steps"].as_array().expect("steps")
}

fn has_runtime_source(step: &Value) -> bool {
    step["request_id"].as_u64().is_some()
        && step["response_digest"]
            .as_str()
            .is_some_and(|digest| digest.starts_with("sha256:"))
        && step["outcome"] == "success"
}

fn expected_actions() -> Vec<&'static str> {
    vec![
        "register",
        "register",
        "register",
        "create_task",
        "accept_task",
        "fund_escrow",
        "complete_task",
        "release_escrow",
        "query_participant_task_projection",
        "query_participant_task_projection",
        "query_verifier_task_projection",
    ]
}
