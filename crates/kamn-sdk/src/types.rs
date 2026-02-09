use crate::SdkError;

/// Strongly typed agent DID value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentDid(String);

impl AgentDid {
    /// Parses and validates an agent DID string.
    /// Input values must use the `kamn:did:agent:` prefix.
    pub fn parse(value: impl Into<String>) -> Result<Self, SdkError> {
        let value = value.into();
        if !value.starts_with("kamn:did:agent:") {
            return Err(SdkError::InvalidInput {
                field: "did",
                reason: "must start with kamn:did:agent:",
            });
        }
        if value.trim().len() <= "kamn:did:agent:".len() {
            return Err(SdkError::InvalidInput {
                field: "did",
                reason: "method specific identifier is required",
            });
        }
        Ok(Self(value))
    }

    /// Returns the DID string view.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AgentDid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
