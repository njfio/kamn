use std::fmt;

use crate::agent_upgrade_workflow::support::errors::AgentUpgradeWorkflowError;

pub(super) fn format_state_error(
    error: &AgentUpgradeWorkflowError,
    f: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    match error {
        AgentUpgradeWorkflowError::DuplicateHumanReview {
            proposal_id,
            reviewer_did,
        } => Some(write!(
            f,
            "duplicate human review: proposal={proposal_id}, reviewer={reviewer_did}"
        )),
        AgentUpgradeWorkflowError::InsufficientHumanReviews { required, provided } => Some(write!(
            f,
            "insufficient human reviews: required {required}, provided {provided}"
        )),
        AgentUpgradeWorkflowError::GovernanceSubmissionNotAllowed { proposal_id, state } => {
            Some(write!(
                f,
                "governance submission not allowed: proposal={proposal_id}, state={state:?}"
            ))
        }
        AgentUpgradeWorkflowError::GovernanceStatusNotApproved {
            proposal_id,
            status,
        } => Some(write!(
            f,
            "governance status is not approved: proposal={proposal_id}, status={status:?}"
        )),
        AgentUpgradeWorkflowError::MissingGovernanceApprovalTimestamp(proposal_id) => Some(write!(
            f,
            "governance approval timestamp is missing for proposal: {proposal_id}"
        )),
        AgentUpgradeWorkflowError::ActivationDelayNotElapsed {
            proposal_id,
            earliest_activation_unix,
            attempted_activation_unix,
        } => Some(write!(
            f,
            "activation delay not elapsed: proposal={proposal_id}, earliest_activation_unix={earliest_activation_unix}, attempted_activation_unix={attempted_activation_unix}"
        )),
        _ => None,
    }
}
