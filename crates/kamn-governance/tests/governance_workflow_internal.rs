use kamn_governance::{
    GovernanceProposalDraft, GovernanceProposalStatus, GovernanceVoteChoice, GovernanceWorkflow,
    GovernanceWorkflowError,
};

fn draft(proposal_id: &str) -> GovernanceProposalDraft {
    GovernanceProposalDraft {
        proposal_id: proposal_id.to_owned(),
        title: format!("proposal {proposal_id}"),
        description: format!("description {proposal_id}"),
        proposer_did: "kamn:did:agent:validator-1".to_owned(),
        created_at_unix: 100,
        voting_deadline_unix: 200,
        quorum_threshold: 2,
        parameter_change: None,
    }
}

fn submit_default(workflow: &mut GovernanceWorkflow, proposal_id: &str) {
    workflow
        .submit_proposal(draft(proposal_id))
        .expect("proposal should submit");
}

fn cast_vote_ok(
    workflow: &mut GovernanceWorkflow,
    proposal_id: &str,
    voter_did: &str,
    choice: GovernanceVoteChoice,
    cast_at_unix: u64,
) {
    workflow
        .cast_vote(proposal_id, voter_did, choice, cast_at_unix)
        .expect("vote should pass");
}

fn approve(workflow: &mut GovernanceWorkflow, proposal_id: &str) {
    cast_vote_ok(
        workflow,
        proposal_id,
        "kamn:did:agent:validator-2",
        GovernanceVoteChoice::Yes,
        110,
    );
    cast_vote_ok(
        workflow,
        proposal_id,
        "kamn:did:agent:validator-3",
        GovernanceVoteChoice::Yes,
        111,
    );
}

#[test]
fn submit_rejects_invalid_deadline() {
    let mut workflow = GovernanceWorkflow::new();
    assert_eq!(
        workflow.submit_proposal(GovernanceProposalDraft {
            voting_deadline_unix: 99,
            quorum_threshold: 1,
            ..draft("gov-deadline")
        }),
        Err(GovernanceWorkflowError::InvalidDeadline {
            created_at_unix: 100,
            voting_deadline_unix: 99,
        })
    );
}

#[test]
fn no_quorum_transitions_to_rejected() {
    let mut workflow = GovernanceWorkflow::new();
    submit_default(&mut workflow, "gov-reject");
    cast_vote_ok(
        &mut workflow,
        "gov-reject",
        "kamn:did:agent:validator-2",
        GovernanceVoteChoice::No,
        110,
    );
    cast_vote_ok(
        &mut workflow,
        "gov-reject",
        "kamn:did:agent:validator-3",
        GovernanceVoteChoice::No,
        111,
    );

    assert_eq!(
        workflow
            .evaluate("gov-reject", 112)
            .expect("evaluation should succeed"),
        GovernanceProposalStatus::Rejected
    );
}

#[test]
fn yes_quorum_transitions_to_approved_and_execute_records_history() {
    let mut workflow = GovernanceWorkflow::new();
    submit_default(&mut workflow, "gov-approve");
    approve(&mut workflow, "gov-approve");
    assert_eq!(
        workflow
            .proposal("gov-approve")
            .expect("proposal exists")
            .status,
        GovernanceProposalStatus::Approved
    );

    let execution = workflow
        .execute(
            "gov-approve",
            "kamn:did:agent:validator-9",
            120,
            "op-hash-approve",
        )
        .expect("approved proposal should execute");

    assert_eq!(execution.proposal_id, "gov-approve");
    assert_eq!(workflow.execution_history().len(), 1);
}

#[test]
fn late_vote_expires_proposal_and_rejects_vote() {
    let mut workflow = GovernanceWorkflow::new();
    submit_default(&mut workflow, "gov-expire");

    assert_eq!(
        workflow.cast_vote(
            "gov-expire",
            "kamn:did:agent:validator-2",
            GovernanceVoteChoice::Yes,
            201,
        ),
        Err(GovernanceWorkflowError::ProposalClosed {
            proposal_id: "gov-expire".to_owned(),
            status: GovernanceProposalStatus::Expired,
        })
    );
}

#[test]
fn duplicate_vote_is_rejected_fail_closed() {
    let mut workflow = GovernanceWorkflow::new();
    submit_default(&mut workflow, "gov-duplicate-vote");
    cast_vote_ok(
        &mut workflow,
        "gov-duplicate-vote",
        "kamn:did:agent:validator-2",
        GovernanceVoteChoice::Abstain,
        110,
    );

    assert_eq!(
        workflow.cast_vote(
            "gov-duplicate-vote",
            "kamn:did:agent:validator-2",
            GovernanceVoteChoice::Yes,
            111,
        ),
        Err(GovernanceWorkflowError::DuplicateVote {
            proposal_id: "gov-duplicate-vote".to_owned(),
            voter_did: "kamn:did:agent:validator-2".to_owned(),
        })
    );
}

#[test]
fn execute_rejects_proposal_that_never_reached_approval() {
    let mut workflow = GovernanceWorkflow::new();
    submit_default(&mut workflow, "gov-not-approved");

    assert_eq!(
        workflow.execute(
            "gov-not-approved",
            "kamn:did:agent:validator-9",
            150,
            "op-hash-not-approved",
        ),
        Err(GovernanceWorkflowError::ProposalNotApproved {
            proposal_id: "gov-not-approved".to_owned(),
            status: GovernanceProposalStatus::Voting,
        })
    );
}
