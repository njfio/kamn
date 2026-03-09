#![warn(missing_docs)]
//! Rust SDK surface for interacting with KAMN transports and domain types.

mod bridge;
mod channel_create;
mod events;
mod observability;

/// Agent-facing traits and transport mode primitives.
pub mod agent;
/// Shared SDK error types.
pub mod error;
/// Live transport adapter backed by networked Service API routes.
pub mod live;
/// In-memory reference client used for deterministic local flows and tests.
pub mod memory;
/// Service HTTP and websocket client for runtime API routes.
pub mod service;
/// TCP relay transport adapter and cryptographic wire envelope models.
pub mod tcp;
/// Core data structures used by agent, task, messaging, and escrow APIs.
pub mod types;

/// Re-exported agent traits and transport mode enum.
pub use agent::{KamnAgent, KamnTransport, TransportMode};
/// Re-exported bridge lifecycle types.
pub use bridge::{BridgeId, BridgeStatus};
/// Re-exported SDK error type.
pub use error::SdkError;
/// Re-exported service events types.
pub use events::{KamnServiceEvents, ServiceEventSnapshot};
/// Re-exported live transport client and configuration.
pub use live::{LiveTransportConfig, LiveTransportKamnClient};
/// Re-exported in-memory client.
pub use memory::InMemoryKamnClient;
/// Re-exported service observability types.
pub use observability::{KamnServiceObservability, ServiceHealthSnapshot};
/// Re-exported service API client primitives.
pub use service::{
    service_public_key_for_private_key, service_signature_for_fields,
    service_signature_for_state_hash_with_private_key, service_signer_public_key_for_fields,
    service_verify_signature_with_public_key, ServiceAgentBalance, ServiceAgentProfile,
    ServiceApiClient, ServiceBridgeStatus, ServiceBridgeSubmission, ServiceChannelMessages,
    ServiceChannelReceipt, ServiceContentRegistration, ServiceContentStatus, ServiceEscrowStatus,
    ServiceHealthStatus, ServiceMessageReceipt, ServiceMessageStatus, ServiceRequestAuth,
    ServiceRouteEvent, ServiceTaskReceipt, ServiceTaskStatus,
};
/// Re-exported TCP relay adapter and envelope helpers.
pub use tcp::{
    signature_for_fields, TcpReceivedEnvelope, TcpSignedEnvelope, TcpTransportAdapter,
    TcpTransportConfig,
};
/// Re-exported core domain data types.
pub use types::{
    AgentDid, AgentMetadata, AgentQuery, AgentReputation, AgentSummary, Artifact, ArtifactId,
    ArtifactStatus, ChannelId, DidDocument, EscrowConfig, EscrowId, Message, MessageId,
    MessageRecord, MessageStatus, MessageStream, TaskDefinition, TaskId, TaskStatus, TokenAmount,
};
