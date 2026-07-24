use super::super::*;
use super::support::{
    assert_non_placeholder_bridge_evidence, assert_non_placeholder_bridge_payload,
    set_state_file_env, submit_and_restart_live_bridge, unique_named_state_file,
    LiveBridgeScenario, LiveBridgeTestOverride, LiveBridgeTransactionEnv,
};

#[test]
fn persists_finalized_transaction_receipt() {
    let scenario = LiveBridgeScenario::new(
        "bridge-finalized",
        "kamn:did:agent:bridge-finalized",
        "127.0.0.1:34117",
        LiveBridgeTestOverride::Success,
    );
    let bridge_id = scenario.submit(201, r#"{"source_message_id":"msg-bridge-live-source"}"#);
    let forwarded = scenario.forward(202, bridge_id.as_str());
    assert_non_placeholder_bridge_evidence(bridge_id.as_str(), &forwarded);
    let state = scenario.state();
    let receipt = &state["bridges"][&bridge_id]["bridge_receipt"];
    assert_eq!(forwarded["bridge_status"], "finalized");
    assert_eq!(
        receipt["transaction_signature"],
        forwarded["transaction_signature"]
    );
    assert_eq!(receipt["finalized_slot"], 42);
    assert_eq!(receipt["commitment"], "finalized");
    assert_eq!(forwarded["bridge_receipt"], *receipt);
    assert_eq!(receipt["payload_hash"].as_str().map(str::len), Some(71));
    assert!(receipt["receipt_digest"]
        .as_str()
        .is_some_and(|digest| digest.starts_with("sha256:")));
}

#[test]
fn integration_service_api_endpoint_live_bridge_forward_evidence_survives_restart() {
    let _env = acquire_service_api_test_env();
    let _transaction_env = LiveBridgeTransactionEnv::enable("bridge-restart-keypair");
    let _override = crate::service_api_endpoint::set_test_live_solana_settlement_override(true);
    let state_file = unique_named_state_file("kamn-node-service-api-bridge-live-restart");
    let _state_file_guard = set_state_file_env(state_file.as_path());
    let caller_did = "kamn:did:agent:test-client-bridge-live-restart";
    let (bridge_id, queried) = submit_and_restart_live_bridge(
        caller_did,
        ("127.0.0.1:34119", "127.0.0.1:34120"),
        (301, 302, 303),
        r#"{"source_message_id":"msg-bridge-live-restart-source"}"#,
    );
    assert_non_placeholder_bridge_payload(bridge_id.as_str(), &queried);
    let _ = fs::remove_file(state_file);
}

#[test]
fn rejects_mismatched_finality_evidence() {
    let scenario = LiveBridgeScenario::new(
        "bridge-mismatch",
        "kamn:did:agent:bridge-mismatch",
        "127.0.0.1:34121",
        LiveBridgeTestOverride::Success,
    );
    crate::service_api_endpoint::set_test_live_solana_settlement_evidence_mismatch();
    let bridge_id = scenario.submit(401, "{}");
    let response = scenario.forward_response(402, bridge_id.as_str());
    let state = scenario.state();
    assert!(
        response.contains("BRIDGE_FINALITY_EVIDENCE_INVALID"),
        "{response}"
    );
    assert!(state["bridges"][&bridge_id]["bridge_receipt"].is_null());
    assert_ne!(state["bridges"][&bridge_id]["bridge_status"], "finalized");
}

#[test]
fn reconciles_before_resubmit() {
    let scenario = LiveBridgeScenario::new(
        "bridge-reconcile",
        "kamn:did:agent:bridge-reconcile",
        "127.0.0.1:34122",
        LiveBridgeTestOverride::Ambiguous,
    );
    let bridge_id = scenario.submit(501, "{}");
    let first = scenario.forward_response(502, bridge_id.as_str());
    crate::service_api_endpoint::set_test_live_solana_settlement_reconcile_confirmed();
    let second = scenario.forward(503, bridge_id.as_str());
    assert!(first.contains("BRIDGE_RECONCILIATION_REQUIRED"), "{first}");
    assert_eq!(second["bridge_status"], "finalized");
    assert_eq!(
        crate::service_api_endpoint::test_live_solana_settlement_submission_count(),
        1
    );
}

#[test]
fn observes_exactly_one_transfer() {
    let scenario = LiveBridgeScenario::new(
        "bridge-once",
        "kamn:did:agent:bridge-once",
        "127.0.0.1:34123",
        LiveBridgeTestOverride::Success,
    );
    let bridge_id = scenario.submit(601, "{}");
    let first = scenario.forward(602, bridge_id.as_str());
    let second = scenario.forward(603, bridge_id.as_str());
    assert_eq!(
        first["transaction_signature"],
        second["transaction_signature"]
    );
    assert_eq!(first["receipt_digest"], second["receipt_digest"]);
    assert_eq!(
        crate::service_api_endpoint::test_live_solana_settlement_submission_count(),
        1
    );
}
