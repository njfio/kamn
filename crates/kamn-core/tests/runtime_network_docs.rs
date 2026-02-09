const DOC: &str = include_str!("../../../docs/foundation/runtime-network.md");

#[test]
fn doc_contains_runtime_network_scope_and_models() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("## Node CLI Recovery-Check Mapping"));
    assert!(DOC.contains("## Node CLI Daemon Lifecycle Mapping"));
    assert!(DOC.contains("## Bridge Quorum Runtime Mapping"));
    assert!(DOC.contains("## Snapshot Persistence and Restore Contract Rules"));
    assert!(DOC.contains("## Deterministic Fault Simulation Harness Rules"));
    assert!(DOC.contains("PeerLifecycle"));
    assert!(DOC.contains("AuthenticatedPeerFrame"));
    assert!(DOC.contains("PeerFrameAuthenticator"));
    assert!(DOC.contains("RuntimeBackpressurePolicy"));
    assert!(DOC.contains("DeterministicBackpressureController"));
    assert!(DOC.contains("BoundedRuntimeQueue<T>"));
    assert!(DOC.contains("RuntimeLifecycleError"));
    assert!(DOC.contains("NetworkFaultSimulationInput"));
    assert!(DOC.contains("DeterministicNetworkFaultSimulator"));
    assert!(DOC.contains("RuntimeSnapshot"));
    assert!(DOC.contains("SnapshotRestoreGuard"));
    assert!(DOC.contains("SnapshotStoreError"));
    assert!(DOC.contains("simulate_daemon_network_fault(...)"));
}

#[test]
fn doc_contains_peer_lifecycle_and_queue_rules() {
    assert!(DOC.contains("## Peer Lifecycle Rules"));
    assert!(DOC.contains("## Authenticated Peer Transport Framing Rules"));
    assert!(DOC.contains("## Deterministic Input Mutation Fail-Closed Rules"));
    assert!(DOC.contains("## Queue Guard Rules"));
    assert!(DOC.contains("## Deterministic Backpressure and Stale-Peer Queue Rules"));
    assert!(DOC.contains("## Scheduler Determinism Rules"));
    assert!(DOC.contains("## Recovery and Rejoin Guard Rules"));
    assert!(DOC.contains("version/hash/cursor"));
    assert!(DOC.contains("<state-version>|<state-hash>|<cursor>"));
    assert!(DOC.contains("`--rejoin-attempt <node-id|state-version|state-hash|resume-token>`"));
    assert!(DOC.contains("ConfigError::RuntimeRecovery"));
    assert!(DOC.contains("ConfigError::RuntimeDaemonLifecycle"));
    assert!(DOC.contains("Overflow does not evict existing entries"));
    assert!(DOC.contains("Empty peer IDs are rejected"));
    assert!(DOC.contains(
        "frame|<frame_id>|<sender_peer_did>|<recipient_peer_did>|<nonce>|<payload>|<signature>"
    ));
    assert!(DOC.contains("monotonic sender nonce progression"));
    assert!(DOC.contains("queue depth less than or equal to queue capacity"));
    assert!(DOC.contains("PurgeStalePeerQueue"));
    assert!(DOC.contains("malformed payload shape/identity cases"));
    assert!(DOC.contains("truncated scalar/list payload cases"));
    assert!(DOC.contains("tampered proof-binding cases"));
    assert!(DOC.contains("normalization drift cases"));
    assert!(DOC.contains("encoding/character drift cases"));
    assert!(DOC.contains("method mismatch prefix cases"));
    assert!(DOC.contains("run_input_mutation_contract_lane.sh"));
}

#[test]
fn doc_contains_network_fault_simulation_rules() {
    assert!(DOC.contains("queue_overflow_attempts"));
    assert!(DOC.contains("watchdog-compatible delivery/peer sample values"));
    assert!(
        DOC.contains("daemon network-fault simulation with queue-overflow/degradation reporting")
    );
}

