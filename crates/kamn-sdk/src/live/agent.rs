use super::{
    config::{AGENTS_READ_SCOPE, ESCROW_WRITE_SCOPE, MESSAGES_WRITE_SCOPE, TASKS_WRITE_SCOPE},
    routes::{agent_profile_to_document, agent_profile_to_reputation, service_message_payload},
    state::{build_auth, remember_message_id},
    task_escrow::{
        escrow_payload, prepare_escrow_release, prepare_task_accept, prepare_task_complete,
        remember_escrow_alias, remember_task_alias, task_payload,
    },
    LiveTransportKamnClient,
};
use crate::{
    AgentDid, AgentMetadata, AgentQuery, AgentReputation, AgentSummary, Artifact, ArtifactId,
    DidDocument, EscrowConfig, EscrowId, KamnAgent, KamnTransport, Message, MessageId,
    MessageRecord, MessageStream, SdkError, TaskDefinition, TaskId, TokenAmount, TransportMode,
};

impl KamnTransport for LiveTransportKamnClient {
    fn transport_mode(&self) -> TransportMode {
        TransportMode::Live
    }
}
impl KamnAgent for LiveTransportKamnClient {
    fn register(&mut self, _metadata: AgentMetadata) -> Result<AgentDid, SdkError> {
        Self::unsupported("live transport register route is not available via service api")
    }

    fn resolve(&self, did: &AgentDid) -> Result<DidDocument, SdkError> {
        let auth = build_auth(
            &self.state,
            &self.config,
            &self.config.requester_did,
            "",
            Some(AGENTS_READ_SCOPE),
        )?;
        let profile = self.service_client.get_agent_profile(did.as_str(), &auth)?;
        agent_profile_to_document(profile, self.endpoint())
    }

    fn send(&mut self, message: Message) -> Result<MessageId, SdkError> {
        let payload = service_message_payload(&message);
        let auth = build_auth(
            &self.state,
            &self.config,
            &message.from,
            payload.as_str(),
            Some(MESSAGES_WRITE_SCOPE),
        )?;
        let receipt = self.service_client.send_message(payload.as_str(), &auth)?;
        remember_message_id(&self.state, receipt.message_id.as_str())
    }

    fn receive(&mut self, _did: &AgentDid) -> Result<Vec<MessageRecord>, SdkError> {
        Self::unsupported("live transport receive route is not available via service api")
    }

    fn receive_stream(&mut self, _did: &AgentDid) -> Result<MessageStream, SdkError> {
        Self::unsupported("live transport receive route is not available via service api")
    }

    fn create_task(&mut self, task: TaskDefinition) -> Result<TaskId, SdkError> {
        let payload = task_payload(&task)?;
        let auth = build_auth(
            &self.state,
            &self.config,
            &task.creator,
            payload.as_str(),
            Some(TASKS_WRITE_SCOPE),
        )?;
        let receipt = self.service_client.create_task(payload.as_str(), &auth)?;
        let mut guard = self
            .state
            .lock()
            .map_err(|_| SdkError::TransportFailure("live transport state lock poisoned"))?;
        remember_task_alias(
            &mut guard.task_aliases,
            receipt.task_id.as_str(),
            &task.creator,
        )
    }

    fn accept_task(&mut self, task_id: &TaskId, assignee: &AgentDid) -> Result<(), SdkError> {
        let (service_task_id, sender) = {
            let mut guard = self
                .state
                .lock()
                .map_err(|_| SdkError::TransportFailure("live transport state lock poisoned"))?;
            prepare_task_accept(&mut guard.task_aliases, task_id, assignee)?
        };
        let auth = build_auth(
            &self.state,
            &self.config,
            &sender,
            "{}",
            Some(TASKS_WRITE_SCOPE),
        )?;
        self.service_client
            .accept_task(service_task_id.as_str(), &auth)?;
        Ok(())
    }

    fn submit_artifact(
        &mut self,
        _task_id: &TaskId,
        _artifact: Artifact,
    ) -> Result<ArtifactId, SdkError> {
        Self::unsupported(
            "live transport artifact routes are not yet mapped in sdk kamn-agent surface",
        )
    }

    fn complete_task(&mut self, task_id: &TaskId) -> Result<(), SdkError> {
        let (service_task_id, sender) = {
            let guard = self
                .state
                .lock()
                .map_err(|_| SdkError::TransportFailure("live transport state lock poisoned"))?;
            prepare_task_complete(&guard.task_aliases, task_id)?
        };
        let auth = build_auth(
            &self.state,
            &self.config,
            &sender,
            "{}",
            Some(TASKS_WRITE_SCOPE),
        )?;
        self.service_client
            .complete_task(service_task_id.as_str(), &auth)?;
        Ok(())
    }

    fn create_escrow(&mut self, escrow: EscrowConfig) -> Result<EscrowId, SdkError> {
        let payload = escrow_payload(&escrow)?;
        let auth = build_auth(
            &self.state,
            &self.config,
            &escrow.payer,
            payload.as_str(),
            Some(ESCROW_WRITE_SCOPE),
        )?;
        let receipt = self.service_client.fund_escrow(payload.as_str(), &auth)?;
        let mut guard = self
            .state
            .lock()
            .map_err(|_| SdkError::TransportFailure("live transport state lock poisoned"))?;
        remember_escrow_alias(
            &mut guard.escrow_aliases,
            receipt.escrow_id.as_str(),
            &escrow.payer,
        )
    }

    fn release_escrow(&mut self, escrow_id: &EscrowId) -> Result<(), SdkError> {
        let (service_escrow_id, sender) = {
            let guard = self
                .state
                .lock()
                .map_err(|_| SdkError::TransportFailure("live transport state lock poisoned"))?;
            prepare_escrow_release(&guard.escrow_aliases, escrow_id)?
        };
        let auth = build_auth(
            &self.state,
            &self.config,
            &sender,
            "{}",
            Some(ESCROW_WRITE_SCOPE),
        )?;
        self.service_client
            .release_escrow(service_escrow_id.as_str(), &auth)?;
        Ok(())
    }

    fn balance(&self, did: &AgentDid) -> Result<TokenAmount, SdkError> {
        let auth = build_auth(
            &self.state,
            &self.config,
            &self.config.requester_did,
            "",
            Some(AGENTS_READ_SCOPE),
        )?;
        let balance = self.service_client.get_agent_balance(did.as_str(), &auth)?;
        Ok(TokenAmount(balance.balance))
    }

    fn search_agents(&self, _query: AgentQuery) -> Result<Vec<AgentSummary>, SdkError> {
        Self::unsupported("live transport agent search route is not available via service api")
    }
    fn get_reputation(&self, agent: &AgentDid) -> Result<AgentReputation, SdkError> {
        let auth = build_auth(
            &self.state,
            &self.config,
            &self.config.requester_did,
            "",
            Some(AGENTS_READ_SCOPE),
        )?;
        let profile = self
            .service_client
            .get_agent_profile(agent.as_str(), &auth)?;
        agent_profile_to_reputation(profile)
    }
}
