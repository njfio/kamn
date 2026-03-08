use super::{
    LiveTransportKamnClient,
    bridge::{
        bridge_read_auth, bridge_status_from_service, bridge_status_from_submission,
        bridge_submit_payload, bridge_write_auth, resolve_service_bridge_id,
        resolve_service_message_id,
    },
    config::{CHANNELS_WRITE_SCOPE, CONTENT_READ_SCOPE, MESSAGES_READ_SCOPE, TASKS_READ_SCOPE},
    routes::{
        agent_profile_to_document, agent_profile_to_reputation, agent_profile_to_summary,
        channel_create_payload, recipient_mailbox_channel_id, service_message_to_record,
    },
    task_escrow::{prepare_artifact_status_lookup, prepare_task_status_lookup},
    state::{build_agents_read_auth, build_agents_read_auth_with_body, build_auth, remember_message_id},
};
use crate::{
    channel_create::channel_id as validate_channel_id,
    AgentDid, AgentMetadata, AgentQuery, AgentReputation, AgentSummary, Artifact, ArtifactId,
    ArtifactStatus, BridgeId, BridgeStatus, ChannelId, DidDocument, EscrowConfig, EscrowId,
    KamnAgent, Message, MessageId, MessageRecord, MessageStatus, MessageStream, SdkError,
    ServiceContentStatus, ServiceMessageStatus, ServiceRequestAuth, TaskDefinition, TaskId,
    TaskStatus, TokenAmount, service::agent_search_payload,
};

impl KamnAgent for LiveTransportKamnClient {
    fn register(&mut self, metadata: AgentMetadata) -> Result<AgentDid, SdkError> {
        self.register_via_service(metadata)
    }

    fn resolve(&self, did: &AgentDid) -> Result<DidDocument, SdkError> {
        let auth = build_agents_read_auth(&self.state, &self.config)?;
        let profile = self.service_client.get_agent_profile(did.as_str(), &auth)?;
        agent_profile_to_document(profile, self.endpoint())
    }

    fn send(&mut self, message: Message) -> Result<MessageId, SdkError> {
        self.send_via_service(message)
    }

    fn get_message_status(&self, message_id: &MessageId) -> Result<MessageStatus, SdkError> {
        let service_message_id = resolve_service_message_id(self, message_id)?;
        let auth = message_read_auth(self)?;
        let status = self
            .service_client
            .get_message(service_message_id.as_str(), &auth)?;
        Ok(message_status_from_service(message_id, status))
    }

    fn submit_bridge(
        &mut self,
        source_message_id: &MessageId,
        target_network: &str,
    ) -> Result<BridgeStatus, SdkError> {
        let service_message_id = resolve_service_message_id(self, source_message_id)?;
        let payload = bridge_submit_payload(service_message_id.as_str(), target_network)?;
        let auth = bridge_write_auth(self, payload.as_str())?;
        let submission = self
            .service_client
            .submit_bridge_message(payload.as_str(), &auth)?;
        bridge_status_from_submission(self, submission)
    }

    fn forward_bridge(&mut self, bridge_id: &BridgeId) -> Result<BridgeStatus, SdkError> {
        let service_bridge_id = resolve_service_bridge_id(self, bridge_id)?;
        let auth = bridge_write_auth(self, "{}")?;
        let status = self
            .service_client
            .forward_bridge_message(service_bridge_id.as_str(), &auth)?;
        bridge_status_from_service(self, bridge_id, status)
    }

    fn get_bridge_status(&self, bridge_id: &BridgeId) -> Result<BridgeStatus, SdkError> {
        let service_bridge_id = resolve_service_bridge_id(self, bridge_id)?;
        let auth = bridge_read_auth(self)?;
        let status = self
            .service_client
            .get_bridge_message(service_bridge_id.as_str(), &auth)?;
        bridge_status_from_service(self, bridge_id, status)
    }

    fn receive(&mut self, did: &AgentDid) -> Result<Vec<MessageRecord>, SdkError> {
        let mailbox_auth = build_agents_read_auth(&self.state, &self.config)?;
        let mailbox = self
            .service_client
            .list_channel_messages(recipient_mailbox_channel_id(did).as_str(), &mailbox_auth)?;
        let mut records = Vec::with_capacity(mailbox.messages.len());
        for service_message_id in mailbox.messages {
            let message_auth = build_agents_read_auth(&self.state, &self.config)?;
            let delivery = self
                .service_client
                .get_message_delivery(service_message_id.as_str(), &message_auth)?;
            let message_id = remember_message_id(&self.state, delivery.message_id.as_str())?;
            records.push(service_message_to_record(delivery, message_id)?);
        }
        Ok(records)
    }

    fn receive_stream(&mut self, did: &AgentDid) -> Result<MessageStream, SdkError> {
        Ok(MessageStream::new(self.receive(did)?))
    }

    fn create_channel(&mut self, name: &str) -> Result<ChannelId, SdkError> {
        let payload = channel_create_payload(name)?;
        let auth = build_auth(
            &self.state,
            &self.config,
            &self.config.requester_did,
            payload.as_str(),
            Some(CHANNELS_WRITE_SCOPE),
        )?;
        let receipt = self.service_client.create_channel(payload.as_str(), &auth)?;
        validate_channel_id(receipt.channel_id)
    }

    fn create_task(&mut self, task: TaskDefinition) -> Result<TaskId, SdkError> {
        self.create_task_via_service(task)
    }

