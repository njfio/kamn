use kamn_e2e_harness::build_runtime_receipt_chain_from_actor_paths;
use serde_json::Value;

#[path = "support/pi_transaction_actor_fixture.rs"]
mod pi_transaction_actor_fixture;
use pi_transaction_actor_fixture::{ActorFixture, Overrides};

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
