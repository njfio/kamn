//! Agent-driven runtime upgrade proposal, review, governance, and activation workflow contracts.

mod audit;
mod engine;
mod models;
mod support;
#[cfg(test)]
mod tests;

pub use audit::{AgentUpgradeAuditEvent, AgentUpgradeAuditEventKind};
pub use models::{
    AgentDrivenUpgradeWorkflow, AgentUpgradeProposalDraft, AgentUpgradeProposalRecord,
    AgentUpgradeProposalState, AgentUpgradeWorkflowConfig,
};
pub use support::AgentUpgradeWorkflowError;
