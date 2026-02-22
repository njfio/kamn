#![warn(missing_docs)]
//! Rust SDK surface for interacting with KAMN transports and domain types.

/// Agent-facing traits and transport mode primitives.
pub mod agent;
/// Shared SDK error types.
pub mod error;
/// Live transport adapter backed by endpoint-keyed shared in-memory state.
pub mod live;
/// In-memory reference client used for deterministic local flows and tests.
pub mod memory;
/// Service HTTP and websocket client for runtime API routes.
pub mod service;
/// TCP relay transport adapter and deterministic wire envelope models.
pub mod tcp;
/// Core data structures used by agent, task, messaging, and escrow APIs.
pub mod types;

/// Re-exported agent traits and transport mode enum.
pub use agent::{KamnAgent, KamnTransport, TransportMode};
/// Re-exported SDK error type.
pub use error::SdkError;
/// Re-exported live transport client and configuration.
pub use live::{LiveTransportConfig, LiveTransportKamnClient};
/// Re-exported in-memory client.
pub use memory::InMemoryKamnClient;
/// Re-exported service API client primitives.
pub use service::{
    service_signature_for_fields, ServiceAgentProfile, ServiceApiClient, ServiceChannelMessages,
    ServiceChannelReceipt, ServiceHealthStatus, ServiceMessageReceipt, ServiceMessageStatus,
    ServiceRequestAuth, ServiceRouteEvent, ServiceTaskReceipt, ServiceTaskStatus,
};
/// Re-exported TCP relay adapter and envelope helpers.
pub use tcp::{
    signature_for_fields, TcpReceivedEnvelope, TcpSignedEnvelope, TcpTransportAdapter,
    TcpTransportConfig,
};
/// Re-exported core domain data types.
pub use types::{
    AgentDid, AgentMetadata, AgentQuery, AgentReputation, AgentSummary, Artifact, ArtifactId,
    ChannelId, DidDocument, EscrowConfig, EscrowId, Message, MessageId, MessageRecord,
    MessageStream, TaskDefinition, TaskId, TokenAmount,
};
