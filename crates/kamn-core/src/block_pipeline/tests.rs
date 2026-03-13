use super::{
    payload_digest_for_transactions, BlockConsensusRoundInput, BlockPipelineError,
    CanonicalCommitRecord, DeterministicCompetingBranchForkChoiceHook, ForkChoiceDecision,
    ForkChoiceHook, MempoolBlockPipeline,
};
use crate::config::NodeRole;
use crate::transaction::BaselineTransaction;

fn canonical_commit(
    block_height: u64,
    payload_digest: &str,
    transaction_id: &str,
) -> CanonicalCommitRecord {
    CanonicalCommitRecord {
        block_height,
        producer_role: NodeRole::Processor,
        payload_digest: payload_digest.to_owned(),
        transaction_ids: vec![transaction_id.to_owned()],
    }
}

fn assert_hook_decision(
    hook: &mut DeterministicCompetingBranchForkChoiceHook,
    candidate: &CanonicalCommitRecord,
) -> ForkChoiceDecision {
    hook.evaluate_candidate(candidate)
        .expect("fork-choice hook should evaluate candidate")
}

#[test]
fn constructor_rejects_zero_listener_quorum_threshold() {
    let result = MempoolBlockPipeline::new(true, 0, 1);
    assert!(matches!(result, Err(BlockPipelineError::Listener(_))));
}

#[test]
fn constructor_rejects_zero_approver_quorum_threshold() {
    let result = MempoolBlockPipeline::new(true, 1, 0);
    assert!(matches!(result, Err(BlockPipelineError::Approver(_))));
}

#[test]
fn regression_consensus_round_rejects_empty_mempool() {
    // Regression: #2927
    let mut pipeline = MempoolBlockPipeline::new(true, 1, 1).expect("pipeline builds");
    let result = pipeline.run_consensus_round(BlockConsensusRoundInput {
        listener_event_id: "event-1".to_owned(),
        listener_event_sequence: 1,
        outbound_action_id: "outbound-1".to_owned(),
        listener_votes: vec![("kamn:did:listener:alpha".to_owned(), "att-1".to_owned())],
        approver_votes: vec![(
            "kamn:did:agent:approver-alpha".to_owned(),
            "att-1".to_owned(),
            None,
        )],
    });
    assert_eq!(result, Err(BlockPipelineError::EmptyMempool));
}

#[test]
fn payload_digest_is_deterministic_across_orderings() {
    let tx1 = BaselineTransaction::signed("tx-1", "agent-a", 1, "p1", "state:genesis");
    let tx2 = BaselineTransaction::signed("tx-2", "agent-b", 1, "p2", "state:genesis");
    let digest_a = payload_digest_for_transactions(&[tx1.clone(), tx2.clone()]);
    let digest_b = payload_digest_for_transactions(&[tx2, tx1]);
    assert_eq!(digest_a, digest_b);
}

#[test]
fn deterministic_competing_branch_hook_rejects_stale_candidate_height() {
    let seeded_head = canonical_commit(8, "digest-z", "tx-z");
    let mut hook = DeterministicCompetingBranchForkChoiceHook::with_canonical_head(seeded_head);
    let stale_candidate = canonical_commit(7, "digest-a", "tx-a");
    let decision = assert_hook_decision(&mut hook, &stale_candidate);
    assert_eq!(
        decision,
        ForkChoiceDecision::Reject {
            reason_code: "fork_choice_stale_block_height".to_owned(),
        }
    );
    let head = hook
        .canonical_head()
        .expect("seeded canonical head should remain set");
    assert_eq!(head.payload_digest, "digest-z");
}

#[test]
fn deterministic_competing_branch_hook_prefers_lexicographically_lower_digest_on_tie() {
    let mut hook = DeterministicCompetingBranchForkChoiceHook::new();
    let branch_high = canonical_commit(5, "digest-b", "tx-b");
    let branch_low = canonical_commit(5, "digest-a", "tx-a");
    let first = assert_hook_decision(&mut hook, &branch_high);
    let second = assert_hook_decision(&mut hook, &branch_low);

    assert_eq!(first, ForkChoiceDecision::Accept);
    assert_eq!(second, ForkChoiceDecision::Accept);
    let head = hook
        .canonical_head()
        .expect("head should be selected after tie break");
    assert_eq!(head.payload_digest, "digest-a");
}

#[test]
fn deterministic_competing_branch_hook_rejects_duplicate_candidate() {
    let mut hook = DeterministicCompetingBranchForkChoiceHook::new();
    let candidate = CanonicalCommitRecord {
        block_height: 11,
        producer_role: NodeRole::Processor,
        payload_digest: "digest-11".to_owned(),
        transaction_ids: vec!["tx-11".to_owned()],
    };

    let first = hook
        .evaluate_candidate(&candidate)
        .expect("first candidate should evaluate");
    let second = hook
        .evaluate_candidate(&candidate)
        .expect("duplicate candidate should evaluate");

    assert_eq!(first, ForkChoiceDecision::Accept);
    assert_eq!(
        second,
        ForkChoiceDecision::Reject {
            reason_code: "fork_choice_duplicate_candidate".to_owned(),
        }
    );
}

#[test]
fn block_pipeline_error_reason_code_extracts_commit_store_marker() {
    let error = BlockPipelineError::CommitStore(
        "commit store read failed (canonical_commit_store_io)".to_owned(),
    );
    assert_eq!(error.reason_code(), "canonical_commit_store_io");
}

#[test]
fn block_pipeline_error_reason_code_uses_stable_fallback_when_marker_missing() {
    let error = BlockPipelineError::CommitStore("opaque commit store failure".to_owned());
    assert_eq!(error.reason_code(), "block_pipeline_commit_store_error");
}
