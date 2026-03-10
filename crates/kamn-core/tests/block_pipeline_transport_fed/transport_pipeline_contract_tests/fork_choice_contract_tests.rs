use super::super::support::*;

#[test]
fn regression_competing_branch_fork_choice_prefers_stable_head_independent_of_candidate_order() {
    let mut hook_a = DeterministicCompetingBranchForkChoiceHook::default();
    let mut hook_b = DeterministicCompetingBranchForkChoiceHook::default();
    let branch_a = sample_canonical_record(9, "digest-b", "tx-b");
    let branch_b = sample_canonical_record(9, "digest-a", "tx-a");

    hook_a.evaluate_candidate(&branch_a).expect("first branch should evaluate");
    hook_a.evaluate_candidate(&branch_b).expect("second branch should evaluate");
    hook_b.evaluate_candidate(&branch_b).expect("first branch should evaluate");
    hook_b.evaluate_candidate(&branch_a).expect("second branch should evaluate");

    let head_a = hook_a.canonical_head().expect("head should be assigned after competing candidates");
    let head_b = hook_b.canonical_head().expect("head should be assigned after competing candidates");
    assert_eq!(head_a.payload_digest, "digest-a");
    assert_eq!(head_b.payload_digest, "digest-a");
}

#[test]
fn regression_transport_fed_pipeline_rejects_stale_candidate_against_seeded_head() {
    let feed = InMemoryTransportMempoolFeed::new(build_valid_chain_transactions());
    let store = InMemoryCanonicalCommitStore::default();
    let seeded_head = sample_canonical_record(50, "head-50", "head-tx");
    let hook = DeterministicCompetingBranchForkChoiceHook::with_canonical_head(seeded_head);
    let mut pipeline = TransportFedBlockPipeline::new(true, 1, 1, feed, store, hook)
        .expect("transport-fed pipeline should build");

    let result = pipeline.run_transport_consensus_round(sample_consensus_input());
    assert_eq!(result, Err(BlockPipelineError::ForkChoiceRejected { reason_code: "fork_choice_stale_block_height".to_owned() }));
    assert!(pipeline.list_canonical_commits().expect("canonical commit list should load").is_empty());
}
