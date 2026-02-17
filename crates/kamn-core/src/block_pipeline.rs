//! Mempool block production and consensus validation pipeline contracts.

use crate::p2p_transport::PeerLifecycleTransport;
use crate::runtime::{
    ApproverAttestation, ApproverQuorumDecision, ApproverQuorumError, ApproverQuorumEvaluator,
    ApproverQuorumInput, ListenerAttestation, ListenerQuorumDecision, ListenerQuorumError,
    ListenerQuorumEvaluator, ListenerQuorumInput,
};
use crate::smoke::{ProducedBlock, RoleSmokeNetwork, SmokeError};
use crate::sqlite_store_backend::{SqliteStoreBackend, SqliteStoreBackendError};
use crate::transaction::BaselineTransaction;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Input contract for one consensus-validation and block-commit round.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockConsensusRoundInput {
    /// Listener quorum event identifier.
    pub listener_event_id: String,
    /// Listener quorum event sequence.
    pub listener_event_sequence: u64,
    /// Approver outbound action identifier.
    pub outbound_action_id: String,
    /// Listener votes as `(listener_did, attestation_id)`.
    pub listener_votes: Vec<(String, String)>,
    /// Approver votes as `(approver_did, attestation_id, payload_digest_override)`.
    pub approver_votes: Vec<(String, String, Option<String>)>,
}

/// Commit report emitted when a consensus round commits a block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPipelineCommitReport {
    /// Committed block payload.
    pub block: ProducedBlock,
    /// Listener quorum decision used for admission.
    pub listener_decision: ListenerQuorumDecision,
    /// Approver quorum decision used for authorization.
    pub approver_decision: ApproverQuorumDecision,
    /// Deterministic payload digest used by approver quorum validation.
    pub payload_digest: String,
}

/// Error variants for block pipeline validation and commit flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockPipelineError {
    /// Listener quorum validation failed.
    Listener(ListenerQuorumError),
    /// Approver quorum validation failed.
    Approver(ApproverQuorumError),
    /// Smoke network transport/guard operation failed.
    Smoke(SmokeError),
    /// Pipeline attempted a consensus round with an empty mempool.
    EmptyMempool,
    /// Approver payload digest override mismatched deterministic block digest.
    ConsensusPayloadDigestMismatch {
        /// Expected deterministic digest.
        expected: String,
        /// Found override digest.
        found: String,
    },
    /// Transport feed returned an error while draining mempool candidates.
    TransportFeed(String),
    /// Canonical commit store returned an error while persisting/listing records.
    CommitStore(String),
    /// Fork-choice hook rejected canonical candidate block.
    ForkChoiceRejected {
        /// Deterministic reason code supplied by fork-choice hook.
        reason_code: String,
    },
    /// Restart/replay lineage drift detected for canonical commit persistence.
    ReplayDrift {
        /// Deterministic replay drift reason code.
        reason_code: String,
        /// Human-readable drift details.
        detail: String,
    },
}

impl BlockPipelineError {
    /// Returns deterministic reason code marker for policy and recovery matrix checks.
    pub fn reason_code(&self) -> String {
        match self {
            Self::Listener(_) => "block_pipeline_listener_error".to_owned(),
            Self::Approver(_) => "block_pipeline_approver_error".to_owned(),
            Self::Smoke(_) => "block_pipeline_smoke_error".to_owned(),
            Self::EmptyMempool => "block_pipeline_empty_mempool".to_owned(),
            Self::ConsensusPayloadDigestMismatch { .. } => {
                "block_pipeline_payload_digest_mismatch".to_owned()
            }
            Self::TransportFeed(detail) => validation::extract_error_reason_marker(detail)
                .unwrap_or_else(|| "block_pipeline_transport_feed_error".to_owned()),
            Self::CommitStore(detail) => validation::extract_error_reason_marker(detail)
                .unwrap_or_else(|| "block_pipeline_commit_store_error".to_owned()),
            Self::ForkChoiceRejected { reason_code } => reason_code.clone(),
            Self::ReplayDrift { reason_code, .. } => reason_code.clone(),
        }
    }
}

const DURABLE_COMMIT_CHECKER_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.durable-commit-checker-reason-taxonomy.v1";

