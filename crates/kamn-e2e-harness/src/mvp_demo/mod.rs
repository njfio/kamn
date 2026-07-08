//! MVP evaluator demo report generation and verification.

mod devnet_settlement;
mod devnet_settlement_build;
mod devnet_settlement_json;
mod devnet_settlement_live;
mod devnet_settlement_node;
mod devnet_settlement_service;
mod devnet_settlement_solana;
mod devnet_settlement_state;
mod local_artifacts;
mod localhost_signed;
mod report;
mod report_artifacts;
mod runner;
mod service_api_proof;
mod three_agent_claim;
mod verify;
mod verify_support;

pub use report::{
    CLAIM_LABEL_DEVNET_BACKED, CLAIM_LABEL_DRY_RUN, CLAIM_LABEL_LOCAL_ONLY,
    CLAIM_LABEL_PLACEHOLDER, CLAIM_LABEL_REAL, CLAIM_LABEL_ROADMAP, MVP_DEMO_REPORT_SCHEMA_VERSION,
};
pub use runner::{
    execute_mvp_demo_contract, execute_verify_mvp_demo_contract, MvpDemoCommandConfig,
    VerifyMvpDemoCommandConfig,
};
pub use verify::verify_mvp_demo_report_json;

pub(crate) const DEFAULT_MVP_DEMO_OUTPUT_ROOT: &str = ".kamn/demo";
