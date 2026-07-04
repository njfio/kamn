/// Shared canonical agent DID value.
///
/// Agent DID strings must use the `kamn:did:agent:` prefix.
pub use kamn_types::AgentDid;

/// Agent registration metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentMetadata {
    /// Logical agent type/classification.
    pub agent_type: String,
    /// Underlying model family name.
    pub model_family: String,
    /// Capability labels exposed by the agent.
    pub capabilities: Vec<String>,
}

/// Minimal DID document returned by SDK resolver calls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DidDocument {
    /// DID identifier.
    pub id: AgentDid,
    /// Agent metadata associated with the DID.
    pub metadata: AgentMetadata,
    /// Service endpoint for messaging and operations.
    pub service_endpoint: String,
}

/// Message identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageId(pub u64);

/// Channel identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChannelId(pub String);

/// Agent-to-agent message payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    /// Sender DID.
    pub from: AgentDid,
    /// Recipient DID.
    pub to: AgentDid,
    /// Message body content.
    pub body: String,
    /// Optional channel scope for the message.
    pub channel: Option<ChannelId>,
}

/// Stored message record with assigned identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageRecord {
    /// Message identifier.
    pub id: MessageId,
    /// Message payload.
    pub message: Message,
}

/// Lifecycle view for a previously sent message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageStatus {
    /// SDK message identifier.
    pub message_id: MessageId,
    /// Lifecycle state reported by the transport.
    pub status: String,
}

impl MessageStatus {
    pub(crate) fn from_status(message_id: &MessageId, status: &str) -> Self {
        Self {
            message_id: message_id.clone(),
            status: status.to_owned(),
        }
    }
}

/// Iterator wrapper for streamed message retrieval.
#[derive(Debug)]
pub struct MessageStream {
    records: std::vec::IntoIter<MessageRecord>,
}

impl MessageStream {
    /// Creates a new message stream from buffered records.
    pub fn new(records: Vec<MessageRecord>) -> Self {
        Self {
            records: records.into_iter(),
        }
    }
}

impl Iterator for MessageStream {
    type Item = MessageRecord;

    fn next(&mut self) -> Option<Self::Item> {
        self.records.next()
    }
}

/// Task identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TaskId(pub u64);

/// Task creation payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskDefinition {
    /// DID creating the task.
    pub creator: AgentDid,
    /// Task type label.
    pub task_type: String,
    /// Human-readable task description.
    pub description: String,
}

/// Lifecycle view for a previously created task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskStatus {
    /// SDK task identifier.
    pub task_id: TaskId,
    /// Lifecycle state reported by the transport.
    pub state: String,
}

impl TaskStatus {
    pub(crate) fn from_state(task_id: &TaskId, state: &str) -> Self {
        Self {
            task_id: task_id.clone(),
            state: state.to_owned(),
        }
    }
}

/// Artifact identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArtifactId(pub u64);

/// Task artifact payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    /// Artifact name.
    pub name: String,
    /// Artifact bytes.
    pub bytes: Vec<u8>,
}

/// Lifecycle view for a previously submitted artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactStatus {
    /// SDK artifact identifier.
    pub artifact_id: ArtifactId,
    /// Lifecycle state reported by the transport.
    pub lifecycle_state: String,
    /// Redaction status reported by the transport.
    pub redaction_status: String,
}

impl ArtifactStatus {
    pub(crate) fn retained(artifact_id: &ArtifactId) -> Self {
        Self::from_lifecycle(artifact_id, "retained".to_owned(), "none".to_owned())
    }

    pub(crate) fn from_lifecycle(
        artifact_id: &ArtifactId,
        lifecycle_state: String,
        redaction_status: String,
    ) -> Self {
        Self {
            artifact_id: artifact_id.clone(),
            lifecycle_state,
            redaction_status,
        }
    }
}

/// Token amount wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenAmount(pub u64);

/// Escrow identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EscrowId(pub u64);

/// Escrow creation payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EscrowConfig {
    /// DID paying into escrow.
    pub payer: AgentDid,
    /// DID receiving escrow payout.
    pub payee: AgentDid,
    /// Escrow amount.
    pub amount: TokenAmount,
}

/// Query filters for agent discovery.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AgentQuery {
    /// Optional required capability.
    pub capability: Option<String>,
    /// Optional model-family filter.
    pub model_family: Option<String>,
}

/// Search result summary for a registered agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSummary {
    /// Agent DID.
    pub did: AgentDid,
    /// Agent type label.
    pub agent_type: String,
    /// Model family label.
    pub model_family: String,
    /// Declared capabilities.
    pub capabilities: Vec<String>,
}

/// Simplified agent reputation view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentReputation {
    /// Agent DID.
    pub did: AgentDid,
    /// Reputation score in the 0..=1000 range.
    pub score: u32,
}
