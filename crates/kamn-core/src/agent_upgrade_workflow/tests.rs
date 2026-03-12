use super::{
    AgentDrivenUpgradeWorkflow, AgentUpgradeProposalDraft, AgentUpgradeProposalState,
    AgentUpgradeWorkflowConfig, AgentUpgradeWorkflowError,
};
use crate::GovernanceVoteChoice;

const PROPOSER_DID: &str = "kamn:did:agent:upgrade-bot";
const VALIDATOR_DID: &str = "kamn:did:agent:validator-1";
const ROGUE_VALIDATOR_DID: &str = "kamn:did:agent:validator-rogue";

#[test]
fn constructor_requires_allowlisted_agent_set() {
    assert_eq!(
        AgentDrivenUpgradeWorkflow::new(config(Vec::new(), vec![VALIDATOR_DID.to_owned()], 2)),
        Err(AgentUpgradeWorkflowError::MissingAllowedAgentProposers)
    );
}

#[test]
fn constructor_requires_allowlisted_validator_set() {
    assert_eq!(
        AgentDrivenUpgradeWorkflow::new(config(vec![PROPOSER_DID.to_owned()], Vec::new(), 2)),
        Err(AgentUpgradeWorkflowError::MissingAllowedValidatorVoters)
    );
}

#[test]
fn constructor_rejects_zero_activation_delay() {
    assert_eq!(
        AgentDrivenUpgradeWorkflow::new(AgentUpgradeWorkflowConfig {
            min_activation_delay_secs: 0,
            ..config(
                vec![PROPOSER_DID.to_owned()],
                vec![VALIDATOR_DID.to_owned()],
                2
            )
        }),
        Err(AgentUpgradeWorkflowError::InvalidMinActivationDelaySecs(0))
    );
}

#[test]
fn duplicate_human_review_is_rejected() {
    let mut workflow = seeded_workflow("agent-upgrade-a", "initial proposal", 2);
    approve_review(&mut workflow, "agent-upgrade-a", VALIDATOR_DID, 110);

    assert_eq!(
        workflow.approve_human_review("agent-upgrade-a", VALIDATOR_DID, 111),
        Err(AgentUpgradeWorkflowError::DuplicateHumanReview {
            proposal_id: "agent-upgrade-a".to_owned(),
            reviewer_did: VALIDATOR_DID.to_owned(),
        })
    );

    assert_pending_human_review(&workflow, "agent-upgrade-a");
}

#[test]
fn cast_validator_vote_rejects_non_allowlisted_validator() {
    let mut workflow = seeded_governance_workflow("agent-upgrade-b", "validator-vote-allowlist");

    assert_eq!(
        workflow.cast_validator_vote(
            "agent-upgrade-b",
            ROGUE_VALIDATOR_DID,
            GovernanceVoteChoice::Yes,
            130,
        ),
        Err(AgentUpgradeWorkflowError::UnauthorizedValidatorVoter(
            ROGUE_VALIDATOR_DID.to_owned()
        ))
    );
}

#[test]
fn approve_human_review_rejects_non_allowlisted_reviewer() {
    let mut workflow = seeded_workflow("agent-upgrade-c", "reviewer-allowlist", 1);

    assert_eq!(
        workflow.approve_human_review("agent-upgrade-c", ROGUE_VALIDATOR_DID, 110),
        Err(AgentUpgradeWorkflowError::UnauthorizedHumanReviewer(
            ROGUE_VALIDATOR_DID.to_owned()
        ))
    );
}

fn config(
    allowed_agent_proposers: Vec<String>,
    allowed_validator_voters: Vec<String>,
    required_validator_quorum: usize,
) -> AgentUpgradeWorkflowConfig {
    AgentUpgradeWorkflowConfig {
        current_version: "v0.1.0".to_owned(),
        allowed_agent_proposers,
        allowed_validator_voters,
        required_human_reviews: 1,
        required_validator_quorum,
        min_activation_delay_secs: 60,
    }
}

fn seeded_workflow(
    proposal_id: &str,
    rationale: &str,
    required_validator_quorum: usize,
) -> AgentDrivenUpgradeWorkflow {
    let mut workflow = new_workflow(required_validator_quorum);
    register_proposal(&mut workflow, proposal_id, rationale);
    workflow
}

fn seeded_governance_workflow(proposal_id: &str, rationale: &str) -> AgentDrivenUpgradeWorkflow {
    let mut workflow = seeded_workflow(proposal_id, rationale, 1);
    approve_review(&mut workflow, proposal_id, VALIDATOR_DID, 110);
    workflow
        .submit_to_governance(proposal_id, 120)
        .expect("governance submission should pass");
    workflow
}

fn new_workflow(required_validator_quorum: usize) -> AgentDrivenUpgradeWorkflow {
    AgentDrivenUpgradeWorkflow::new(config(
        vec![PROPOSER_DID.to_owned()],
        vec![VALIDATOR_DID.to_owned()],
        required_validator_quorum,
    ))
    .expect("workflow should initialize")
}

fn register_proposal(
    workflow: &mut AgentDrivenUpgradeWorkflow,
    proposal_id: &str,
    rationale: &str,
) {
    workflow
        .submit_agent_proposal(proposal_draft(proposal_id, rationale))
        .expect("proposal should register");
}

fn proposal_draft(proposal_id: &str, rationale: &str) -> AgentUpgradeProposalDraft {
    AgentUpgradeProposalDraft {
        proposal_id: proposal_id.to_owned(),
        target_version: "v0.2.0".to_owned(),
        agent_did: PROPOSER_DID.to_owned(),
        rationale: rationale.to_owned(),
        created_at_unix: 100,
        voting_deadline_unix: 200,
    }
}

fn approve_review(
    workflow: &mut AgentDrivenUpgradeWorkflow,
    proposal_id: &str,
    reviewer_did: &str,
    reviewed_at_unix: u64,
) {
    workflow
        .approve_human_review(proposal_id, reviewer_did, reviewed_at_unix)
        .expect("review should pass");
}

fn assert_pending_human_review(workflow: &AgentDrivenUpgradeWorkflow, proposal_id: &str) {
    let record = workflow
        .proposal(proposal_id)
        .expect("proposal should exist");
    assert_eq!(record.state, AgentUpgradeProposalState::PendingHumanReview);
}