#[test]
fn doc_contains_recovery_check_cli_command_example() {
    assert!(DOC.contains("`kamn-node --role processor --runtime-mode recovery-check`"));
}

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core runtime::tests::"));
    assert!(DOC.contains(
        "cargo test -p kamn-core runtime::tests::functional_runtime_backpressure_classifies_queue_saturation"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core runtime::tests::regression_runtime_backpressure_rejects_capacity_overflow_sample"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core runtime::tests::functional_authenticated_peer_frame_roundtrips_wire_and_signature"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core runtime::tests::regression_forged_or_unauthorized_peer_frame_is_rejected"
    ));
    assert!(DOC.contains("cargo test -p kamn-core network_fault_simulation"));
    assert!(DOC.contains("cargo test -p kamn-core snapshot_store"));
    assert!(DOC.contains("cargo test -p kamn-core --test bridge_quorum_runtime_docs"));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test message_envelope_fuzz_smoke functional_envelope_mutation_suite_covers_malformed_truncated_and_tampered_classes -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test did_fuzz_smoke functional_did_mutation_suite_covers_normalization_encoding_and_method_mismatch_classes -- --exact"
    ));
    assert!(DOC.contains("bash scripts/runtime/run_input_mutation_contract_lane.sh"));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test task_state_machine task_lifecycle_property_generated_sequences_preserve_transition_contracts -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test escrow_lifecycle escrow_property_generated_action_sequences_preserve_amount_and_status_invariants -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test runtime_peer_lifecycle peer_lifecycle_property_generated_event_sequences_match_transition_contract -- --exact"
    ));
    assert!(DOC.contains("bash scripts/runtime/run_lifecycle_property_contract_lane.sh"));
    assert!(DOC.contains("cargo test -p kamn-node --test node_runtime_cli_docs"));
    assert!(DOC.contains("bash scripts/runtime/run_runtime_snapshot_contract_lane.sh"));
    assert!(DOC.contains(
        "cargo test -p kamn-node regression_runtime_daemon_rejects_invalid_lifecycle_transition"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core performance_network_fault_simulation_chaos_lane_stress -- --ignored"
    ));
    assert!(DOC.contains("bash scripts/runtime/run_runtime_snapshot_deep_lane.sh"));
    assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
}

#[test]
fn regression_requires_rejoin_and_overflow_rejection_rules() {
    // Regression: #324
    assert!(DOC.contains("rejoin without disconnect is rejected"));
    assert!(DOC.contains("queue overflow rejects new event"));
    assert!(DOC.contains("duplicate candidate ID is rejected"));
    assert!(DOC.contains("stale state hash is rejected"));
    assert!(DOC.contains("rejoin replay token is rejected"));
    assert!(DOC.contains("rejoin state hash mismatch is rejected"));
    assert!(DOC.contains("CLI recovery-check replay/version/hash mismatch rejection"));
    assert!(DOC.contains("daemon lifecycle invalid transition rejection"));
    assert!(DOC.contains("listener attestation replay rejection (`Regression: #371`)"));
    assert!(DOC.contains("outbound under-quorum rejection (`Regression: #372`)"));
    assert!(DOC.contains(
        "network fault simulation censorship critical-boundary guard (`Regression: #618`)"
    ));
    assert!(DOC.contains("forged or unauthorized peer frame rejection (`Regression: #618`)"));
    assert!(DOC.contains("replayed peer-frame nonce rejection (`Regression: #618`)"));
    assert!(DOC.contains("queue depth above capacity is rejected (`Regression: #618`)"));
    assert!(DOC.contains(
        "stale disconnected peer queue purge decision remains deterministic (`Regression: #618`)"
    ));
    assert!(DOC.contains("snapshot stale metadata rejection (`Regression: #617`)"));
    assert!(DOC.contains("snapshot restore cursor mismatch rejection (`Regression: #617`)"));
    assert!(DOC.contains(
        "deterministic envelope/DID mutation fail-closed reasons remain stable (`Regression: #843`)"
    ));
    assert!(DOC.contains(
        "task/escrow/peer lifecycle generated invariant lanes remain deterministic (`Regression: #842`)"
    ));
}

#[test]
fn doc_contains_mutation_fail_closed_contract_rules() {
    assert!(DOC.contains("All mutation corpus entries must fail closed"));
    assert!(DOC.contains("typed errors and stable reason strings"));
    assert!(DOC.contains("Regression: #843"));
}
