use kamn_sdk::{
    AgentMetadata, Artifact, ChannelId, EscrowConfig, InMemoryKamnClient, KamnAgent, Message,
    SdkError, TaskDefinition, TokenAmount,
};

fn sanitize(value: &str) -> String {
    value.replace('\n', " ")
}

fn run_demo() -> Result<(), SdkError> {
    let mut client = InMemoryKamnClient::new();

    let requester = client.register(AgentMetadata {
        agent_type: "requester".to_owned(),
        model_family: "gpt-4".to_owned(),
        capabilities: vec!["task-request".to_owned()],
    })?;

    let worker = client.register(AgentMetadata {
        agent_type: "worker".to_owned(),
        model_family: "llama-3".to_owned(),
        capabilities: vec!["analysis".to_owned(), "artifact-submit".to_owned()],
    })?;

    let requester_balance_before = client.balance(&requester)?.0;
    let worker_balance_before = client.balance(&worker)?.0;

    let message_id = client.send(Message {
        from: requester.clone(),
        to: worker.clone(),
        body: "Please summarize the attached signal bundle.".to_owned(),
        channel: Some(ChannelId("task-ops".to_owned())),
    })?;
    let inbox_records = client.receive(&worker)?;

    let task_id = client.create_task(TaskDefinition {
        creator: requester.clone(),
        task_type: "market-analysis".to_owned(),
        description: "Summarize the bundle and provide key risks.".to_owned(),
    })?;
    client.accept_task(&task_id, &worker)?;

    let artifact_id = client.submit_artifact(
        &task_id,
        Artifact {
            name: "analysis-summary.txt".to_owned(),
            bytes: b"trend=flat;risk=medium;action=monitor".to_vec(),
        },
    )?;
    client.complete_task(&task_id)?;

    let escrow_id = client.create_escrow(EscrowConfig {
        payer: requester.clone(),
        payee: worker.clone(),
        amount: TokenAmount(25),
    })?;
    client.release_escrow(&escrow_id)?;

    let requester_balance_after = client.balance(&requester)?.0;
    let worker_balance_after = client.balance(&worker)?.0;
    let worker_reputation = client.get_reputation(&worker)?.score;

    println!("status=ok");
    println!("requester_did={requester}");
    println!("worker_did={worker}");
    println!("message_id={}", message_id.0);
    println!("inbox_count={}", inbox_records.len());
    println!("task_id={}", task_id.0);
    println!("artifact_id={}", artifact_id.0);
    println!("escrow_id={}", escrow_id.0);
    println!("requester_balance_before={requester_balance_before}");
    println!("requester_balance_after={requester_balance_after}");
    println!("worker_balance_before={worker_balance_before}");
    println!("worker_balance_after={worker_balance_after}");
    println!("worker_reputation={worker_reputation}");

    Ok(())
}

fn main() {
    if let Err(error) = run_demo() {
        println!("status=error");
        println!("error={}", sanitize(&error.to_string()));
        std::process::exit(1);
    }
}
