use crate::{
    AgentDid, AgentMetadata, AgentQuery, AgentReputation, AgentSummary, Artifact, ArtifactId,
    ArtifactStatus, BridgeId, BridgeStatus, ChannelId, DidDocument, EscrowConfig, EscrowId,
    Message, MessageId, MessageRecord, MessageStatus, MessageStream, SdkError,
    ServiceHealthSnapshot, TaskDefinition, TaskId, TaskStatus, TokenAmount,
};

/// Transport backends available for SDK clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportMode {
    /// Pure in-memory transport mode.
    InMemory,
    /// Endpoint-backed live transport mode.
    Live,
}

impl TransportMode {
    /// Returns a stable human-readable identifier for this transport mode.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InMemory => "in-memory",
            Self::Live => "live",
        }
    }
}

/// Common transport capabilities shared by all SDK clients.
pub trait KamnTransport {
    /// Returns the concrete transport mode used by this client.
    fn transport_mode(&self) -> TransportMode;

    /// Validates that the client matches the expected transport mode.
    fn assert_transport_mode(&self, expected: TransportMode) -> Result<(), SdkError> {
        let found = self.transport_mode();
        if found == expected {
            return Ok(());
        }

        Err(SdkError::TransportModeMismatch {
            expected: expected.as_str(),
            found: found.as_str(),
        })
    }
}

/// Public service observability capabilities exposed by SDK transports.
pub trait KamnServiceObservability {
    /// Returns a stable SDK snapshot of the service health route.
    fn service_health(&self) -> Result<ServiceHealthSnapshot, SdkError>;

    /// Returns the raw `/metrics` exposition text.
    fn service_metrics(&self) -> Result<String, SdkError>;
}

/// High-level KAMN agent workflow API.
pub trait KamnAgent {
    /// Registers an agent and returns its DID.
    fn register(&mut self, metadata: AgentMetadata) -> Result<AgentDid, SdkError>;
    /// Resolves an agent DID to a DID document.
    fn resolve(&self, did: &AgentDid) -> Result<DidDocument, SdkError>;

    /// Sends a message and returns the resulting message identifier.
    fn send(&mut self, message: Message) -> Result<MessageId, SdkError>;
    /// Returns lifecycle metadata for a previously sent message.
    fn get_message_status(&self, message_id: &MessageId) -> Result<MessageStatus, SdkError>;
    /// Submits one message for bridge processing.
    fn submit_bridge(
        &mut self,
        source_message_id: &MessageId,
        target_network: &str,
    ) -> Result<BridgeStatus, SdkError>;
    /// Forwards a previously submitted bridge.
    fn forward_bridge(&mut self, bridge_id: &BridgeId) -> Result<BridgeStatus, SdkError>;
    /// Returns lifecycle metadata for a previously submitted bridge.
    fn get_bridge_status(&self, bridge_id: &BridgeId) -> Result<BridgeStatus, SdkError>;
    /// Receives and drains pending messages for the given DID.
    fn receive(&mut self, did: &AgentDid) -> Result<Vec<MessageRecord>, SdkError>;
    /// Receives pending messages as an iterator stream abstraction.
    fn receive_stream(&mut self, did: &AgentDid) -> Result<MessageStream, SdkError>;
    /// Creates a channel and returns its channel identifier.
    fn create_channel(&mut self, name: &str) -> Result<ChannelId, SdkError>;

    /// Creates a task definition and returns its task identifier.
    fn create_task(&mut self, task: TaskDefinition) -> Result<TaskId, SdkError>;
    /// Accepts a task on behalf of an assignee DID.
    fn accept_task(&mut self, task_id: &TaskId, assignee: &AgentDid) -> Result<(), SdkError>;
    /// Returns lifecycle metadata for a previously created task.
    fn get_task_status(&self, task_id: &TaskId) -> Result<TaskStatus, SdkError>;
    /// Submits a task artifact and returns its artifact identifier.
    fn submit_artifact(
        &mut self,
        task_id: &TaskId,
        artifact: Artifact,
    ) -> Result<ArtifactId, SdkError>;
    /// Returns lifecycle metadata for a previously submitted artifact.
    fn get_artifact_status(&self, artifact_id: &ArtifactId) -> Result<ArtifactStatus, SdkError>;
    /// Expires a previously submitted artifact and returns updated lifecycle metadata.
    fn expire_artifact(&mut self, artifact_id: &ArtifactId) -> Result<ArtifactStatus, SdkError>;
    /// Tombstones a previously submitted artifact and returns updated lifecycle metadata.
    fn tombstone_artifact(&mut self, artifact_id: &ArtifactId) -> Result<ArtifactStatus, SdkError>;
    /// Marks a task as completed.
    fn complete_task(&mut self, task_id: &TaskId) -> Result<(), SdkError>;

    /// Creates an escrow configuration and returns its escrow identifier.
    fn create_escrow(&mut self, escrow: EscrowConfig) -> Result<EscrowId, SdkError>;
    /// Releases a previously created escrow.
    fn release_escrow(&mut self, escrow_id: &EscrowId) -> Result<(), SdkError>;
    /// Returns the token balance for the provided DID.
    fn balance(&self, did: &AgentDid) -> Result<TokenAmount, SdkError>;

    /// Searches agents matching the supplied query filters.
    fn search_agents(&self, query: AgentQuery) -> Result<Vec<AgentSummary>, SdkError>;
    /// Returns reputation information for the provided DID.
    fn get_reputation(&self, agent: &AgentDid) -> Result<AgentReputation, SdkError>;
}
