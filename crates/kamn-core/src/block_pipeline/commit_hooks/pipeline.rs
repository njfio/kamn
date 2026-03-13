use crate::block_pipeline::commit_hooks::sorting::payload_digest_for_transactions;
use crate::block_pipeline::models::{
    BlockConsensusRoundInput, BlockPipelineCommitReport, BlockPipelineError,
};
use crate::runtime::{
    ApproverAttestation, ApproverQuorumEvaluator, ApproverQuorumInput, ListenerAttestation,
    ListenerQuorumEvaluator, ListenerQuorumInput,
};
use crate::smoke::RoleSmokeNetwork;
use crate::transaction::BaselineTransaction;

/// Deterministic mempool->consensus->commit pipeline for processor runtime flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MempoolBlockPipeline {
    network: RoleSmokeNetwork,
    listener_evaluator: ListenerQuorumEvaluator,
    approver_required_approvals: usize,
}

impl MempoolBlockPipeline {
    /// Builds a deterministic mempool pipeline with quorum thresholds.
    pub fn new(
        gossip_enabled: bool,
        listener_required_confirmations: usize,
        approver_required_approvals: usize,
    ) -> Result<Self, BlockPipelineError> {
        let listener_evaluator = ListenerQuorumEvaluator::new(listener_required_confirmations)?;
        let _ = ApproverQuorumEvaluator::new(approver_required_approvals)?;
        Ok(Self {
            network: RoleSmokeNetwork::new(gossip_enabled),
            listener_evaluator,
            approver_required_approvals,
        })
    }

    /// Returns the expected state hash for the current smoke-network state.
    pub fn expected_state_hash(&self) -> &str {
        self.network.expected_state_hash()
    }

    /// Returns the number of queued processor mempool entries.
    pub fn processor_mempool_len(&self) -> usize {
        self.network.processor.mempool_len()
    }

    /// Enqueues a new baseline transaction into the processor mempool.
    pub fn submit_transaction(
        &mut self,
        tx: BaselineTransaction,
    ) -> Result<(), BlockPipelineError> {
        self.network.submit_transaction(tx)?;
        Ok(())
    }

    /// Runs one deterministic consensus round and emits the commit report.
    pub fn run_consensus_round(
        &mut self,
        input: BlockConsensusRoundInput,
    ) -> Result<BlockPipelineCommitReport, BlockPipelineError> {
        let pending = self.network.processor_mempool_snapshot();
        if pending.is_empty() {
            return Err(BlockPipelineError::EmptyMempool);
        }
        let payload_digest = payload_digest_for_transactions(&pending);
        validate_approver_payload_overrides(&input, &payload_digest)?;
        let listener_decision = build_listener_decision(&mut self.listener_evaluator, &input)?;
        let approver_decision =
            build_approver_decision(self.approver_required_approvals, &input, &payload_digest)?;
        let block = self.network.produce_block()?;
        Ok(BlockPipelineCommitReport {
            block,
            listener_decision,
            approver_decision,
            payload_digest,
        })
    }
}

fn validate_approver_payload_overrides(
    input: &BlockConsensusRoundInput,
    payload_digest: &str,
) -> Result<(), BlockPipelineError> {
    for (_, _, override_digest) in &input.approver_votes {
        if let Some(found) = override_digest {
            if found != payload_digest {
                return Err(BlockPipelineError::ConsensusPayloadDigestMismatch {
                    expected: payload_digest.to_owned(),
                    found: found.clone(),
                });
            }
        }
    }
    Ok(())
}

fn build_listener_decision(
    evaluator: &mut ListenerQuorumEvaluator,
    input: &BlockConsensusRoundInput,
) -> Result<crate::runtime::ListenerQuorumDecision, BlockPipelineError> {
    let attestations = input
        .listener_votes
        .iter()
        .map(|(listener_did, attestation_id)| {
            ListenerAttestation::new(listener_did, attestation_id)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let quorum_input = ListenerQuorumInput::new(
        &input.listener_event_id,
        input.listener_event_sequence,
        attestations,
    )?;
    Ok(evaluator.evaluate(quorum_input)?)
}

fn build_approver_decision(
    required_approvals: usize,
    input: &BlockConsensusRoundInput,
    payload_digest: &str,
) -> Result<crate::runtime::ApproverQuorumDecision, BlockPipelineError> {
    let attestations = input
        .approver_votes
        .iter()
        .map(|(approver_did, attestation_id, override_digest)| {
            let digest = override_digest
                .clone()
                .unwrap_or_else(|| payload_digest.to_owned());
            ApproverAttestation::new(approver_did, &digest, attestation_id)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let quorum_input =
        ApproverQuorumInput::new(&input.outbound_action_id, payload_digest, attestations)?;
    let evaluator = ApproverQuorumEvaluator::new(required_approvals)?;
    Ok(evaluator.authorize(quorum_input)?)
}
