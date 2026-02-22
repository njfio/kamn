#![warn(missing_docs)]
//! Shared phase-1 agent library for authenticated KAMN operations.

use std::sync::Mutex;

use auth::KamnAuthHeaders;
use client::ServiceApiHttpClient;
use envelope::{build_and_sign_envelope, CanonicalMessageEnvelope};
use kolme::KolmeClient;
use nonce::NonceTracker;

pub use errors::AgentLibError;
pub use identity::AgentIdentity;
pub use kolme::{KolmeProofReceipt, KolmeProofVerification};

pub use kamn_sdk::{
    ServiceAgentProfile, ServiceChannelMessages, ServiceChannelReceipt, ServiceEscrowStatus,
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
        let auth = self
            .service_client
            .build_auth(self.identity.did(), nonce, payload)?;
        self.service_client.send_message(payload, &auth)
    }

    /// Queries a message status by identifier.
    pub fn query_message(&self, message_id: &str) -> Result<ServiceMessageStatus, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self
            .service_client
            .build_auth(self.identity.did(), nonce, "")?;
        self.service_client.get_message(message_id, &auth)
    }

    /// Creates a channel through the service API.
    pub fn create_channel(&self, payload: &str) -> Result<ServiceChannelReceipt, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self
            .service_client
            .build_auth(self.identity.did(), nonce, payload)?;
        self.service_client.create_channel(payload, &auth)
    }

    /// Creates a task through the service API.
    pub fn create_task(&self, payload: &str) -> Result<ServiceTaskReceipt, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self
            .service_client
            .build_auth(self.identity.did(), nonce, payload)?;
        self.service_client.create_task(payload, &auth)
    }

    /// Queries task status by identifier.
    pub fn query_task(&self, task_id: &str) -> Result<ServiceTaskStatus, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self
            .service_client
            .build_auth(self.identity.did(), nonce, "")?;
        self.service_client.get_task(task_id, &auth)
    }

    /// Queries agent profile by DID.
    pub fn query_agent_profile(&self, did: &str) -> Result<ServiceAgentProfile, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self
            .service_client
            .build_auth(self.identity.did(), nonce, "")?;
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
        let auth = self
            .service_client
            .build_auth(self.identity.did(), nonce, "{}")?;
        self.service_client.accept_task(task_id, &auth)
    }

    /// Completes one task through the service API.
    pub fn complete_task(&self, task_id: &str) -> Result<ServiceTaskStatus, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self
            .service_client
            .build_auth(self.identity.did(), nonce, "{}")?;
        self.service_client.complete_task(task_id, &auth)
    }

    /// Funds escrow through the service API.
    pub fn fund_escrow(&self, payload: &str) -> Result<ServiceEscrowStatus, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self
            .service_client
            .build_auth(self.identity.did(), nonce, payload)?;
        self.service_client.fund_escrow(payload, &auth)
    }

    /// Releases one escrow through the service API.
    pub fn release_escrow(&self, escrow_id: &str) -> Result<ServiceEscrowStatus, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self
            .service_client
            .build_auth(self.identity.did(), nonce, "{}")?;
        self.service_client.release_escrow(escrow_id, &auth)
    }

    /// Lists channel messages through the service API.
    pub fn list_messages(&self, channel_id: &str) -> Result<ServiceChannelMessages, AgentLibError> {
        let nonce = self.next_nonce()?;
        let auth = self
            .service_client
            .build_auth(self.identity.did(), nonce, "")?;
        self.service_client.list_channel_messages(channel_id, &auth)
    }

    fn next_nonce(&self) -> Result<u64, AgentLibError> {
        let mut tracker = self
            .nonce_tracker
            .lock()
            .map_err(|error| AgentLibError::Internal(format!("nonce lock poisoned: {error}")))?;
        Ok(tracker.next_nonce())
    }
}

#[cfg(test)]
mod tests {
    use super::{AgentIdentity, KamnAgentHandle};

    #[test]
    fn unit_kamn_agent_handle_exposes_identity_after_connect() {
        let identity = AgentIdentity::from_agent_name("alice").expect("identity");
        let handle = KamnAgentHandle::with_identity(
            "http://localhost:8080",
            "http://localhost:3000",
            identity.clone(),
        )
        .expect("handle");

        assert_eq!(handle.identity().did(), identity.did());
    }
}