/// Deterministic durable commit checker reason classifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurableCommitCheckerReasonClass {
    /// Replay/checkpoint lineage drift reason class.
    ReplayDrift,
    /// Commit-store persistence and parsing reason class.
    CommitStore,
    /// CI smoke/local-heavy boundary enforcement reason class.
    LaneBoundary,
    /// Fallback class for non-durable-commit-specific reason markers.
    Unclassified,
}

/// Deterministic durable commit checker reason projection output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurableCommitCheckerReasonProjection {
    reason_code: String,
    reason_class: DurableCommitCheckerReasonClass,
    source_marker: &'static str,
}

impl DurableCommitCheckerReasonProjection {
    fn new(reason_code: String, reason_class: DurableCommitCheckerReasonClass) -> Self {
        Self {
            reason_code,
            reason_class,
            source_marker: "durable_commit_checker_reason_projection",
        }
    }

    /// Returns deterministic projected reason code.
    pub fn reason_code(&self) -> &str {
        self.reason_code.as_str()
    }

    /// Returns deterministic projected reason class.
    pub fn reason_class(&self) -> DurableCommitCheckerReasonClass {
        self.reason_class
    }

    /// Returns deterministic projection source marker.
    pub fn source_marker(&self) -> &'static str {
        self.source_marker
    }

    /// Returns deterministic reason taxonomy marker for this projection.
    pub fn reason_taxonomy_version(&self) -> &'static str {
        durable_commit_checker_reason_taxonomy_version()
    }
}

/// Durable commit checker lane mode used for CI/local boundary enforcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurableCommitCheckerLaneMode {
    /// Low-cost CI smoke lane mode.
    CiSmoke,
    /// Opt-in local-heavy lane mode.
    LocalHeavy,
}

/// Deterministic lane-boundary report for durable commit checker enforcement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurableCommitCheckerLaneBoundaryReport {
    /// Final decision marker for boundary checks.
    pub final_decision: String,
    /// CI smoke/local-heavy boundary verification marker.
    pub ci_smoke_local_heavy_boundary_status: String,
    /// CI smoke cost profile marker.
    pub ci_smoke_lane_cost_profile: String,
    /// Local-heavy execution mode marker.
    pub local_heavy_lane_execution_mode: String,
    /// Deterministic boundary enforcement reason marker.
    pub enforcement_reason_code: String,
}

/// Returns deterministic durable commit checker reason taxonomy marker.
pub fn durable_commit_checker_reason_taxonomy_version() -> &'static str {
    DURABLE_COMMIT_CHECKER_REASON_TAXONOMY_VERSION
}

/// Projects deterministic reason output for durable commit checker failures.
pub fn project_durable_commit_checker_reason(
    error: &BlockPipelineError,
) -> DurableCommitCheckerReasonProjection {
    let reason_code = error.reason_code();
    let reason_class = classify_durable_commit_checker_reason(reason_code.as_str());
    DurableCommitCheckerReasonProjection::new(reason_code, reason_class)
}

