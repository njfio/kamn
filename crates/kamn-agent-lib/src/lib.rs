#![warn(missing_docs)]
//! Shared phase-1 agent library for authenticated KAMN operations.

use std::sync::Mutex;

use auth::KamnAuthHeaders;
use client::ServiceApiHttpClient;
use envelope::{build_and_sign_envelope, CanonicalMessageEnvelope};
use kolme::KolmeClient;
use nonce::{NonceTracker, NonceTrackerError};

pub use errors::AgentLibError;
pub use identity::AgentIdentity;
pub use kolme::{KolmeProofReceipt, KolmeProofVerification};

pub use kamn_sdk::{
    ServiceAgentProfile, ServiceBridgeStatus, ServiceBridgeSubmission, ServiceChannelMessages,
    ServiceChannelReceipt, ServiceContentRegistration, ServiceContentStatus, ServiceEscrowStatus,
    ServiceHealthStatus, ServiceMessageReceipt, ServiceMessageStatus, ServiceTaskReceipt,
    ServiceTaskStatus,
};

/// Authentication helpers.
pub mod auth;
/// Typed service client facade.
pub mod client;
/// Envelope construction helpers.
pub mod envelope;
/// Error taxonomy.
pub mod errors;
/// Identity utilities.
pub mod identity;
/// Kolme proof verification adapter.
pub mod kolme;
/// Monotonic nonce tracking.
pub mod nonce;

/// Top-level phase-1 facade for authenticated KAMN operations.
#[derive(Debug)]
pub struct KamnAgentHandle {
    identity: AgentIdentity,
    service_client: ServiceApiHttpClient,
    kolme_client: KolmeClient,
    nonce_tracker: Mutex<NonceTracker>,
}

impl KamnAgentHandle {
    /// Connects an agent handle using deterministic identity derivation.
    pub fn connect(
        service_endpoint: &str,
        kolme_endpoint: &str,
        agent_name: &str,
    ) -> Result<Self, AgentLibError> {
        let identity = AgentIdentity::from_agent_name(agent_name)?;
        Self::with_identity(service_endpoint, kolme_endpoint, identity)
    }

    /// Connects an agent handle using explicit identity material.
    pub fn with_identity(
        service_endpoint: &str,
        kolme_endpoint: &str,
        identity: AgentIdentity,
    ) -> Result<Self, AgentLibError> {
        Ok(Self {
            identity,
            service_client: ServiceApiHttpClient::connect(service_endpoint)?,
            kolme_client: KolmeClient::new(kolme_endpoint)?,
            nonce_tracker: Mutex::new(NonceTracker::new(0)),
        })
    }

    /// Returns bound identity material.
    pub fn identity(&self) -> &AgentIdentity {
        &self.identity
    }

    /// Builds deterministic auth headers with an incremented nonce.
    pub fn build_auth_headers(
        &self,
        state_hash: &str,
        body: &[u8],
        authz_scope: Option<&str>,
    ) -> Result<KamnAuthHeaders, AgentLibError> {
        let nonce = self.next_nonce()?;
        KamnAuthHeaders::build(
            self.identity.did().as_str(),
            self.identity.signing_key(),
            nonce,
            state_hash,
            body,
            authz_scope,
        )
    }

    /// Builds a canonical signed message envelope with an incremented nonce.
    pub fn build_envelope(
        &self,
        to: &str,
        state_hash: &str,
        body: &str,
    ) -> Result<CanonicalMessageEnvelope, AgentLibError> {
        let nonce = self.next_nonce()?;
        build_and_sign_envelope(&self.identity, to, state_hash, nonce, body)
    }

