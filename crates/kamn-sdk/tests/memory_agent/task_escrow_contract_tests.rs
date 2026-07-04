use super::support::*;

#[test]
fn task_accept_rejects_second_acceptance() {
    let mut client = InMemoryKamnClient::new();
    let (creator, assignee) = registered_research_pair(&mut client);
    let task_id = submit_analysis_task(&mut client, creator, "research", "compare protocols");
    client
        .accept_task(&task_id, &assignee)
        .expect("first accept failed");
    assert_eq!(
        client.accept_task(&task_id, &assignee),
        Err(SdkError::Conflict("task already accepted"))
    );
}

#[test]
fn get_task_status_reports_submitted_accepted_and_completed_states() {
    let mut client = InMemoryKamnClient::new();
    let (creator, assignee) = registered_research_pair(&mut client);
    let task_id = submit_analysis_task(&mut client, creator, "research", "compare protocols");
    assert_eq!(
        client
            .get_task_status(&task_id)
            .expect("submitted status")
            .state,
        "submitted"
    );
    client
        .accept_task(&task_id, &assignee)
        .expect("accept task should succeed");
    assert_eq!(
        client
            .get_task_status(&task_id)
            .expect("accepted status")
            .state,
        "accepted"
    );
    client
        .complete_task(&task_id)
        .expect("complete task should succeed");
    assert_eq!(
        client
            .get_task_status(&task_id)
            .expect("completed status")
            .state,
        "completed"
    );
}

#[test]
fn get_task_status_rejects_unknown_task() {
    let client = InMemoryKamnClient::new();
    assert_not_found(client.get_task_status(&TaskId(45)), "task", "45");
}

#[test]
fn escrow_moves_balances_from_payer_to_payee() {
    let mut client = InMemoryKamnClient::new();
    let payer = register_agent(&mut client, "autonomous", "claude-4", &["pay"]);
    let payee = register_agent(&mut client, "assistant", "gpt-5", &["deliver"]);
    let payer_before = client.balance(&payer).expect("payer before failed");
    let payee_before = client.balance(&payee).expect("payee before failed");
    let escrow_id = client
        .create_escrow(EscrowConfig {
            payer: payer.clone(),
            payee: payee.clone(),
            amount: TokenAmount(25),
        })
        .expect("create escrow failed");
    client
        .release_escrow(&escrow_id)
        .expect("release escrow failed");
    let payer_after = client.balance(&payer).expect("payer after failed");
    let payee_after = client.balance(&payee).expect("payee after failed");
    assert_eq!(payer_after.0, payer_before.0.saturating_sub(25));
    assert_eq!(payee_after.0, payee_before.0.saturating_add(25));
}

#[test]
fn submit_artifact_and_complete_task_flow() {
    let mut client = InMemoryKamnClient::new();
    let (creator, assignee) = registered_research_pair(&mut client);
    let task_id = submit_analysis_task(
        &mut client,
        creator,
        "analysis",
        "analyze benchmark results",
    );
    client
        .accept_task(&task_id, &assignee)
        .expect("accept task failed");
    let artifact_id = submit_report_artifact(&mut client, &task_id);
    assert!(artifact_id.0 > 0);
    client
        .complete_task(&task_id)
        .expect("complete task failed");
    assert_eq!(
        client.complete_task(&task_id),
        Err(SdkError::Conflict("task already completed"))
    );
}
