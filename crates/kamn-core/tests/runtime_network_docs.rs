const DOC: &str = include_str!("../../../docs/foundation/runtime-network.md");

#[test]
fn doc_contains_runtime_network_scope_and_models() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("crates/kamn-core/src/runtime_peer_coordination.rs"));
    assert!(DOC.contains("crates/kamn-core/src/runtime_phase_coordination.rs"));
    assert!(DOC.contains("crates/kamn-core/src/runtime_transport_coordination.rs"));
    assert!(DOC.contains("crates/kamn-core/src/runtime_snapshot_store.rs"));
    assert!(DOC.contains("crates/kamn-core/src/runtime_tests_snapshot_store.rs"));
    assert!(DOC.contains("crates/kamn-core/src/runtime_recovery_guard.rs"));
    assert!(DOC.contains("## Node CLI Recovery-Check Mapping"));
    assert!(DOC.contains("## Node CLI Daemon Lifecycle Mapping"));
    assert!(DOC.contains("## Bridge Quorum Runtime Mapping"));
    assert!(DOC.contains("## Snapshot Persistence and Restore Contract Rules"));
    assert!(DOC.contains("## Deterministic Fault Simulation Harness Rules"));
    assert!(DOC.contains("## Runtime Test Module Ownership Rules"));
    assert!(DOC.contains("## Kolme Notifications Websocket Consumer Contract Rules"));
    assert!(DOC.contains("## Kolme Block Fallback Finality Reconciliation Contract Rules"));
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
    assert!(DOC.contains("## Deterministic Concurrency Harness Rules"));
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
    assert!(DOC.contains("run_input_mutation_coverage_guided_contract_lane.sh"));
    assert!(DOC.contains("run_input_mutation_coverage_guided_deep_lane.sh"));
    assert!(DOC.contains("runtime_input_mutation_coverage_guided_deep=skipped_local_only"));
    assert!(DOC.contains("run_processor_proof_admission_contract_lane.sh"));
    assert!(DOC.contains("single-winner task accept races"));
    assert!(DOC.contains("deterministic peer lifecycle phase summaries"));
    assert!(DOC.contains("run_concurrency_state_mutation_contract_lane.sh"));
    assert!(DOC.contains("run_concurrency_state_mutation_deep_lane.sh"));
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
    assert!(DOC.contains("cargo test -p kamn-core --test kolme_runtime_commit_notifications"));
    assert!(DOC.contains("cargo test -p kamn-core --test kolme_runtime_commit_block_fallback"));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test message_envelope_fuzz_smoke functional_envelope_mutation_suite_covers_malformed_truncated_and_tampered_classes -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test did_fuzz_smoke functional_did_mutation_suite_covers_normalization_encoding_and_method_mismatch_classes -- --exact"
    ));
    assert!(DOC.contains("bash scripts/runtime/run_input_mutation_contract_lane.sh"));
    assert!(DOC.contains(
        "bash scripts/runtime/run_input_mutation_coverage_guided_contract_lane.sh --output-json /tmp/input-mutation-coverage-guided-contract-report.json"
    ));
    assert!(DOC.contains("bash scripts/runtime/run_processor_proof_admission_contract_lane.sh"));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test concurrency_state_mutation functional_task_accept_concurrency_replay_fixture_preserves_invariants -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test concurrency_state_mutation integration_peer_lifecycle_concurrency_replay_is_deterministic_across_rounds -- --exact"
    ));
    assert!(DOC.contains("bash scripts/runtime/run_concurrency_state_mutation_contract_lane.sh"));
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
    assert!(DOC.contains(
        "bash scripts/runtime/run_invariant_fuzz_concurrency_contract_lane.sh --output-json /tmp/invariant-fuzz-concurrency-contract-report.json"
    ));
    assert!(DOC.contains(
        "bash scripts/runtime/check_invariant_fuzz_concurrency_policy.sh --report-file /tmp/invariant-fuzz-concurrency-contract-report.json"
    ));
    assert!(DOC.contains("cargo test -p kamn-node --test node_runtime_cli_docs"));
    assert!(DOC.contains("bash scripts/runtime/run_runtime_snapshot_contract_lane.sh"));
    assert!(DOC.contains("bash scripts/kolme/run_notifications_consumer_contract_lane.sh"));
    assert!(DOC.contains("bash scripts/kolme/run_block_fallback_reconciliation_contract_lane.sh"));
    assert!(DOC.contains(
        "cargo test -p kamn-node regression_runtime_daemon_rejects_invalid_lifecycle_transition"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core performance_network_fault_simulation_chaos_lane_stress -- --ignored"
    ));
    assert!(DOC.contains("bash scripts/runtime/run_concurrency_state_mutation_deep_lane.sh"));
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
    assert!(
        DOC.contains("concurrent accept races never allow multiple winners (`Regression: #844`)")
    );
    assert!(DOC.contains(
        "deterministic envelope/DID mutation fail-closed reasons remain stable (`Regression: #843`)"
    ));
    assert!(DOC.contains(
        "deep coverage-guided parser fuzz remains local-only and excluded from `ci-fast-gate` (`Regression: #2693`)"
    ));
    assert!(DOC.contains(
        "task/escrow/peer lifecycle generated invariant lanes remain deterministic (`Regression: #842`)"
    ));
    assert!(DOC.contains(
        "invariant/fuzz/concurrency combined lane reason-code policy remains fail-closed (`Regression: #897`)"
    ));
    assert!(DOC.contains(
        "notifications websocket variant/reconnect fail-closed contract remains stable (`Regression: #1463`)"
    ));
    assert!(DOC.contains(
        "block fallback stale-window/height-mismatch fail-closed contract remains stable (`Regression: #1464`)"
    ));
    assert!(DOC.contains(
        "processor proof admission message/commitment/replay/format guards remain fail-closed (`Regression: #995`)"
    ));
}

