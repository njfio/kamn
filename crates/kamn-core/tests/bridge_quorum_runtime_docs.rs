const DOC: &str = include_str!("../../../docs/foundation/bridge-quorum-runtime.md");

#[test]
fn doc_contains_bridge_quorum_scope_and_models() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("## Listener Quorum Workflow Rules"));
    assert!(DOC.contains("## Approver Quorum Workflow Rules"));
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("listener attestation"));
    assert!(DOC.contains("approver attestation"));
    assert!(DOC.contains("ApproverQuorumEvaluator"));
    assert!(DOC.contains("authorize_daemon_outbound_action"));
}

#[test]
fn doc_contains_bridge_quorum_fast_lane_commands() {
    assert!(DOC.contains("cargo test -p kamn-core --test bridge_quorum_runtime_docs"));
    assert!(DOC.contains("cargo test -p kamn-core --test runtime_network_docs"));
    assert!(DOC.contains("cargo test -p kamn-core approver_quorum"));
    assert!(DOC.contains("bridge_replay_matrix.sh"));
    assert!(DOC.contains("--suites bridge_adapter,discord_bridge"));
    assert!(DOC.contains("run_cross_chain_outbound_intent_contract_lane.sh"));
    assert!(DOC.contains("cargo fmt --check"));
    assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
}

#[test]
fn regression_requires_listener_and_approver_quorum_guard_rules() {
    // Regression: #373
    assert!(DOC.contains("Duplicate listener attestation replay is rejected."));
    assert!(DOC.contains("Replayed or out-of-order listener event sequences are rejected."));
    assert!(DOC.contains("Outbound under-quorum approval sets are rejected."));
    assert!(DOC.contains("Malformed approver attestation payload is rejected."));
    assert!(DOC.contains("idempotency-key and payload-hash consistency across attempts"));
    assert!(DOC.contains("Duplicate outbound replay requests are rejected"));
    assert!(DOC.contains("unauthorized approver signature-failure rejection"));
    assert!(DOC.contains("Regression: #587"));
    assert!(DOC.contains("Regression: #742"));
}
