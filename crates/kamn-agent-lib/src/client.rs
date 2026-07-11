use crate::errors::AgentLibError;
use kamn_sdk::{
    service_public_key_for_private_key, service_signature_for_state_hash_with_private_key,
    AgentDid, AgentMetadata, ServiceAgentProfile, ServiceApiClient, ServiceBridgeStatus,
    ServiceBridgeSubmission, ServiceChannelMessages, ServiceChannelReceipt,
    ServiceContentRegistration, ServiceContentStatus, ServiceEscrowStatus, ServiceHealthStatus,
    ServiceMessageReceipt, ServiceMessageStatus, ServiceRequestAuth, ServiceTaskReceipt,
    ServiceTaskStatus,
};
use std::env;

const DEFAULT_CHAIN_ID: &str = "kamn-devnet";
const DEFAULT_CHAIN_VERSION: &str = "v0.1.0";
const AGENT_CHAIN_ID_ENV: &str = "KAMN_AGENT_CHAIN_ID";
const AGENT_CHAIN_VERSION_ENV: &str = "KAMN_AGENT_CHAIN_VERSION";

fn env_var_or_default(key: &str, default: &str) -> String {
    match env::var(key) {
        Ok(value) if !value.trim().is_empty() => value,
        _ => default.to_owned(),
    }
}

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
        let chain_id = env_var_or_default(AGENT_CHAIN_ID_ENV, DEFAULT_CHAIN_ID);
        let chain_version = env_var_or_default(AGENT_CHAIN_VERSION_ENV, DEFAULT_CHAIN_VERSION);
        Self::connect_with_chain_context(endpoint, chain_id.as_str(), chain_version.as_str())
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
        signing_key: &str,
        nonce: u64,
        body: &str,
        authz_scope: Option<&str>,
    ) -> Result<ServiceRequestAuth, AgentLibError> {
        let state_hash = format!("service-api:{}:{}", self.chain_id, self.chain_version);
        let signature = service_signature_for_state_hash_with_private_key(
            sender_did,
            nonce,
            state_hash.as_str(),
            body,
            signing_key,
        )?;
        let signer_public_key_hex = service_public_key_for_private_key(signing_key)?;
        Ok(ServiceRequestAuth::new_with_signer_public_key_and_scope(
            sender_did.clone(),
            nonce,
            signature,
            Some(signer_public_key_hex.as_str()),
            authz_scope,
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

    /// Lists channel messages via `GET /v1/channels/{id}/messages`.
    pub fn list_channel_messages(
        &self,
        channel_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceChannelMessages, AgentLibError> {
        Ok(self.inner.list_channel_messages(channel_id, auth)?)
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

    /// Accepts one task with a canonical transition payload.
    pub fn accept_task_with_payload(
        &self,
        task_id: &str,
        payload: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceTaskStatus, AgentLibError> {
        Ok(self
            .inner
            .accept_task_with_payload(task_id, payload, auth)?)
    }

    /// Completes one task via `POST /v1/tasks/{id}/complete`.
    pub fn complete_task(
        &self,
        task_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceTaskStatus, AgentLibError> {
        Ok(self.inner.complete_task(task_id, auth)?)
    }

    /// Completes one task with a canonical evidence payload.
    pub fn complete_task_with_payload(
        &self,
        task_id: &str,
        payload: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceTaskStatus, AgentLibError> {
        Ok(self
            .inner
            .complete_task_with_payload(task_id, payload, auth)?)
    }

    /// Funds escrow via `POST /v1/escrow/fund`.
    pub fn fund_escrow(
        &self,
        payload: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceEscrowStatus, AgentLibError> {
        Ok(self.inner.fund_escrow(payload, auth)?)
    }

    /// Releases one escrow via `POST /v1/escrow/{id}/release`.
    pub fn release_escrow(
        &self,
        escrow_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceEscrowStatus, AgentLibError> {
        Ok(self.inner.release_escrow(escrow_id, auth)?)
    }

    /// Releases escrow with a canonical idempotency payload.
    pub fn release_escrow_with_payload(
        &self,
        escrow_id: &str,
        payload: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceEscrowStatus, AgentLibError> {
        Ok(self
            .inner
            .release_escrow_with_payload(escrow_id, payload, auth)?)
    }

    /// Registers content lifecycle via `POST /v1/content/register`.
    pub fn register_content(
        &self,
        payload: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceContentRegistration, AgentLibError> {
        Ok(self.inner.register_content(payload, auth)?)
    }

    /// Expires one content record via `POST /v1/content/{id}/expire`.
    pub fn expire_content(
        &self,
        content_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceContentStatus, AgentLibError> {
        Ok(self.inner.expire_content(content_id, auth)?)
    }

    /// Tombstones one content record via `POST /v1/content/{id}/tombstone`.
    pub fn tombstone_content(
        &self,
        content_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceContentStatus, AgentLibError> {
        Ok(self.inner.tombstone_content(content_id, auth)?)
    }

    /// Queries one content lifecycle record via `GET /v1/content/{id}`.
    pub fn get_content(
        &self,
        content_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceContentStatus, AgentLibError> {
        Ok(self.inner.get_content(content_id, auth)?)
    }

    /// Submits one bridge message via `POST /v1/bridge/submit`.
    pub fn submit_bridge_message(
        &self,
        payload: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceBridgeSubmission, AgentLibError> {
        Ok(self.inner.submit_bridge_message(payload, auth)?)
    }

    /// Forwards one bridge message via `POST /v1/bridge/{id}/forward`.
    pub fn forward_bridge_message(
        &self,
        bridge_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceBridgeStatus, AgentLibError> {
        Ok(self.inner.forward_bridge_message(bridge_id, auth)?)
    }

    /// Queries one bridge message via `GET /v1/bridge/{id}`.
    pub fn get_bridge_message(
        &self,
        bridge_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceBridgeStatus, AgentLibError> {
        Ok(self.inner.get_bridge_message(bridge_id, auth)?)
    }

    /// Queries one agent profile via `GET /v1/agents/{did}`.
    pub fn get_agent_profile(
        &self,
        did: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceAgentProfile, AgentLibError> {
        Ok(self.inner.get_agent_profile(did, auth)?)
    }

    /// Registers the authenticated sender through `POST /v1/agents/register`.
    pub fn register_agent(
        &self,
        metadata: &AgentMetadata,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceAgentProfile, AgentLibError> {
        Ok(self.inner.register_agent(metadata, auth)?)
    }

    /// Queries service health via `GET /healthz`.
    pub fn health(&self) -> Result<ServiceHealthStatus, AgentLibError> {
        Ok(self.inner.health()?)
    }
}
