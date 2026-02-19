const DOC: &str = include_str!("../../../docs/foundation/runtime-watchdog-attestation.md");

#[test]
fn doc_contains_watchdog_attestation_scope_and_models() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("## Runtime Watchdog Attestation Models"));
    assert!(DOC.contains("crates/kamn-core/src/runtime_state_divergence.rs"));
    assert!(DOC.contains("RuntimeWatchdogAttestation"));
    assert!(DOC.contains("WatchdogSeverity"));
    assert!(DOC.contains("WatchdogIncidentEvidence"));
    assert!(DOC.contains("WatchdogIncidentRecord"));
    assert!(DOC.contains("StateDivergenceWatchInput"));
    assert!(DOC.contains("StateDivergenceEvaluator"));
    assert!(DOC.contains("StateDivergenceReport"));
    assert!(DOC.contains("StateDivergenceError"));
    assert!(DOC.contains("evaluate_daemon_state_divergence"));
    assert!(DOC.contains("WatchdogAnomalyWatchInput"));
    assert!(DOC.contains("WatchdogAnomalyEvaluator"));
    assert!(DOC.contains("WatchdogAnomalyReport"));
    assert!(DOC.contains("WatchdogAnomalyError"));
    assert!(DOC.contains("evaluate_daemon_watchdog_anomaly"));
    assert!(DOC.contains("RuntimeBackpressurePolicy"));
    assert!(DOC.contains("RuntimeBackpressureInput"));
    assert!(DOC.contains("RuntimeBackpressureDecision"));
    assert!(DOC.contains("RuntimeBackpressureAction"));
    assert!(DOC.contains("DeterministicBackpressureController"));
    assert!(DOC.contains("ValidatorProofConsensusEvaluator"));
    assert!(DOC.contains("ValidatorProofConsensusDecision"));
    assert!(DOC.contains("ProofWatchdogProjector"));
    assert!(DOC.contains("ProofWatchdogProjection"));
}

#[test]
fn doc_contains_incident_response_mapping_and_fast_lane() {
    assert!(DOC.contains("## Incident Response Mapping"));
    assert!(DOC.contains("`docs/foundation/upgrade-rollback-runbook.md`"));
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core --test runtime_watchdog_attestation_docs"));
    assert!(DOC.contains("cargo test -p kamn-core --test watchdog_node_docs"));
    assert!(DOC.contains("cargo test -p kamn-core --test upgrade_rollback_runbook_docs"));
    assert!(DOC.contains("cargo test -p kamn-core divergence_watchdog"));
    assert!(DOC.contains("cargo test -p kamn-core watchdog_anomaly"));
    assert!(DOC.contains(
        "cargo test -p kamn-core runtime::tests::functional_runtime_backpressure_classifies_queue_saturation"
    ));
    assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
}

#[test]
fn doc_contains_validator_watchdog_proof_consensus_deep_lane_contract() {
    assert!(DOC.contains("## Validator/Watchdog Proof Consensus Deep-Lane Contract"));
    assert!(DOC.contains("run_watchdog_proof_consensus_contract_lane.sh"));
    assert!(DOC.contains("run_watchdog_proof_consensus_deep_lane.sh"));
    assert!(DOC.contains("generate_watchdog_proof_consensus_evidence_bundle.sh"));
    assert!(DOC.contains("check_watchdog_proof_consensus_policy.sh"));
    assert!(DOC.contains("KAMN_WATCHDOG_PROOF_CONSENSUS_DEEP_CADENCE"));
    assert!(DOC.contains("KAMN_WATCHDOG_PROOF_CONSENSUS_MAX_SECONDS"));
    assert!(DOC.contains("KAMN_WATCHDOG_PROOF_CONSENSUS_DEEP_MAX_SECONDS"));
}

#[test]
fn doc_contains_parser_fuzz_surface_inventory_contracts() {
    assert!(DOC.contains("## Parser Fuzz Surface Inventory"));
    assert!(DOC.contains("message_envelope_fuzz_smoke"));
    assert!(DOC.contains("did_fuzz_smoke"));
    assert!(DOC.contains("run_invariant_fuzz_concurrency_contract_lane.sh"));
    assert!(DOC.contains("check_invariant_fuzz_concurrency_policy.sh"));
    assert!(DOC.contains("kamn.runtime.invariant-fuzz-concurrency-contract-report.v1"));
    assert!(DOC.contains("input_mutation_replay:v1"));
    assert!(DOC.contains("concurrency_mutation_replay:v1"));
}

#[test]
fn doc_contains_task_escrow_proptest_invariant_catalog_contracts() {
    assert!(DOC.contains("## Task/Escrow Proptest Invariant Catalog"));
    assert!(DOC.contains("cargo test -p kamn-core --test task_escrow_proptest_invariants"));
    assert!(DOC.contains("TASK_SEED"));
    assert!(DOC.contains("ESCROW_SEED"));
    assert!(DOC.contains("FileFailurePersistence::SourceParallel(\"proptest-regressions\")"));
    assert!(DOC.contains(
        "crates/kamn-core/proptest-regressions/tests/task_escrow_proptest_invariants.txt"
    ));
    assert!(DOC.contains("bounded case-count envelope: TASK_CASES=192, ESCROW_CASES=192."));
    assert!(DOC.contains("bounded sequence envelope: MAX_SEQUENCE_LEN=32."));
    assert!(DOC.contains(
        "deterministic shrink behavior relies on proptest minimal-counterexample shrinking with fixed seeds."
    ));
    assert!(DOC.contains("accepted transitions must match the legal state graph."));
    assert!(DOC.contains("released + refunded + remaining == total"));
    assert!(DOC.contains("deterministic seed corpus is versioned in git"));
}

#[test]
fn regression_requires_divergence_and_censorship_guard_rules() {
    // Regression: #383
    assert!(DOC
        .contains("state-hash divergence attestation must include expected and observed hashes."));
    assert!(
        DOC.contains("single-recipient deliveries are excluded from censorship classification.")
    );
    assert!(DOC.contains(
        "attestation replay for the same incident fingerprint is rejected (`Regression: #383`)."
    ));
    assert!(DOC.contains("hash mismatch false-negative is rejected (`Regression: #381`)."));
    assert!(DOC.contains(
        "censorship edge-signal remains critical when targeted peers are at least two and delivery ratio is 500 per-mille or lower (`Regression: #382`)."
    ));
    assert!(DOC.contains(
        "backpressure overflow sample validation rejects queue depth above capacity (`Regression: #618`)."
    ));
    assert!(DOC.contains(
        "stale disconnected peer queue purge mapping remains deterministic (`Regression: #618`)."
    ));
    assert!(DOC.contains("proof consensus alignment (`ConsensusValid`) projects `info` severity."));
    assert!(DOC.contains(
        "proof consensus invalid/replay/mismatch projects `critical` severity with deterministic fingerprint fields."
    ));
    assert!(DOC.contains(
        "unscheduled proof-consensus deep-lane execution force-fails via scheduled/manual cadence guard (`Regression: #996`)."
    ));
    assert!(DOC.contains(
        "invalid, replay, and mismatch proof-consensus anomaly artifacts must remain `NO-GO` under policy checks (`Regression: #996`)."
    ));
}
