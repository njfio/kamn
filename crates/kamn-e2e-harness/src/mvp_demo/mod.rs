//! MVP evaluator demo report generation and verification.

mod agent_harness;
mod agent_harness_actor_receipts;
mod agent_harness_actor_rehearsal;
mod agent_harness_json;
mod agent_harness_observation_receipts;
mod agent_harness_three_agent;
mod artifact_digest;
mod command_config;
mod devnet_settlement;
mod devnet_settlement_build;
mod devnet_settlement_json;
mod devnet_settlement_live;
mod devnet_settlement_node;
mod devnet_settlement_service;
mod devnet_settlement_solana;
mod devnet_settlement_state;
mod local_artifact_paths;
mod local_artifact_verify;
mod local_artifacts;
mod localhost_signed;
mod report;
mod report_artifacts;
mod report_markdown;
mod report_writer;
mod runner;
mod service_api_proof;
mod three_agent_claim;
mod three_agent_receipt_spec;
mod three_agent_receipt_verify;
mod three_agent_receipt_verify_support;
mod three_agent_receipt_write;
mod three_agent_receipts;
mod three_agent_transcript;
mod three_agent_transcript_build;
mod three_agent_verify;
mod three_agent_view_artifacts;
mod three_agent_view_verify;
mod three_agent_views;
mod verify;
mod verify_support;

pub use command_config::{MvpDemoCommandConfig, VerifyMvpDemoCommandConfig};
pub use report::{
    CLAIM_LABEL_DEVNET_BACKED, CLAIM_LABEL_DRY_RUN, CLAIM_LABEL_LOCAL_ONLY,
    CLAIM_LABEL_PLACEHOLDER, CLAIM_LABEL_REAL, CLAIM_LABEL_ROADMAP, MVP_DEMO_REPORT_SCHEMA_VERSION,
};
pub use runner::{execute_mvp_demo_contract, execute_verify_mvp_demo_contract};
pub use verify::verify_mvp_demo_report_json;

pub(crate) const DEFAULT_MVP_DEMO_OUTPUT_ROOT: &str = ".kamn/demo";
