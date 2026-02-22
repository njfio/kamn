use crate::errors::AgentLibError;
use kamn_sdk::{
    service_signature_for_fields, AgentDid, ServiceAgentProfile, ServiceApiClient,
    ServiceChannelReceipt, ServiceEscrowFundReceipt, ServiceEscrowReleaseReceipt,
    ServiceHealthStatus, ServiceMessageReceipt, ServiceMessageStatus, ServiceRequestAuth,
    ServiceTaskReceipt, ServiceTaskStatus,
};

const DEFAULT_CHAIN_ID: &str = "kamn-agent-lib";
const DEFAULT_CHAIN_VERSION: &str = "1";

/// Typed Service API HTTP client wrapper used by `KamnAgentHandle`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceApiHttpClient {
    inner: ServiceApiClient,
    chain_id: String,
    chain_version: String,
}

impl ServiceApiHttpClient {
    /// Connects to a KAMN service endpoint with default chain-signature context.
    pub fn connect(endpoint: &str) -> Result<Self, AgentLibError> {
        Self::connect_with_chain_context(endpoint, DEFAULT_CHAIN_ID, DEFAULT_CHAIN_VERSION)
    }

    /// Connects to a KAMN service endpoint with explicit chain-signature context.
    pub fn connect_with_chain_context(
        endpoint: &str,
        chain_id: &str,
        chain_version: &str,
    ) -> Result<Self, AgentLibError> {
        if chain_id.trim().is_empty() {
            return Err(AgentLibError::InvalidInput {
                field: "chain_id",
                reason: "must not be empty".to_owned(),
            });
        }
        if chain_version.trim().is_empty() {
            return Err(AgentLibError::InvalidInput {
                field: "chain_version",
                reason: "must not be empty".to_owned(),
            });
        }
        Ok(Self {
            inner: ServiceApiClient::connect(endpoint)?,
            chain_id: chain_id.to_owned(),
            chain_version: chain_version.to_owned(),
        })
    }

    /// Builds request auth from DID + nonce + request body.
    pub fn build_auth(
        &self,
        sender_did: &AgentDid,
        nonce: u64,
        body: &str,
    ) -> Result<ServiceRequestAuth, AgentLibError> {
        let signature = service_signature_for_fields(
            sender_did,
            nonce,
            self.chain_id.as_str(),
            self.chain_version.as_str(),
            body,
        );
        Ok(ServiceRequestAuth::new(
            sender_did.clone(),
            nonce,
            signature,
        )?)
    }

    /// Sends a message through `POST /v1/messages/send`.
    pub fn send_message(
        &self,
        payload: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceMessageReceipt, AgentLibError> {
        Ok(self.inner.send_message(payload, auth)?)
    }

    /// Queries a message via `GET /v1/messages/{id}`.
    pub fn get_message(
        &self,
        message_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceMessageStatus, AgentLibError> {
        Ok(self.inner.get_message(message_id, auth)?)
    }

    /// Creates a channel via `POST /v1/channels/create`.
    pub fn create_channel(
        &self,
        payload: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceChannelReceipt, AgentLibError> {
        Ok(self.inner.create_channel(payload, auth)?)
    }

    /// Creates a task via `POST /v1/tasks/create`.
    pub fn create_task(
        &self,
        payload: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceTaskReceipt, AgentLibError> {
        Ok(self.inner.create_task(payload, auth)?)
    }

    /// Queries a task via `GET /v1/tasks/{id}`.
    pub fn get_task(
        &self,
        task_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceTaskStatus, AgentLibError> {
        Ok(self.inner.get_task(task_id, auth)?)
    }

    /// Accepts one task via `POST /v1/tasks/{id}/accept`.
    pub fn accept_task(
        &self,
        task_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceTaskStatus, AgentLibError> {
        Ok(self.inner.accept_task(task_id, auth)?)
    }

    /// Completes one task via `POST /v1/tasks/{id}/complete`.
    pub fn complete_task(
        &self,
        task_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceTaskStatus, AgentLibError> {
        Ok(self.inner.complete_task(task_id, auth)?)
    }

    /// Funds one escrow via `POST /v1/escrow/fund`.
    pub fn fund_escrow(
        &self,
        payload: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceEscrowFundReceipt, AgentLibError> {
        Ok(self.inner.fund_escrow(payload, auth)?)
    }

    /// Releases one escrow via `POST /v1/escrow/{id}/release`.
    pub fn release_escrow(
        &self,
        escrow_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceEscrowReleaseReceipt, AgentLibError> {
        Ok(self.inner.release_escrow(escrow_id, auth)?)
    }

    /// Queries one agent profile via `GET /v1/agents/{did}`.
    pub fn get_agent_profile(
        &self,
        did: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceAgentProfile, AgentLibError> {
        Ok(self.inner.get_agent_profile(did, auth)?)
    }

    /// Queries service health via `GET /healthz`.
    pub fn health(&self) -> Result<ServiceHealthStatus, AgentLibError> {
        Ok(self.inner.health()?)
    }
}