    fn accept_task(&mut self, task_id: &TaskId, assignee: &AgentDid) -> Result<(), SdkError> {
        self.accept_task_via_service(task_id, assignee)
    }

    fn get_task_status(&self, task_id: &TaskId) -> Result<TaskStatus, SdkError> {
        let service_task_id = resolve_service_task_id(self, task_id)?;
        let auth = build_auth(
            &self.state,
            &self.config,
            &self.config.requester_did,
            "",
            Some(TASKS_READ_SCOPE),
        )?;
        let status = self
            .service_client
            .get_task(service_task_id.as_str(), &auth)?;
        Ok(TaskStatus::from_state(task_id, status.state.as_str()))
    }

    fn submit_artifact(
        &mut self,
        task_id: &TaskId,
        artifact: Artifact,
    ) -> Result<ArtifactId, SdkError> {
        self.submit_artifact_via_service(task_id, artifact)
    }

    fn get_artifact_status(&self, artifact_id: &ArtifactId) -> Result<ArtifactStatus, SdkError> {
        let service_content_id = resolve_service_content_id(self, artifact_id)?;
        let auth = build_auth(
            &self.state,
            &self.config,
            &self.config.requester_did,
            "",
            Some(CONTENT_READ_SCOPE),
        )?;
        let status = self
            .service_client
            .get_content(service_content_id.as_str(), &auth)?;
        Ok(artifact_status_from_service(artifact_id, status))
    }

    fn expire_artifact(&mut self, artifact_id: &ArtifactId) -> Result<ArtifactStatus, SdkError> {
        let service_content_id = resolve_service_content_id(self, artifact_id)?;
        let auth = build_auth(
            &self.state,
            &self.config,
            &self.config.requester_did,
            "{}",
            Some(super::config::CONTENT_WRITE_SCOPE),
        )?;
        let status = self
            .service_client
            .expire_content(service_content_id.as_str(), &auth)?;
        Ok(artifact_status_from_service(artifact_id, status))
    }

    fn tombstone_artifact(
        &mut self,
        artifact_id: &ArtifactId,
    ) -> Result<ArtifactStatus, SdkError> {
        let service_content_id = resolve_service_content_id(self, artifact_id)?;
        let auth = build_auth(
            &self.state,
            &self.config,
            &self.config.requester_did,
            "{}",
            Some(super::config::CONTENT_WRITE_SCOPE),
        )?;
        let status = self
            .service_client
            .tombstone_content(service_content_id.as_str(), &auth)?;
        Ok(artifact_status_from_service(artifact_id, status))
    }

    fn complete_task(&mut self, task_id: &TaskId) -> Result<(), SdkError> {
        self.complete_task_via_service(task_id)
    }

    fn create_escrow(&mut self, escrow: EscrowConfig) -> Result<EscrowId, SdkError> {
        self.create_escrow_via_service(escrow)
    }

    fn release_escrow(&mut self, escrow_id: &EscrowId) -> Result<(), SdkError> {
        self.release_escrow_via_service(escrow_id)
    }

    fn balance(&self, did: &AgentDid) -> Result<TokenAmount, SdkError> {
        let auth = build_agents_read_auth(&self.state, &self.config)?;
        let balance = self.service_client.get_agent_balance(did.as_str(), &auth)?;
        Ok(TokenAmount(balance.balance))
    }

    fn search_agents(&self, query: AgentQuery) -> Result<Vec<AgentSummary>, SdkError> {
        let payload = agent_search_payload(&query)?;
        let auth = build_agents_read_auth_with_body(&self.state, &self.config, payload.as_str())?;
        let profiles = self.service_client.search_agents(&query, &auth)?;
        profiles.into_iter().map(agent_profile_to_summary).collect()
    }
    fn get_reputation(&self, agent: &AgentDid) -> Result<AgentReputation, SdkError> {
        let auth = build_agents_read_auth(&self.state, &self.config)?;
        let profile = self
            .service_client
            .get_agent_profile(agent.as_str(), &auth)?;
        agent_profile_to_reputation(profile)
    }
}

fn resolve_service_content_id(
    client: &LiveTransportKamnClient,
    artifact_id: &ArtifactId,
) -> Result<String, SdkError> {
    let guard = client
        .state
        .lock()
        .map_err(|_| SdkError::TransportFailure("live transport state lock poisoned"))?;
    prepare_artifact_status_lookup(&guard.artifact_ids, artifact_id)
}

fn resolve_service_task_id(
    client: &LiveTransportKamnClient,
    task_id: &TaskId,
) -> Result<String, SdkError> {
    let guard = client
        .state
        .lock()
        .map_err(|_| SdkError::TransportFailure("live transport state lock poisoned"))?;
    prepare_task_status_lookup(&guard.task_aliases, task_id)
}

fn message_read_auth(client: &LiveTransportKamnClient) -> Result<ServiceRequestAuth, SdkError> {
    build_auth(
        &client.state,
        &client.config,
        &client.config.requester_did,
        "",
        Some(MESSAGES_READ_SCOPE),
    )
}

fn message_status_from_service(
    message_id: &MessageId,
    status: ServiceMessageStatus,
) -> MessageStatus {
    MessageStatus::from_status(message_id, status.status.as_str())
}

fn artifact_status_from_service(
    artifact_id: &ArtifactId,
    status: ServiceContentStatus,
) -> ArtifactStatus {
    ArtifactStatus::from_lifecycle(
        artifact_id,
        status.lifecycle_state,
        status.redaction_status,
    )
}
