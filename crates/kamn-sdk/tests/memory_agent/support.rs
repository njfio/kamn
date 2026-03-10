pub(crate) use kamn_sdk::{
    AgentDid, AgentMetadata, AgentQuery, Artifact, ArtifactId, ArtifactStatus, ChannelId,
    EscrowConfig, InMemoryKamnClient, KamnAgent, Message, MessageId, SdkError, TaskDefinition,
    TaskId, TokenAmount,
};

pub(crate) fn metadata(agent_type: &str, model: &str, capabilities: &[&str]) -> AgentMetadata {
    AgentMetadata {
        agent_type: agent_type.to_owned(),
        model_family: model.to_owned(),
        capabilities: capabilities.iter().map(|cap| (*cap).to_owned()).collect(),
    }
}

pub(crate) fn valid_did(identifier: &str) -> AgentDid {
    match AgentDid::parse(format!("kamn:did:agent:{identifier}")) {
        Ok(did) => did,
        Err(error) => panic!("did parse failed: {error}"),
    }
}

pub(crate) fn register_agent(
    client: &mut InMemoryKamnClient,
    agent_type: &str,
    model: &str,
    capabilities: &[&str],
) -> AgentDid {
    client
        .register(metadata(agent_type, model, capabilities))
        .unwrap_or_else(|error| panic!("register failed: {error}"))
}

pub(crate) fn submit_analysis_task(
    client: &mut InMemoryKamnClient,
    creator: AgentDid,
    task_type: &str,
    description: &str,
) -> TaskId {
    client
        .create_task(TaskDefinition {
            creator,
            task_type: task_type.to_owned(),
            description: description.to_owned(),
        })
        .unwrap_or_else(|error| panic!("create task failed: {error}"))
}

pub(crate) fn submit_report_artifact(
    client: &mut InMemoryKamnClient,
    task_id: &TaskId,
) -> ArtifactId {
    client
        .submit_artifact(
            task_id,
            Artifact {
                name: "report.md".to_owned(),
                bytes: b"summary".to_vec(),
            },
        )
        .unwrap_or_else(|error| panic!("submit artifact failed: {error}"))
}

pub(crate) fn prepare_task_with_artifact(
    client: &mut InMemoryKamnClient,
) -> (TaskId, ArtifactId) {
    let creator = register_agent(client, "autonomous", "claude-4", &["research"]);
    let assignee = register_agent(client, "assistant", "gpt-5", &["research"]);
    let task_id = submit_analysis_task(client, creator, "analysis", "analyze benchmark results");
    client
        .accept_task(&task_id, &assignee)
        .expect("accept task should succeed");
    let artifact_id = submit_report_artifact(client, &task_id);
    (task_id, artifact_id)
}

pub(crate) fn assert_not_found(
    error: Result<impl core::fmt::Debug, SdkError>,
    entity: &'static str,
    id: &str,
) {
    assert_eq!(
        error.err(),
        Some(SdkError::NotFound {
            entity,
            id: id.to_owned(),
        })
    );
}

pub(crate) fn assert_artifact_status(
    client: &InMemoryKamnClient,
    artifact_id: &ArtifactId,
    lifecycle_state: &str,
    redaction_status: &str,
) {
    assert_eq!(
        client
            .get_artifact_status(artifact_id)
            .expect("artifact status should succeed"),
        ArtifactStatus {
            artifact_id: artifact_id.clone(),
            lifecycle_state: lifecycle_state.to_owned(),
            redaction_status: redaction_status.to_owned(),
        }
    );
}

pub(crate) fn assert_channel_id(channel: ChannelId, expected: &str) {
    assert_eq!(channel.0, expected);
}
