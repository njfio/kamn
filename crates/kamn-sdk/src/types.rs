use crate::SdkError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentDid(String);

impl AgentDid {
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

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AgentDid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentMetadata {
    pub agent_type: String,
    pub model_family: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DidDocument {
    pub id: AgentDid,
    pub metadata: AgentMetadata,
    pub service_endpoint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChannelId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub from: AgentDid,
    pub to: AgentDid,
    pub body: String,
    pub channel: Option<ChannelId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageRecord {
    pub id: MessageId,
    pub message: Message,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TaskId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskDefinition {
    pub creator: AgentDid,
    pub task_type: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArtifactId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    pub name: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenAmount(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EscrowId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EscrowConfig {
    pub payer: AgentDid,
    pub payee: AgentDid,
    pub amount: TokenAmount,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AgentQuery {
    pub capability: Option<String>,
    pub model_family: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSummary {
    pub did: AgentDid,
    pub agent_type: String,
    pub model_family: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentReputation {
    pub did: AgentDid,
    pub score: u32,
}
