use crate::{
    GovernanceVoteChoice, GovernanceVoteRecord, UpgradeOrchestrationError,
    VersionUpgradeOrchestrator,
};

pub(crate) fn apply_yes_votes_as_upgrade_approvals(
    orchestrator: &mut VersionUpgradeOrchestrator,
    proposal_id: &str,
    votes: Vec<GovernanceVoteRecord>,
) -> Result<(), UpgradeOrchestrationError> {
    for vote in votes {
        if vote.choice == GovernanceVoteChoice::Yes {
            orchestrator.approve_upgrade(proposal_id, &vote.voter_did, vote.cast_at_unix)?;
        }
    }
    Ok(())
}
