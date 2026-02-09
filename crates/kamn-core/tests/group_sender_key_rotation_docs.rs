const DOC: &str = include_str!("../../../docs/foundation/group-sender-key-rotation.md");

#[test]
fn doc_contains_group_sender_replay_ratchet_contract_scope() {
    assert!(DOC.contains("# Group Sender-Key Replay and Ratchet Contract Rules"));
    assert!(DOC.contains("run_group_sender_replay_ratchet_contract_lane.sh"));
    assert!(DOC.contains("kamn.group-sender.replay-ratchet-evidence.v1"));
}

#[test]
fn regression_requires_stale_generation_replay_marker() {
    // Regression: #932
    assert!(DOC
        .contains("stale-generation and nonce replay payloads are rejected (`Regression: #932`)"));
    assert!(DOC.contains("check_group_sender_replay_ratchet_policy.sh"));
}