/// Enforces deterministic CI-smoke and local-heavy durable commit checker boundaries.
pub fn enforce_durable_commit_checker_lane_boundary(
    lane_mode: DurableCommitCheckerLaneMode,
    ci_fast_gate: bool,
    local_heavy_opt_in: bool,
) -> Result<DurableCommitCheckerLaneBoundaryReport, BlockPipelineError> {
    match lane_mode {
        DurableCommitCheckerLaneMode::CiSmoke => {
            if !ci_fast_gate {
                return Err(BlockPipelineError::ReplayDrift {
                    reason_code: "durable_commit_checker_ci_smoke_fast_gate_required".to_owned(),
                    detail: "ci-smoke durable commit checker mode requires ci-fast-gate PASS"
                        .to_owned(),
                });
            }

            Ok(DurableCommitCheckerLaneBoundaryReport {
                final_decision: "GO".to_owned(),
                ci_smoke_local_heavy_boundary_status: "verified".to_owned(),
                ci_smoke_lane_cost_profile: "low".to_owned(),
                local_heavy_lane_execution_mode: "not-applicable".to_owned(),
                enforcement_reason_code: "durable_commit_checker_ci_smoke_boundary_verified"
                    .to_owned(),
            })
        }
        DurableCommitCheckerLaneMode::LocalHeavy => {
            if ci_fast_gate {
                return Err(BlockPipelineError::ReplayDrift {
                    reason_code: "durable_commit_checker_local_heavy_ci_fast_gate_mismatch"
                        .to_owned(),
                    detail:
                        "local-heavy durable commit checker mode must remain excluded from ci-fast-gate"
                            .to_owned(),
                });
            }
            if !local_heavy_opt_in {
                return Err(BlockPipelineError::ReplayDrift {
                    reason_code: "durable_commit_checker_local_heavy_opt_in_required".to_owned(),
                    detail:
                        "local-heavy durable commit checker mode requires explicit local opt-in"
                            .to_owned(),
                });
            }

            Ok(DurableCommitCheckerLaneBoundaryReport {
                final_decision: "GO".to_owned(),
                ci_smoke_local_heavy_boundary_status: "verified".to_owned(),
                ci_smoke_lane_cost_profile: "low".to_owned(),
                local_heavy_lane_execution_mode: "opt_in".to_owned(),
                enforcement_reason_code: "durable_commit_checker_local_heavy_boundary_verified"
                    .to_owned(),
            })
        }
    }
}

fn classify_durable_commit_checker_reason(reason_code: &str) -> DurableCommitCheckerReasonClass {
    if reason_code.starts_with("canonical_replay_") {
        DurableCommitCheckerReasonClass::ReplayDrift
    } else if reason_code.starts_with("canonical_commit_store_")
        || reason_code == "block_pipeline_commit_store_error"
    {
        DurableCommitCheckerReasonClass::CommitStore
    } else if reason_code.starts_with("durable_commit_checker_")
        || reason_code == "ci_fast_gate_failed"
    {
        DurableCommitCheckerReasonClass::LaneBoundary
    } else {
        DurableCommitCheckerReasonClass::Unclassified
    }
}

impl From<ListenerQuorumError> for BlockPipelineError {
    fn from(value: ListenerQuorumError) -> Self {
        Self::Listener(value)
    }
}

impl From<ApproverQuorumError> for BlockPipelineError {
    fn from(value: ApproverQuorumError) -> Self {
        Self::Approver(value)
    }
}

impl From<SmokeError> for BlockPipelineError {
    fn from(value: SmokeError) -> Self {
        Self::Smoke(value)
    }
}

impl Display for BlockPipelineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Listener(error) => write!(f, "{error}"),
            Self::Approver(error) => write!(f, "{error}"),
            Self::Smoke(error) => write!(f, "{error}"),
            Self::EmptyMempool => write!(
                f,
                "block pipeline requires at least one mempool transaction"
            ),
            Self::ConsensusPayloadDigestMismatch { expected, found } => {
                write!(
                    f,
                    "block pipeline payload digest mismatch: expected {expected}, found {found}"
                )
            }
            Self::TransportFeed(detail) => {
                write!(f, "block pipeline transport feed error: {detail}")
            }
            Self::CommitStore(detail) => write!(f, "block pipeline commit store error: {detail}"),
            Self::ForkChoiceRejected { reason_code } => {
                write!(
                    f,
                    "block pipeline fork-choice rejected candidate: {reason_code}"
                )
            }
            Self::ReplayDrift {
                reason_code,
                detail,
            } => {
                write!(f, "block pipeline replay drift: {detail} ({reason_code})")
            }
        }
    }
}

impl Error for BlockPipelineError {}

mod block_pipeline_support;
#[allow(dead_code)]
mod commit_store;
#[allow(dead_code)]
mod evidence;
#[allow(dead_code)]
mod fork_choice;
#[allow(dead_code)]
mod gossip_ingress;
#[allow(dead_code)]
mod validation;

pub use block_pipeline_support::*;
pub use gossip_ingress::{
    decode_transport_candidate_payload, decode_transport_canonical_candidate_payload,
    encode_transport_candidate_payload, encode_transport_canonical_candidate_payload,
    encode_transport_commit_report_payload,
};

