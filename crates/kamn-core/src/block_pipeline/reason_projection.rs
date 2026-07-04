use super::models::BlockPipelineError;
use super::validation;

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

    /// Returns the deterministic reason code emitted for the pipeline error.
    pub fn reason_code(&self) -> &str {
        self.reason_code.as_str()
    }
    /// Returns the classified durable-commit-checker reason bucket.
    pub fn reason_class(&self) -> DurableCommitCheckerReasonClass {
        self.reason_class
    }
    /// Returns the fixed source marker for this projection.
    pub fn source_marker(&self) -> &'static str {
        self.source_marker
    }
    /// Returns the current durable-commit-checker reason taxonomy version.
    pub fn reason_taxonomy_version(&self) -> &'static str {
        durable_commit_checker_reason_taxonomy_version()
    }
}

/// Returns the durable-commit-checker reason taxonomy version marker.
pub fn durable_commit_checker_reason_taxonomy_version() -> &'static str {
    DURABLE_COMMIT_CHECKER_REASON_TAXONOMY_VERSION
}

/// Projects a block-pipeline error into a deterministic durable-commit-checker reason payload.
pub fn project_durable_commit_checker_reason(
    error: &BlockPipelineError,
) -> DurableCommitCheckerReasonProjection {
    let reason_code = error.reason_code();
    let reason_class = classify_durable_commit_checker_reason(reason_code.as_str());
    DurableCommitCheckerReasonProjection::new(reason_code, reason_class)
}

pub(crate) fn classify_durable_commit_checker_reason(
    reason_code: &str,
) -> DurableCommitCheckerReasonClass {
    if reason_code.contains("ci_smoke") || reason_code.contains("local_heavy") {
        return DurableCommitCheckerReasonClass::LaneBoundary;
    }
    if reason_code.starts_with("block_pipeline_commit_store")
        || reason_code.starts_with("canonical_commit_store")
    {
        return DurableCommitCheckerReasonClass::CommitStore;
    }
    if reason_code.starts_with("canonical_replay_") {
        return DurableCommitCheckerReasonClass::ReplayDrift;
    }
    DurableCommitCheckerReasonClass::Unclassified
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