#[test]
fn doc_contains_mutation_fail_closed_contract_rules() {
    assert!(DOC.contains("All mutation corpus entries must fail closed"));
    assert!(DOC.contains("typed errors and stable reason strings"));
    assert!(DOC.contains("--target envelope"));
    assert!(DOC.contains("--target did"));
    assert!(DOC.contains("kamn.runtime.input-mutation-replay-metadata.v1"));
    assert!(DOC.contains("input_mutation_envelope_seed:v1"));
    assert!(DOC.contains("input_mutation_did_seed:v1"));
    assert!(DOC.contains("kamn.runtime.input-mutation-coverage-guided-replay-metadata.v1"));
    assert!(DOC.contains("input_mutation_coverage_guided_replay:v1"));
    assert!(DOC.contains("minimal_failing_seed_prefix"));
    assert!(DOC.contains("Regression: #843"));
}

#[test]
fn doc_contains_concurrency_harness_contract_rules() {
    assert!(DOC.contains("single-winner task accept races under parallel contender workloads"));
    assert!(DOC.contains("deterministic duplicate-submit rejection outcomes"));
    assert!(DOC.contains("deterministic peer lifecycle phase summaries"));
    assert!(DOC.contains("run_concurrency_state_mutation_contract_lane.sh"));
    assert!(DOC.contains("run_concurrency_state_mutation_deep_lane.sh"));
    assert!(DOC.contains("Regression: #844"));
}

#[test]
fn doc_contains_invariant_fuzz_concurrency_contract_rules() {
    assert!(DOC.contains("run_invariant_fuzz_concurrency_contract_lane.sh"));
    assert!(DOC.contains("check_invariant_fuzz_concurrency_policy.sh"));
    assert!(DOC.contains("kamn.runtime.invariant-fuzz-concurrency-contract-report.v1"));
}

#[test]
fn doc_contains_localhost_signed_integration_evidence_key_contract_rules() {
    assert!(DOC.contains("## Localhost Signed Integration Evidence Key Contract Rules"));
    assert!(DOC.contains("run_localhost_signed_integration_harness.sh"));
    assert!(DOC.contains("run_localhost_signed_integration_contract_lane.sh"));
    assert!(DOC.contains("check_localhost_signed_integration_evidence_policy.sh"));
    assert!(DOC.contains("localhost_signed_integration_contract:v1"));
    assert!(DOC.contains("localhost_signed_integration:success:v1"));
    assert!(DOC.contains("localhost_signed_integration:signature-mismatch:v1"));
    assert!(DOC.contains("localhost_signed_integration:timeout:v1"));
    assert!(DOC.contains("Regression: #899"));
}

#[test]
fn doc_contains_live_transport_replay_tamper_evidence_contract_rules() {
    assert!(DOC.contains("## Live Transport Replay/Tamper Evidence Contract Rules"));
    assert!(DOC.contains("generate_live_transport_replay_tamper_evidence_bundle.sh"));
    assert!(DOC.contains("check_live_transport_replay_tamper_policy.sh"));
    assert!(DOC.contains("run_live_transport_replay_tamper_contract_lane.sh"));
    assert!(DOC.contains("run_live_transport_replay_tamper_fast_lane.sh"));
    assert!(DOC.contains("run_live_transport_replay_tamper_deep_lane.sh"));
    assert!(DOC.contains("kamn.sdk.live-transport-replay-tamper-evidence.v1"));
    assert!(DOC.contains("malformed_signature_detected"));
    assert!(DOC.contains("replay_nonce_detected"));
    assert!(DOC.contains("tamper_payload_detected"));
    assert!(DOC.contains("lane_mode=fast"));
    assert!(DOC.contains("lane_mode=deep"));
    assert!(DOC.contains("deep_no_go_status=verified"));
    assert!(DOC.contains("Regression: #1380"));
}

#[test]
fn doc_contains_live_transport_troubleshooting_taxonomy_and_runbook_commands() {
    assert!(DOC.contains("## Live Transport Demo Failure Taxonomy and Troubleshooting"));
    assert!(DOC.contains("signature_mismatch_detected"));
    assert!(DOC.contains("malformed_signature_detected"));
    assert!(DOC.contains("listener_timeout_detected"));
    assert!(DOC.contains("session_expired_detected"));
    assert!(DOC.contains("replay_nonce_detected"));
    assert!(DOC.contains("session_admission_guards_detected"));
    assert!(DOC.contains("tamper_payload_detected"));
    assert!(DOC.contains("ci_fast_gate_failed"));
    assert!(
        DOC.contains("run_localhost_signed_integration_harness.sh --scenario malformed-signature")
    );
    assert!(DOC.contains("run_localhost_signed_integration_harness.sh --scenario session-expired"));
    assert!(DOC.contains("run_localhost_signed_integration_harness.sh --scenario replay-nonce"));
    assert!(DOC.contains(
        "run_live_transport_replay_tamper_fast_lane.sh --output-report /tmp/live-transport-replay-tamper-fast-report.json"
    ));
    assert!(DOC.contains(
        "check_live_transport_replay_tamper_policy.sh --bundle-file /tmp/live-transport-replay-tamper-fast-report.json"
    ));
    assert!(DOC.contains("/tmp/localhost-signed-integration-contract-report.json"));
    assert!(DOC.contains("/tmp/live-transport-replay-tamper-fast-report.json"));
}