/// Deterministic mempool->consensus->commit pipeline for processor runtime flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MempoolBlockPipeline {
    network: RoleSmokeNetwork,
    listener_evaluator: ListenerQuorumEvaluator,
    approver_required_approvals: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Transport-fed block pipeline that persists canonical commit records.
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
    /// Builds transport-fed block pipeline with supplied transport, persistence, and fork-choice hook.
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

    /// Reconciles transport-received canonical block candidates through fork-choice.
    pub fn reconcile_transport_candidates(
        &mut self,
    ) -> Result<Vec<CanonicalCandidateOutcome>, BlockPipelineError> {
        let mut candidates = self.transport_feed.drain_canonical_candidates()?;
        sort_canonical_candidates_for_reconciliation(&mut candidates);

        let mut outcomes = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            let block_height = candidate.block_height;
            let payload_digest = candidate.payload_digest.clone();
            match self.fork_choice_hook.evaluate_candidate(&candidate)? {
                ForkChoiceDecision::Accept => {
                    self.commit_store.persist_canonical_commit(candidate)?;
                    outcomes.push(CanonicalCandidateOutcome {
                        block_height,
                        payload_digest,
                        decision: CanonicalCandidateDecision::Accepted,
                    });
                }
                ForkChoiceDecision::Reject { reason_code } => {
                    outcomes.push(CanonicalCandidateOutcome {
                        block_height,
                        payload_digest,
                        decision: CanonicalCandidateDecision::Rejected { reason_code },
                    });
                }
            }
        }
        Ok(outcomes)
    }

    /// Runs one transport-fed consensus round and persists canonical commit record.
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

    /// Lists canonical commit records from configured persistence backend.
    pub fn list_canonical_commits(&self) -> Result<Vec<CanonicalCommitRecord>, BlockPipelineError> {
        self.commit_store.list_canonical_commits()
    }
}

fn sort_candidates_for_ingress(candidates: &mut [BaselineTransaction]) {
    candidates.sort_by(|left, right| {
        left.nonce
            .cmp(&right.nonce)
            .then_with(|| left.id.cmp(&right.id))
            .then_with(|| left.sender.cmp(&right.sender))
    });
}

fn sort_canonical_candidates_for_reconciliation(candidates: &mut [CanonicalCommitRecord]) {
    candidates.sort_by(|left, right| {
        left.block_height
            .cmp(&right.block_height)
            .then_with(|| left.payload_digest.cmp(&right.payload_digest))
            .then_with(|| {
                left.producer_role
                    .as_str()
                    .cmp(right.producer_role.as_str())
            })
            .then_with(|| left.transaction_ids.cmp(&right.transaction_ids))
    });
}

const CANONICAL_REPLAY_EVIDENCE_SCHEMA_VERSION: &str = "kamn.runtime.canonical-replay-evidence.v1";