    /// Sends a message through the service API.
    pub fn send_message(&self, payload: &str) -> Result<ServiceMessageReceipt, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self.service_client.build_auth(
            self.identity.did(),
            self.identity.signing_key(),
            nonce,
            payload,
            Some("messages:write"),
        )?;
        self.service_client.send_message(payload, &auth)
    }

    /// Queries a message status by identifier.
    pub fn query_message(&self, message_id: &str) -> Result<ServiceMessageStatus, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self.service_client.build_auth(
            self.identity.did(),
            self.identity.signing_key(),
            nonce,
            "",
            Some("messages:read"),
        )?;
        self.service_client.get_message(message_id, &auth)
    }

    /// Creates a channel through the service API.
    pub fn create_channel(&self, payload: &str) -> Result<ServiceChannelReceipt, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self.service_client.build_auth(
            self.identity.did(),
            self.identity.signing_key(),
            nonce,
            payload,
            Some("channels:write"),
        )?;
        self.service_client.create_channel(payload, &auth)
    }

    /// Creates a task through the service API.
    pub fn create_task(&self, payload: &str) -> Result<ServiceTaskReceipt, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self.service_client.build_auth(
            self.identity.did(),
            self.identity.signing_key(),
            nonce,
            payload,
            Some("tasks:write"),
        )?;
        self.service_client.create_task(payload, &auth)
    }

    /// Queries task status by identifier.
    pub fn query_task(&self, task_id: &str) -> Result<ServiceTaskStatus, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self.service_client.build_auth(
            self.identity.did(),
            self.identity.signing_key(),
            nonce,
            "",
            Some("tasks:read"),
        )?;
        self.service_client.get_task(task_id, &auth)
    }

    /// Queries agent profile by DID.
    pub fn query_agent_profile(&self, did: &str) -> Result<ServiceAgentProfile, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self.service_client.build_auth(
            self.identity.did(),
            self.identity.signing_key(),
            nonce,
            "",
            Some("agents:read"),
        )?;
        self.service_client.get_agent_profile(did, &auth)
    }

    /// Queries service health.
    pub fn health(&self) -> Result<ServiceHealthStatus, AgentLibError> {
        self.service_client.health()
    }

    /// Verifies one Kolme proof receipt.
    pub fn verify_proof(
        &self,
        message_id: &str,
        receipt: &KolmeProofReceipt,
    ) -> Result<KolmeProofVerification, AgentLibError> {
        self.kolme_client.verify_proof(message_id, receipt)
    }

    /// Accepts one task through the service API.
    pub fn accept_task(&self, task_id: &str) -> Result<ServiceTaskStatus, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self.service_client.build_auth(
            self.identity.did(),
            self.identity.signing_key(),
            nonce,
            "{}",
            Some("tasks:write"),
        )?;
        self.service_client.accept_task(task_id, &auth)
    }

    /// Completes one task through the service API.
    pub fn complete_task(&self, task_id: &str) -> Result<ServiceTaskStatus, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self.service_client.build_auth(
            self.identity.did(),
            self.identity.signing_key(),
            nonce,
            "{}",
            Some("tasks:write"),
        )?;
        self.service_client.complete_task(task_id, &auth)
    }

    /// Funds escrow through the service API.
    pub fn fund_escrow(&self, payload: &str) -> Result<ServiceEscrowStatus, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self.service_client.build_auth(
            self.identity.did(),
            self.identity.signing_key(),
            nonce,
            payload,
            Some("escrow:write"),
        )?;
        self.service_client.fund_escrow(payload, &auth)
    }

    /// Releases one escrow through the service API.
    pub fn release_escrow(&self, escrow_id: &str) -> Result<ServiceEscrowStatus, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self.service_client.build_auth(
            self.identity.did(),
            self.identity.signing_key(),
            nonce,
            "{}",
            Some("escrow:write"),
        )?;
        self.service_client.release_escrow(escrow_id, &auth)
    }

    /// Registers content lifecycle state through the service API.
    pub fn register_content(
        &self,
        payload: &str,
    ) -> Result<ServiceContentRegistration, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self.service_client.build_auth(
            self.identity.did(),
            self.identity.signing_key(),
            nonce,
            payload,
            Some("content:write"),
        )?;
        self.service_client.register_content(payload, &auth)
    }

    /// Expires one content record through the service API.
    pub fn expire_content(&self, content_id: &str) -> Result<ServiceContentStatus, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self.service_client.build_auth(
            self.identity.did(),
            self.identity.signing_key(),
            nonce,
            "{}",
            Some("content:write"),
        )?;
        self.service_client.expire_content(content_id, &auth)
    }

    /// Tombstones one content record through the service API.
    pub fn tombstone_content(
        &self,
        content_id: &str,
    ) -> Result<ServiceContentStatus, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self.service_client.build_auth(
            self.identity.did(),
            self.identity.signing_key(),
            nonce,
            "{}",
            Some("content:write"),
        )?;
        self.service_client.tombstone_content(content_id, &auth)
    }

    /// Queries one content lifecycle record through the service API.
    pub fn query_content(&self, content_id: &str) -> Result<ServiceContentStatus, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self.service_client.build_auth(
            self.identity.did(),
            self.identity.signing_key(),
            nonce,
            "",
            Some("content:read"),
        )?;
        self.service_client.get_content(content_id, &auth)
    }

    /// Submits one bridge message through the service API.
    pub fn submit_bridge_message(
        &self,
        payload: &str,
    ) -> Result<ServiceBridgeSubmission, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self.service_client.build_auth(
            self.identity.did(),
            self.identity.signing_key(),
            nonce,
            payload,
            Some("bridge:write"),
        )?;
        self.service_client.submit_bridge_message(payload, &auth)
    }

    /// Forwards one bridge message through the service API.
    pub fn forward_bridge_message(
        &self,
        bridge_id: &str,
    ) -> Result<ServiceBridgeStatus, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self.service_client.build_auth(
            self.identity.did(),
            self.identity.signing_key(),
            nonce,
            "{}",
            Some("bridge:write"),
        )?;
        self.service_client.forward_bridge_message(bridge_id, &auth)
    }

    /// Queries one bridge message forwarding state through the service API.
    pub fn query_bridge_message(
        &self,
        bridge_id: &str,
    ) -> Result<ServiceBridgeStatus, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self.service_client.build_auth(
            self.identity.did(),
            self.identity.signing_key(),
            nonce,
            "",
            Some("bridge:read"),
        )?;
        self.service_client.get_bridge_message(bridge_id, &auth)
    }

    /// Lists channel messages through the service API.
    pub fn list_messages(&self, channel_id: &str) -> Result<ServiceChannelMessages, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self.service_client.build_auth(
            self.identity.did(),
            self.identity.signing_key(),
            nonce,
            "",
            Some("channels:read"),
        )?;
        self.service_client.list_channel_messages(channel_id, &auth)
    }

    fn next_nonce(&self) -> Result<u64, AgentLibError> {
        let mut tracker = self
            .nonce_tracker
            .lock()
            .map_err(|error| AgentLibError::Internal(format!("nonce lock poisoned: {error}")))?;
        tracker.next_nonce().map_err(|error| match error {
            NonceTrackerError::Exhausted => AgentLibError::InvalidInput {
                field: "nonce",
                reason: "nonce tracker exhausted at u64::MAX".to_owned(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{AgentIdentity, AgentLibError, KamnAgentHandle, NonceTracker};

    #[test]
    fn unit_kamn_agent_handle_exposes_identity_after_connect() {
        let identity = AgentIdentity::from_agent_name("alice").expect("identity");
        let expected_did = identity.did().clone();
        let handle = KamnAgentHandle::with_identity(
            "http://localhost:8080",
            "http://localhost:3000",
            identity,
        )
        .expect("handle");

        assert_eq!(handle.identity().did(), &expected_did);
    }

    #[test]
    fn regression_kamn_agent_handle_rejects_nonce_overflow() {
        // Regression: #5907
        let identity = AgentIdentity::from_agent_name("overflow-agent").expect("identity");
        let handle = KamnAgentHandle::with_identity(
            "http://localhost:8080",
            "http://localhost:3000",
            identity,
        )
        .expect("handle");

        {
            let mut tracker = handle.nonce_tracker.lock().expect("nonce lock");
            *tracker = NonceTracker::new(u64::MAX);
        }

        let error = handle
            .build_auth_headers("state:overflow", b"{}", Some("messages:write"))
            .expect_err("nonce overflow must fail closed");
        assert_eq!(
            error,
            AgentLibError::InvalidInput {
                field: "nonce",
                reason: "nonce tracker exhausted at u64::MAX".to_owned(),
            }
        );
    }
}
