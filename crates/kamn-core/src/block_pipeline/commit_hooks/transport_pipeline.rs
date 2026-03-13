use crate::block_pipeline::block_pipeline_support::{
    CanonicalCandidateDecision, CanonicalCandidateOutcome, CanonicalCommitRecord,
    CanonicalCommitStore, ForkChoiceDecision, ForkChoiceHook, TransportCanonicalCandidateFeed,
    TransportMempoolFeed,
};
use crate::block_pipeline::commit_hooks::pipeline::MempoolBlockPipeline;
use crate::block_pipeline::commit_hooks::sorting::{
    sort_candidates_for_ingress, sort_canonical_candidates_for_reconciliation,
};
use crate::block_pipeline::models::{
    BlockConsensusRoundInput, BlockPipelineCommitReport, BlockPipelineError,
};

/// Transport-fed block pipeline that persists canonical commit records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportFedBlockPipeline<TFeed, TStore, THook> {
    pipeline: MempoolBlockPipeline,
    transport_feed: TFeed,
    commit_store: TStore,
    fork_choice_hook: THook,
}

impl<TFeed, TStore, THook> TransportFedBlockPipeline<TFeed, TStore, THook>
where
    TFeed: TransportMempoolFeed + TransportCanonicalCandidateFeed,
    TStore: CanonicalCommitStore,
    THook: ForkChoiceHook,
{
    /// Builds a transport-fed pipeline with canonical commit persistence.
    pub fn new(
        gossip_enabled: bool,
        listener_required_confirmations: usize,
        approver_required_approvals: usize,
        transport_feed: TFeed,
        commit_store: TStore,
        fork_choice_hook: THook,
    ) -> Result<Self, BlockPipelineError> {
        Ok(Self {
            pipeline: MempoolBlockPipeline::new(
                gossip_enabled,
                listener_required_confirmations,
                approver_required_approvals,
            )?,
            transport_feed,
            commit_store,
            fork_choice_hook,
        })
    }

    /// Reconciles canonical candidates from the transport feed through fork choice.
    pub fn reconcile_transport_candidates(
        &mut self,
    ) -> Result<Vec<CanonicalCandidateOutcome>, BlockPipelineError> {
        let mut candidates = self.transport_feed.drain_canonical_candidates()?;
        sort_canonical_candidates_for_reconciliation(&mut candidates);
        let mut outcomes = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            outcomes.push(self.reconcile_transport_candidate(candidate)?);
        }
        Ok(outcomes)
    }

    /// Runs one transport-fed consensus round and persists the accepted commit.
    pub fn run_transport_consensus_round(
        &mut self,
        input: BlockConsensusRoundInput,
    ) -> Result<BlockPipelineCommitReport, BlockPipelineError> {
        let _candidate_outcomes = self.reconcile_transport_candidates()?;
        let mut candidates = self.transport_feed.drain_pending_transactions()?;
        if candidates.is_empty() {
            return Err(BlockPipelineError::EmptyMempool);
        }
        sort_candidates_for_ingress(&mut candidates);
        for candidate in candidates {
            self.pipeline.submit_transaction(candidate)?;
        }
        let report = self.pipeline.run_consensus_round(input)?;
        let commit_record = CanonicalCommitRecord::from_commit_report(&report);
        match self.fork_choice_hook.evaluate_candidate(&commit_record)? {
            ForkChoiceDecision::Accept => {}
            ForkChoiceDecision::Reject { reason_code } => {
                return Err(BlockPipelineError::ForkChoiceRejected { reason_code });
            }
        }
        self.commit_store.persist_canonical_commit(commit_record)?;
        Ok(report)
    }

    /// Lists the currently persisted canonical commits.
    pub fn list_canonical_commits(&self) -> Result<Vec<CanonicalCommitRecord>, BlockPipelineError> {
        self.commit_store.list_canonical_commits()
    }

    fn reconcile_transport_candidate(
        &mut self,
        candidate: CanonicalCommitRecord,
    ) -> Result<CanonicalCandidateOutcome, BlockPipelineError> {
        let block_height = candidate.block_height;
        let payload_digest = candidate.payload_digest.clone();
        match self.fork_choice_hook.evaluate_candidate(&candidate)? {
            ForkChoiceDecision::Accept => {
                self.commit_store.persist_canonical_commit(candidate)?;
                Ok(CanonicalCandidateOutcome {
                    block_height,
                    payload_digest,
                    decision: CanonicalCandidateDecision::Accepted,
                })
            }
            ForkChoiceDecision::Reject { reason_code } => Ok(CanonicalCandidateOutcome {
                block_height,
                payload_digest,
                decision: CanonicalCandidateDecision::Rejected { reason_code },
            }),
        }
    }
}