/// Validates canonical lineage continuity across restart/replay checkpoints.
pub fn build_canonical_replay_evidence_bundle(
    pre_restart: &[CanonicalCommitRecord],
    post_restart: &[CanonicalCommitRecord],
) -> Result<CanonicalReplayEvidenceBundle, BlockPipelineError> {
    let Some(restart_boundary) = pre_restart.last() else {
        return Err(BlockPipelineError::ReplayDrift {
            reason_code: "canonical_replay_pre_restart_lineage_empty".to_owned(),
            detail: "pre-restart canonical lineage cannot be empty".to_owned(),
        });
    };

    for (index, expected) in pre_restart.iter().enumerate() {
        let Some(found) = post_restart.get(index) else {
            return Err(BlockPipelineError::ReplayDrift {
                reason_code: "canonical_replay_checkpoint_missing".to_owned(),
                detail: format!(
                    "post-restart lineage missing checkpoint at index {index} (expected height {})",
                    expected.block_height
                ),
            });
        };

        if found.block_height != expected.block_height {
            return Err(BlockPipelineError::ReplayDrift {
                reason_code: "canonical_replay_block_height_mismatch".to_owned(),
                detail: format!(
                    "canonical replay block height mismatch at index {index}: expected {}, found {}",
                    expected.block_height, found.block_height
                ),
            });
        }
        if found.producer_role != expected.producer_role {
            return Err(BlockPipelineError::ReplayDrift {
                reason_code: "canonical_replay_producer_role_mismatch".to_owned(),
                detail: format!(
                    "canonical replay producer role mismatch at index {index}: expected {}, found {}",
                    expected.producer_role.as_str(),
                    found.producer_role.as_str()
                ),
            });
        }
        if found.payload_digest != expected.payload_digest {
            return Err(BlockPipelineError::ReplayDrift {
                reason_code: "canonical_replay_payload_digest_mismatch".to_owned(),
                detail: format!(
                    "canonical replay payload digest mismatch at index {index}: expected {}, found {}",
                    expected.payload_digest, found.payload_digest
                ),
            });
        }
        if found.transaction_ids != expected.transaction_ids {
            return Err(BlockPipelineError::ReplayDrift {
                reason_code: "canonical_replay_transaction_ids_mismatch".to_owned(),
                detail: format!(
                    "canonical replay transaction ids mismatch at index {index}: expected {:?}, found {:?}",
                    expected.transaction_ids, found.transaction_ids
                ),
            });
        }
    }

    let replay_checkpoint = post_restart
        .get(pre_restart.len().saturating_sub(1))
        .ok_or_else(|| BlockPipelineError::ReplayDrift {
            reason_code: "canonical_replay_checkpoint_missing".to_owned(),
            detail: "post-restart lineage missing replay checkpoint".to_owned(),
        })?;

    Ok(CanonicalReplayEvidenceBundle {
        schema_version: CANONICAL_REPLAY_EVIDENCE_SCHEMA_VERSION.to_owned(),
        restart_boundary_block_height: restart_boundary.block_height,
        replay_checkpoint_block_height: replay_checkpoint.block_height,
        pre_restart_commit_count: pre_restart.len(),
        post_restart_commit_count: post_restart.len(),
        continuity_status: "verified".to_owned(),
    })
}

impl MempoolBlockPipeline {
    /// Builds a new block pipeline with explicit listener/approver quorum thresholds.
    pub fn new(
        gossip_enabled: bool,
        listener_required_confirmations: usize,
        approver_required_approvals: usize,
    ) -> Result<Self, BlockPipelineError> {
        let listener_evaluator = ListenerQuorumEvaluator::new(listener_required_confirmations)?;
        // Validate approver threshold during construction.
        let _ = ApproverQuorumEvaluator::new(approver_required_approvals)?;

        Ok(Self {
            network: RoleSmokeNetwork::new(gossip_enabled),
            listener_evaluator,
            approver_required_approvals,
        })
    }

    /// Returns expected state hash for the next mempool submission.
    pub fn expected_state_hash(&self) -> &str {
        self.network.expected_state_hash()
    }

    /// Returns current processor mempool length.
    pub fn processor_mempool_len(&self) -> usize {
        self.network.processor.mempool_len()
    }

    /// Submits a transaction into the pipeline mempool using existing guardrails.
    pub fn submit_transaction(
        &mut self,
        tx: BaselineTransaction,
    ) -> Result<(), BlockPipelineError> {
        self.network.submit_transaction(tx)?;
        Ok(())
    }

