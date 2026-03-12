use crate::{GovernanceProposalRecord, VersionUpgradeAuditView};

use crate::agent_upgrade_workflow::{
    AgentDrivenUpgradeWorkflow, AgentUpgradeAuditEvent, AgentUpgradeProposalRecord,
};

impl AgentDrivenUpgradeWorkflow {
    /// Return proposal snapshot by identifier if present.
    pub fn proposal(&self, proposal_id: &str) -> Option<AgentUpgradeProposalRecord> {
        self.proposals.get(proposal_id).cloned()
    }

    /// Return mirrored governance proposal record by identifier if present.
    pub fn governance_record(&self, proposal_id: &str) -> Option<GovernanceProposalRecord> {
        self.governance.proposal(proposal_id)
    }

    /// Return upgrade-orchestrator audit view for all tracked operations.
    pub fn upgrade_audit_view(&self) -> VersionUpgradeAuditView {
        self.orchestrator.audit_view()
    }

    /// Return emitted agent workflow audit events in insertion order.
    pub fn agent_audit_log(&self) -> Vec<AgentUpgradeAuditEvent> {
        self.events.clone()
    }
}
