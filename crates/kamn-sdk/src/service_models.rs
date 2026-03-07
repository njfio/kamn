/// Parsed response for `POST /v1/messages/send`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceMessageReceipt {
    /// Service-generated message identifier.
    pub message_id: String,
    /// Lifecycle status marker.
    pub status: String,
    /// Runtime mode reported by the service.
    pub runtime_mode: String,
}

/// Parsed response for `GET /v1/messages/{id}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceMessageStatus {
    /// Message identifier.
    pub message_id: String,
    /// Lifecycle status marker.
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceMessageDelivery {
    pub(crate) message_id: String,
    pub(crate) sender_did: String,
    pub(crate) recipient_did: String,
    pub(crate) body: String,
}

/// Parsed response for `POST /v1/channels/create`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceChannelReceipt {
    /// Service-generated channel identifier.
    pub channel_id: String,
    /// Route status marker.
    pub status: String,
}

/// Parsed response for `GET /v1/channels/{id}/messages`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceChannelMessages {
    /// Channel identifier.
    pub channel_id: String,
    /// Message identifiers observed in the channel.
    pub messages: Vec<String>,
}

/// Parsed response for `POST /v1/tasks/create`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceTaskReceipt {
    /// Service-generated task identifier.
    pub task_id: String,
    /// Initial lifecycle state.
    pub state: String,
}

/// Parsed response for `GET /v1/tasks/{id}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceTaskStatus {
    /// Task identifier.
    pub task_id: String,
    /// Current lifecycle state.
    pub state: String,
}

/// Parsed response for escrow lifecycle routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceEscrowStatus {
    /// Escrow identifier.
    pub escrow_id: String,
    /// Current lifecycle state.
    pub state: String,
}

/// Parsed response for `POST /v1/content/register`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceContentRegistration {
    /// Service-generated content identifier.
    pub content_id: String,
    /// Retention class marker.
    pub retention_class: String,
    /// Lifecycle state marker.
    pub lifecycle_state: String,
    /// Redaction status marker.
    pub redaction_status: String,
}

/// Parsed response for content lifecycle routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceContentStatus {
    /// Content identifier.
    pub content_id: String,
    /// Lifecycle state marker.
    pub lifecycle_state: String,
    /// Redaction status marker.
    pub redaction_status: String,
}

/// Parsed response for `POST /v1/bridge/submit`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceBridgeSubmission {
    /// Bridge identifier.
    pub bridge_id: String,
    /// Source message identifier.
    pub source_message_id: String,
    /// Bridge lifecycle status marker.
    pub bridge_status: String,
}

/// Parsed response for bridge forward/query routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceBridgeStatus {
    /// Bridge identifier.
    pub bridge_id: String,
    /// Bridge lifecycle status marker.
    pub bridge_status: String,
    /// Target message identifier after forwarding.
    pub target_message_id: String,
    /// Forward transaction hash marker.
    pub forward_tx_hash: String,
}

/// Parsed response for `GET /v1/agents/{did}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceAgentProfile {
    /// Agent DID.
    pub did: String,
    /// Current reputation score.
    pub reputation_score: u64,
}

/// Parsed response for `GET /v1/agents/{did}/balance`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceAgentBalance {
    /// Agent DID.
    pub did: String,
    /// Current token balance.
    pub balance: u64,
}

/// Parsed response for `GET /healthz`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceHealthStatus {
    /// Service health marker.
    pub status: String,
    /// Runtime mode marker.
    pub runtime_mode: String,
    /// Node role marker.
    pub role: String,
    /// Observability source marker.
    pub observability_source: String,
    /// Observability health marker.
    pub observability_health: String,
}

/// Parsed event frame payload from `GET /v1/events/ws`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceRouteEvent {
    /// Event name.
    pub event: String,
    /// Runtime mode marker.
    pub runtime_mode: String,
    /// Node role marker.
    pub role: String,
    /// Event sequence identifier.
    pub sequence: u64,
}
