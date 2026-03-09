use super::models::{
    GovernanceExecutionRecord, GovernanceProposalRecord, GovernanceProposalStatus,
    GovernanceVoteRecord,
};
use std::collections::BTreeMap;

/// In-memory governance workflow engine.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GovernanceWorkflow {
    pub(super) proposals: BTreeMap<String, GovernanceProposalState>,
    pub(super) execution_history: Vec<GovernanceExecutionRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GovernanceProposalState {
    pub(super) record: GovernanceProposalRecord,
    pub(super) votes: BTreeMap<String, GovernanceVoteRecord>,
}

pub(super) fn reevaluate_status(record: &mut GovernanceProposalRecord, now_unix: u64) {
    if record.status != GovernanceProposalStatus::Voting {
        return;
    }
    if record.yes_votes >= record.quorum_threshold {
        record.status = GovernanceProposalStatus::Approved;
        return;
    }
    if record.no_votes >= record.quorum_threshold {
        record.status = GovernanceProposalStatus::Rejected;
        return;
    }
    if now_unix > record.voting_deadline_unix {
        record.status = GovernanceProposalStatus::Expired;
    }
}