    /// Runs listener/approver consensus validation and commits one produced block.
    pub fn run_consensus_round(
        &mut self,
        input: BlockConsensusRoundInput,
    ) -> Result<BlockPipelineCommitReport, BlockPipelineError> {
        let pending = self.network.processor_mempool_snapshot();
        if pending.is_empty() {
            return Err(BlockPipelineError::EmptyMempool);
        }

        let payload_digest = payload_digest_for_transactions(&pending);
        for (_, _, override_digest) in &input.approver_votes {
            if let Some(found) = override_digest {
                if found != &payload_digest {
                    return Err(BlockPipelineError::ConsensusPayloadDigestMismatch {
                        expected: payload_digest.clone(),
                        found: found.clone(),
                    });
                }
            }
        }

        let listener_attestations = input
            .listener_votes
            .into_iter()
            .map(|(listener_did, attestation_id)| {
                ListenerAttestation::new(&listener_did, &attestation_id)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let listener_input = ListenerQuorumInput::new(
            &input.listener_event_id,
            input.listener_event_sequence,
            listener_attestations,
        )?;
        let listener_decision = self.listener_evaluator.evaluate(listener_input)?;

        let approver_attestations = input
            .approver_votes
            .into_iter()
            .map(|(approver_did, attestation_id, override_digest)| {
                let digest = override_digest.unwrap_or_else(|| payload_digest.clone());
                ApproverAttestation::new(&approver_did, &digest, &attestation_id)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let approver_input = ApproverQuorumInput::new(
            &input.outbound_action_id,
            &payload_digest,
            approver_attestations,
        )?;
        let approver_evaluator = ApproverQuorumEvaluator::new(self.approver_required_approvals)?;
        let approver_decision = approver_evaluator.authorize(approver_input)?;

        let block = self.network.produce_block()?;

        Ok(BlockPipelineCommitReport {
            block,
            listener_decision,
            approver_decision,
            payload_digest,
        })
    }
}

fn payload_digest_for_transactions(transactions: &[BaselineTransaction]) -> String {
    let mut ordered = transactions.to_vec();
    ordered.sort_by(|left, right| {
        left.nonce
            .cmp(&right.nonce)
            .then_with(|| left.id.cmp(&right.id))
            .then_with(|| left.sender.cmp(&right.sender))
    });

    let mut digest = String::from("block-payload");
    for tx in ordered {
        digest.push('|');
        digest.push_str(&tx.id);
        digest.push(':');
        digest.push_str(&tx.sender);
        digest.push(':');
        digest.push_str(&tx.nonce.to_string());
        digest.push(':');
        digest.push_str(&tx.state_hash);
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::{
        payload_digest_for_transactions, BlockPipelineError, CanonicalCommitRecord,
        DeterministicCompetingBranchForkChoiceHook, ForkChoiceDecision, ForkChoiceHook,
        MempoolBlockPipeline,
    };
    use crate::config::NodeRole;
    use crate::transaction::BaselineTransaction;

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
        let result = pipeline.run_consensus_round(super::BlockConsensusRoundInput {
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
        let seeded_head = CanonicalCommitRecord {
            block_height: 8,
            producer_role: NodeRole::Processor,
            payload_digest: "digest-z".to_owned(),
            transaction_ids: vec!["tx-z".to_owned()],
        };
        let mut hook = DeterministicCompetingBranchForkChoiceHook::with_canonical_head(seeded_head);
        let stale_candidate = CanonicalCommitRecord {
            block_height: 7,
            producer_role: NodeRole::Processor,
            payload_digest: "digest-a".to_owned(),
            transaction_ids: vec!["tx-a".to_owned()],
        };

        let decision = hook
            .evaluate_candidate(&stale_candidate)
            .expect("hook should evaluate stale candidate");
        assert_eq!(
            decision,
            ForkChoiceDecision::Reject {
                reason_code: "fork_choice_stale_block_height".to_owned(),
            }
        );
        assert_eq!(
            hook.canonical_head()
                .expect("seeded canonical head should remain set")
                .payload_digest,
            "digest-z"
        );
    }

    #[test]
    fn deterministic_competing_branch_hook_prefers_lexicographically_lower_digest_on_tie() {
        let mut hook = DeterministicCompetingBranchForkChoiceHook::new();
        let branch_high = CanonicalCommitRecord {
            block_height: 5,
            producer_role: NodeRole::Processor,
            payload_digest: "digest-b".to_owned(),
            transaction_ids: vec!["tx-b".to_owned()],
        };
        let branch_low = CanonicalCommitRecord {
            block_height: 5,
            producer_role: NodeRole::Processor,
            payload_digest: "digest-a".to_owned(),
            transaction_ids: vec!["tx-a".to_owned()],
        };

        let first = hook
            .evaluate_candidate(&branch_high)
            .expect("first branch should evaluate");
        let second = hook
            .evaluate_candidate(&branch_low)
            .expect("second branch should evaluate");

        assert_eq!(first, ForkChoiceDecision::Accept);
        assert_eq!(second, ForkChoiceDecision::Accept);
        assert_eq!(
            hook.canonical_head()
                .expect("head should be selected after tie break")
                .payload_digest,
            "digest-a"
        );
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
}
