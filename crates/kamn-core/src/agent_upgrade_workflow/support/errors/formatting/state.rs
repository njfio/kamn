use std::fmt;

use crate::agent_upgrade_workflow::support::errors::AgentUpgradeWorkflowError;

pub(super) fn format_state_error(
    error: &AgentUpgradeWorkflowError,
    f: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    format_human_review_state_error(error, f).or_else(|| format_governance_state_error(error, f))
}

fn format_human_review_state_error(
    error: &AgentUpgradeWorkflowError,
    f: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    match error {
        AgentUpgradeWorkflowError::DuplicateHumanReview {
            proposal_id,
            reviewer_did,
        } => Some(format_duplicate_human_review(f, proposal_id, reviewer_did)),
        AgentUpgradeWorkflowError::InsufficientHumanReviews { required, provided } => {
            Some(format_insufficient_human_reviews(f, *required, *provided))
        }
        _ => None,
    }
}

fn format_governance_state_error(
    error: &AgentUpgradeWorkflowError,
    f: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    format_governance_submission_error(error, f)
        .or_else(|| format_governance_activation_error(error, f))
}

fn format_governance_submission_error(
    error: &AgentUpgradeWorkflowError,
    f: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    match error {
        AgentUpgradeWorkflowError::GovernanceSubmissionNotAllowed { proposal_id, state } => Some(
            format_governance_submission_not_allowed(f, proposal_id, state),
        ),
        AgentUpgradeWorkflowError::GovernanceStatusNotApproved {
            proposal_id,
            status,
        } => Some(format_governance_status_not_approved(
            f,
            proposal_id,
            status,
        )),
        _ => None,
    }
}

fn format_governance_activation_error(
    error: &AgentUpgradeWorkflowError,
    f: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    match error {
        AgentUpgradeWorkflowError::MissingGovernanceApprovalTimestamp(proposal_id) => {
            Some(format_missing_approval_timestamp(f, proposal_id))
        }
        AgentUpgradeWorkflowError::ActivationDelayNotElapsed {
            proposal_id,
            earliest_activation_unix,
            attempted_activation_unix,
        } => Some(format_activation_delay_not_elapsed(
            f,
            proposal_id,
            *earliest_activation_unix,
            *attempted_activation_unix,
        )),
        _ => None,
    }
}

fn format_duplicate_human_review(
    f: &mut fmt::Formatter<'_>,
    proposal_id: &str,
    reviewer_did: &str,
) -> fmt::Result {
    write!(
        f,
        "duplicate human review: proposal={proposal_id}, reviewer={reviewer_did}"
    )
}

fn format_insufficient_human_reviews(
    f: &mut fmt::Formatter<'_>,
    required: usize,
    provided: usize,
) -> fmt::Result {
    write!(
        f,
        "insufficient human reviews: required {required}, provided {provided}"
    )
}

fn format_governance_submission_not_allowed(
    f: &mut fmt::Formatter<'_>,
    proposal_id: &str,
    state: &impl fmt::Debug,
) -> fmt::Result {
    write!(
        f,
        "governance submission not allowed: proposal={proposal_id}, state={state:?}"
    )
}

fn format_governance_status_not_approved(
    f: &mut fmt::Formatter<'_>,
    proposal_id: &str,
    status: &impl fmt::Debug,
) -> fmt::Result {
    write!(
        f,
        "governance status is not approved: proposal={proposal_id}, status={status:?}"
    )
}

fn format_missing_approval_timestamp(f: &mut fmt::Formatter<'_>, proposal_id: &str) -> fmt::Result {
    write!(
        f,
        "governance approval timestamp is missing for proposal: {proposal_id}"
    )
}

fn format_activation_delay_not_elapsed(
    f: &mut fmt::Formatter<'_>,
    proposal_id: &str,
    earliest_activation_unix: u64,
    attempted_activation_unix: u64,
) -> fmt::Result {
    write!(
        f,
        "activation delay not elapsed: proposal={proposal_id}, earliest_activation_unix={earliest_activation_unix}, attempted_activation_unix={attempted_activation_unix}"
    )
}
