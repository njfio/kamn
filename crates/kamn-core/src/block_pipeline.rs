//! Mempool block production and consensus validation pipeline contracts.

mod block_pipeline_support;
mod commit_hooks;
#[allow(dead_code)]
mod commit_store;
#[allow(dead_code)]
mod evidence;
#[allow(dead_code)]
mod fork_choice;
#[allow(dead_code)]
mod gossip_ingress;
mod lane_boundary;
mod models;
mod reason_projection;
#[cfg(test)]
mod tests;
#[allow(dead_code)]
mod validation;

pub use block_pipeline_support::*;
pub use commit_hooks::{
    build_canonical_replay_evidence_bundle, MempoolBlockPipeline, TransportFedBlockPipeline,
};
pub use gossip_ingress::{
    decode_transport_candidate_payload, decode_transport_canonical_candidate_payload,
    encode_transport_candidate_payload, encode_transport_canonical_candidate_payload,
    encode_transport_commit_report_payload,
};
pub use lane_boundary::{
    enforce_durable_commit_checker_lane_boundary, DurableCommitCheckerLaneBoundaryReport,
    DurableCommitCheckerLaneMode,
};
pub use models::{BlockConsensusRoundInput, BlockPipelineCommitReport, BlockPipelineError};
pub use reason_projection::{
    durable_commit_checker_reason_taxonomy_version, project_durable_commit_checker_reason,
    DurableCommitCheckerReasonClass, DurableCommitCheckerReasonProjection,
};

#[cfg(test)]
pub(crate) use commit_hooks::payload_digest_for_transactions;
