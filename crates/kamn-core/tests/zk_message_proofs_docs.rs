const DOC: &str = include_str!("../../../docs/foundation/zk-message-proof-design.md");

#[test]
fn doc_contains_kolme_constraints_and_design_options() {
    assert!(DOC.contains("## Kolme Constraints That Shape Design"));
    assert!(DOC.contains("single active processor"));
    assert!(DOC.contains("deterministic re-execution"));
    assert!(DOC.contains("## Architecture Options"));
    assert!(DOC.contains("groth16-processor-only"));
    assert!(DOC.contains("plonkish-batched-envelope"));
    assert!(DOC.contains("stark-recursive-watchdog"));
}

#[test]
fn doc_contains_complexity_trust_and_rollout() {
    assert!(DOC.contains("## Complexity and Trust Assumptions"));
    assert!(DOC.contains("trusted setup ceremony"));
    assert!(DOC.contains("watchdog sampling"));
    assert!(DOC.contains("## Recommended Phase 4 Rollout"));
    assert!(DOC.contains("Phase 4.0 - Feasibility harness"));
    assert!(DOC.contains("Phase 4.1 - Processor verification pilot"));
    assert!(DOC.contains("Phase 4.2 - Validator and watchdog expansion"));
}

#[test]
fn doc_contains_fast_cost_effective_validation_lane() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core --test zk_message_proofs"));
    assert!(DOC.contains("cargo clippy -- -D warnings"));
}

#[test]
fn regression_requires_boundary_inclusive_evaluation_rule() {
    // Regression: #62
    assert!(DOC.contains("threshold checks are inclusive"));
}

#[test]
fn regression_requires_tampered_processor_proof_rejection_rule() {
    // Regression: #509
    assert!(DOC.contains("## Processor Admission Guard Contract"));
    assert!(DOC.contains("tampered processor proof artifacts are rejected"));
}

#[test]
fn regression_requires_validator_watchdog_mismatch_projection_rule() {
    // Regression: #509
    assert!(DOC.contains("## Validator Quorum and Watchdog Projection Contract"));
    assert!(DOC.contains("ValidatorProofConsensusDecision"));
    assert!(DOC.contains("validator DID output is lexicographically ordered"));
    assert!(DOC.contains("ConsensusValid"));
    assert!(DOC.contains("validator-mismatch"));
    assert!(DOC.contains(
        "invalid-proof mismatch propagation must project as a critical validator mismatch signal"
    ));
}

#[test]
fn regression_requires_witness_artifact_contract_lane_marker() {
    // Regression: #993
    assert!(DOC.contains("## Witness and Artifact Schema Contract Lane"));
    assert!(DOC.contains("run_processor_proof_artifact_contract_lane.sh"));
    assert!(DOC.contains("private field selector syntax drift is rejected (`Regression: #993`)"));
}

#[test]
fn regression_requires_witness_mutation_fast_and_deep_lane_markers() {
    // Regression: #994
    assert!(DOC.contains("## Witness Mutation Property and Fuzz Lanes"));
    assert!(DOC.contains("run_zk_witness_mutation_contract_lane.sh"));
    assert!(DOC.contains("run_zk_witness_mutation_deep_lane.sh"));
    assert!(DOC.contains("performance_zk_witness_mutation_deep_lane_stress -- --ignored"));
}

#[test]
fn regression_requires_processor_admission_runtime_lane_markers() {
    // Regression: #995
    assert!(DOC.contains("## Processor Admission Runtime Contract Lane"));
    assert!(DOC.contains("run_processor_proof_admission_contract_lane.sh"));
    assert!(DOC.contains(
        "processor proof admission reason signatures remain fail-closed (`Regression: #995`)"
    ));
}
