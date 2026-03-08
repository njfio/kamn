use super::{
    LiveTransportKamnClient,
    routes::{
        agent_profile_to_document, agent_profile_to_reputation, recipient_mailbox_channel_id,
        service_message_to_record,
    },
    state::{build_agents_read_auth, remember_message_id},
};
use crate::{
    AgentDid, AgentMetadata, AgentQuery, AgentReputation, AgentSummary, Artifact, ArtifactId,
    DidDocument, EscrowConfig, EscrowId, KamnAgent, Message, MessageId, MessageRecord,
    MessageStream, SdkError, TaskDefinition, TaskId, TokenAmount,
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

    fn create_task(&mut self, task: TaskDefinition) -> Result<TaskId, SdkError> {
        self.create_task_via_service(task)
    }

    fn accept_task(&mut self, task_id: &TaskId, assignee: &AgentDid) -> Result<(), SdkError> {
        self.accept_task_via_service(task_id, assignee)
    }

    fn submit_artifact(
        &mut self,
        task_id: &TaskId,
        artifact: Artifact,
    ) -> Result<ArtifactId, SdkError> {
        self.submit_artifact_via_service(task_id, artifact)
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

    fn search_agents(&self, _query: AgentQuery) -> Result<Vec<AgentSummary>, SdkError> {
        Self::unsupported("live transport agent search route is not available via service api")
    }
    fn get_reputation(&self, agent: &AgentDid) -> Result<AgentReputation, SdkError> {
        let auth = build_agents_read_auth(&self.state, &self.config)?;
        let profile = self
            .service_client
            .get_agent_profile(agent.as_str(), &auth)?;
        agent_profile_to_reputation(profile)
    }
}
