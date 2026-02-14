const DOC: &str = include_str!("../../../docs/architecture/block-pipeline.md");
const ROADMAP: &str = include_str!("../../../docs/plans/2026-02-08-production-service-roadmap.md");

#[test]
fn architecture_doc_contains_block_pipeline_core_components() {
    assert!(DOC.contains("MempoolBlockPipeline"));
    assert!(DOC.contains("BlockConsensusRoundInput"));
    assert!(DOC.contains("BlockPipelineCommitReport"));
    assert!(DOC.contains("BlockPipelineError"));
}

#[test]
fn architecture_doc_contains_consensus_and_runtime_wiring_contracts() {
    assert!(DOC.contains("ListenerQuorumEvaluator"));
    assert!(DOC.contains("ApproverQuorumEvaluator"));
    assert!(DOC.contains("RoleSmokeNetwork::produce_block"));
    assert!(DOC.contains("consensus-validator"));
}

#[test]
fn roadmap_references_phase_32_initial_block_pipeline_slice() {
    assert!(ROADMAP.contains("Phase 3.2 initial slice delivered"));
    assert!(ROADMAP.contains("Task #2926, Subtask #2927"));
    assert!(ROADMAP.contains("docs/architecture/block-pipeline.md"));
}

#[test]
fn regression_doc_tracks_digest_mismatch_fail_closed_guard() {
    // Regression: #2927
    assert!(DOC.contains("Regression: #2927"));
}
