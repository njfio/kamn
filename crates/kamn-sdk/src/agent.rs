use crate::{
    AgentDid, AgentMetadata, AgentQuery, AgentReputation, AgentSummary, Artifact, ArtifactId,
    DidDocument, EscrowConfig, EscrowId, Message, MessageId, MessageRecord, MessageStream,
    SdkError, TaskDefinition, TaskId, TokenAmount,
};

pub trait KamnAgent {
    fn register(&mut self, metadata: AgentMetadata) -> Result<AgentDid, SdkError>;
    fn resolve(&self, did: &AgentDid) -> Result<DidDocument, SdkError>;

    fn send(&mut self, message: Message) -> Result<MessageId, SdkError>;
    fn receive(&mut self, did: &AgentDid) -> Result<Vec<MessageRecord>, SdkError>;
    fn receive_stream(&mut self, did: &AgentDid) -> Result<MessageStream, SdkError>;

    fn create_task(&mut self, task: TaskDefinition) -> Result<TaskId, SdkError>;
    fn accept_task(&mut self, task_id: &TaskId, assignee: &AgentDid) -> Result<(), SdkError>;
    fn submit_artifact(
        &mut self,
        task_id: &TaskId,
        artifact: Artifact,
    ) -> Result<ArtifactId, SdkError>;
    fn complete_task(&mut self, task_id: &TaskId) -> Result<(), SdkError>;

    fn create_escrow(&mut self, escrow: EscrowConfig) -> Result<EscrowId, SdkError>;
    fn release_escrow(&mut self, escrow_id: &EscrowId) -> Result<(), SdkError>;
    fn balance(&self, did: &AgentDid) -> Result<TokenAmount, SdkError>;

    fn search_agents(&self, query: AgentQuery) -> Result<Vec<AgentSummary>, SdkError>;
    fn get_reputation(&self, agent: &AgentDid) -> Result<AgentReputation, SdkError>;
}
