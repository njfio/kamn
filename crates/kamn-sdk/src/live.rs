use crate::{
    service_signature_for_fields, AgentDid, AgentMetadata, AgentQuery, AgentReputation,
    AgentSummary, Artifact, ArtifactId, DidDocument, EscrowConfig, EscrowId, KamnAgent,
    KamnTransport, Message, MessageId, MessageRecord, MessageStream, SdkError, ServiceApiClient,
    ServiceRequestAuth, TaskDefinition, TaskId, TokenAmount, TransportMode,
};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};

const LIVE_CHAIN_ID_ENV: &str = "KAMN_SDK_LIVE_CHAIN_ID";
const LIVE_CHAIN_VERSION_ENV: &str = "KAMN_SDK_LIVE_CHAIN_VERSION";
const LIVE_REQUESTER_DID_ENV: &str = "KAMN_SDK_LIVE_REQUESTER_DID";
const DEFAULT_LIVE_CHAIN_ID: &str = "kamn-sdk-live";
const DEFAULT_LIVE_CHAIN_VERSION: &str = "1";
const DEFAULT_LIVE_REQUESTER_DID: &str = "kamn:did:agent:live-sdk";
const AGENTS_READ_SCOPE: &str = "agents:read";
const MESSAGES_WRITE_SCOPE: &str = "messages:write";

#[derive(Debug, Default)]
struct LiveTransportState {
    sender_nonces: HashMap<String, u64>,
    message_ids: HashMap<u64, String>,
}

/// Configuration for the live transport client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveTransportConfig {
    /// Service API HTTP(S) endpoint used for live transport operations.
    pub endpoint: String,
    chain_id: String,
    chain_version: String,
    requester_did: AgentDid,
}

impl LiveTransportConfig {
    /// Validates and constructs a live transport configuration.
    pub fn new(endpoint: &str) -> Result<Self, SdkError> {
        let normalized = endpoint.trim().to_ascii_lowercase();
        if !(normalized.starts_with("http://") || normalized.starts_with("https://")) {
            return Err(SdkError::InvalidInput {
                field: "transport.endpoint",
                reason: "must start with http:// or https://",
            });
        }
        if endpoint.trim().len() < "http://a".len() {
            return Err(SdkError::InvalidInput {
                field: "transport.endpoint",
                reason: "must include host information",
            });
        }

        let chain_id = env_var_or_default(LIVE_CHAIN_ID_ENV, DEFAULT_LIVE_CHAIN_ID);
        if chain_id.trim().is_empty() {
            return Err(SdkError::InvalidInput {
                field: "transport.chain_id",
                reason: "must not be empty",
            });
        }

        let chain_version = env_var_or_default(LIVE_CHAIN_VERSION_ENV, DEFAULT_LIVE_CHAIN_VERSION);
        if chain_version.trim().is_empty() {
            return Err(SdkError::InvalidInput {
                field: "transport.chain_version",
                reason: "must not be empty",
            });
        }

        let requester_did_raw =
            env_var_or_default(LIVE_REQUESTER_DID_ENV, DEFAULT_LIVE_REQUESTER_DID);
        let requester_did =
            AgentDid::parse(requester_did_raw).map_err(|_| SdkError::InvalidInput {
                field: "transport.requester_did",
                reason: "must be a valid kamn agent did",
            })?;

        Ok(Self {
            endpoint: endpoint.trim().to_owned(),
            chain_id,
            chain_version,
            requester_did,
        })
    }
}

/// Live transport client backed by the Service API.
#[derive(Debug, Clone)]
pub struct LiveTransportKamnClient {
    config: LiveTransportConfig,
    service_client: ServiceApiClient,
    state: Arc<Mutex<LiveTransportState>>,
}

impl LiveTransportKamnClient {
    /// Connects to a service endpoint and returns a live transport client.
    pub fn connect(endpoint: &str) -> Result<Self, SdkError> {
        let config = LiveTransportConfig::new(endpoint)?;
        let service_client = ServiceApiClient::connect(config.endpoint.as_str())?;
        Ok(Self {
            config,
            service_client,
            state: Arc::new(Mutex::new(LiveTransportState::default())),
        })
    }

    /// Returns the configured endpoint for this client.
    pub fn endpoint(&self) -> &str {
        &self.config.endpoint
    }

    fn next_nonce(&self, sender_did: &AgentDid) -> Result<u64, SdkError> {
        let mut guard = self
            .state
            .lock()
            .map_err(|_| SdkError::TransportFailure("live transport state lock poisoned"))?;
        let nonce = guard
            .sender_nonces
            .entry(sender_did.as_str().to_owned())
            .or_insert(0);
        if *nonce == u64::MAX {
            return Err(SdkError::Conflict(
                "live transport nonce exhausted for sender",
            ));
        }
        *nonce += 1;
        Ok(*nonce)
    }

    fn build_auth(
        &self,
        sender_did: &AgentDid,
        body: &str,
        scope: Option<&str>,
    ) -> Result<ServiceRequestAuth, SdkError> {
        let nonce = self.next_nonce(sender_did)?;
        let signature = service_signature_for_fields(
            sender_did,
            nonce,
            self.config.chain_id.as_str(),
            self.config.chain_version.as_str(),
            body,
        )?;
        ServiceRequestAuth::new_with_scope(sender_did.clone(), nonce, signature, scope)
    }

    fn map_service_message_id(&self, service_message_id: &str) -> Result<MessageId, SdkError> {
        if service_message_id.trim().is_empty() {
            return Err(SdkError::TransportFailure(
                "service returned empty message_id in send response",
            ));
        }

        let numeric_id = deterministic_u64_tag(service_message_id);
        let mut guard = self
            .state
            .lock()
            .map_err(|_| SdkError::TransportFailure("live transport state lock poisoned"))?;
        if let Some(existing) = guard.message_ids.get(&numeric_id) {
            if existing != service_message_id {
                return Err(SdkError::Conflict(
                    "service message id collision detected in sdk numeric alias map",
                ));
            }
        } else {
            guard
                .message_ids
                .insert(numeric_id, service_message_id.to_owned());
        }
        Ok(MessageId(numeric_id))
    }

    fn unsupported<T>(feature: &'static str) -> Result<T, SdkError> {
        Err(SdkError::NotImplemented(feature))
    }
}

fn env_var_or_default(name: &str, default: &str) -> String {
    match std::env::var(name) {
        Ok(value) if !value.trim().is_empty() => value,
        _ => default.to_owned(),
    }
}

fn deterministic_u64_tag(value: &str) -> u64 {
    let mut acc: u64 = 0xcbf29ce484222325;
    for byte in value.as_bytes() {
        acc ^= u64::from(*byte);
        acc = acc.wrapping_mul(0x00000100000001B3);
    }
    acc
}

fn escape_json(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{0008}' => escaped.push_str("\\b"),
            '\u{000C}' => escaped.push_str("\\f"),
            value if value.is_control() => {
                let _ = write!(escaped, "\\u{:04x}", value as u32);
            }
            value => escaped.push(value),
        }
    }
    escaped
}

fn service_message_payload(message: &Message) -> String {
    let channel_segment = match &message.channel {
        Some(channel_id) => {
            format!(",\"channel_id\":\"{}\"", escape_json(channel_id.0.as_str()))
        }
        None => String::new(),
    };

    format!(
        "{{\"from\":\"{}\",\"to\":\"{}\",\"body\":\"{}\"{}}}",
        escape_json(message.from.as_str()),
        escape_json(message.to.as_str()),
        escape_json(message.body.as_str()),
        channel_segment,
    )
}

impl KamnTransport for LiveTransportKamnClient {
    fn transport_mode(&self) -> TransportMode {
        TransportMode::Live
    }
}

impl KamnAgent for LiveTransportKamnClient {
    fn register(&mut self, _metadata: AgentMetadata) -> Result<AgentDid, SdkError> {
        Self::unsupported("live transport register route is not available via service api")
    }

    fn resolve(&self, did: &AgentDid) -> Result<DidDocument, SdkError> {
        let auth = self.build_auth(&self.config.requester_did, "", Some(AGENTS_READ_SCOPE))?;
        let profile = self.service_client.get_agent_profile(did.as_str(), &auth)?;
        let resolved_did = AgentDid::parse(profile.did).map_err(|_| {
            SdkError::TransportFailure("service returned invalid did in agent profile response")
        })?;
        Ok(DidDocument {
            id: resolved_did,
            metadata: AgentMetadata {
                agent_type: "service-agent".to_owned(),
                model_family: "service-api".to_owned(),
                capabilities: vec!["profile:read".to_owned()],
            },
            service_endpoint: self.endpoint().to_owned(),
        })
    }

    fn send(&mut self, message: Message) -> Result<MessageId, SdkError> {
        let payload = service_message_payload(&message);
        let auth = self.build_auth(&message.from, payload.as_str(), Some(MESSAGES_WRITE_SCOPE))?;
        let receipt = self.service_client.send_message(payload.as_str(), &auth)?;
        self.map_service_message_id(receipt.message_id.as_str())
    }

    fn receive(&mut self, _did: &AgentDid) -> Result<Vec<MessageRecord>, SdkError> {
        Self::unsupported("live transport receive route is not available via service api")
    }

    fn receive_stream(&mut self, _did: &AgentDid) -> Result<MessageStream, SdkError> {
        Self::unsupported("live transport receive route is not available via service api")
    }

    fn create_task(&mut self, _task: TaskDefinition) -> Result<TaskId, SdkError> {
        Self::unsupported("live transport task routes are not yet mapped in sdk kamn-agent surface")
    }

    fn accept_task(&mut self, _task_id: &TaskId, _assignee: &AgentDid) -> Result<(), SdkError> {
        Self::unsupported("live transport task routes are not yet mapped in sdk kamn-agent surface")
    }

    fn submit_artifact(
        &mut self,
        _task_id: &TaskId,
        _artifact: Artifact,
    ) -> Result<ArtifactId, SdkError> {
        Self::unsupported(
            "live transport artifact routes are not yet mapped in sdk kamn-agent surface",
        )
    }

    fn complete_task(&mut self, _task_id: &TaskId) -> Result<(), SdkError> {
        Self::unsupported("live transport task routes are not yet mapped in sdk kamn-agent surface")
    }

    fn create_escrow(&mut self, _escrow: EscrowConfig) -> Result<EscrowId, SdkError> {
        Self::unsupported(
            "live transport escrow routes are not yet mapped in sdk kamn-agent surface",
        )
    }

    fn release_escrow(&mut self, _escrow_id: &EscrowId) -> Result<(), SdkError> {
        Self::unsupported(
            "live transport escrow routes are not yet mapped in sdk kamn-agent surface",
        )
    }

    fn balance(&self, _did: &AgentDid) -> Result<TokenAmount, SdkError> {
        Self::unsupported("live transport balance route is not available via service api")
    }

    fn search_agents(&self, _query: AgentQuery) -> Result<Vec<AgentSummary>, SdkError> {
        Self::unsupported("live transport agent search route is not available via service api")
    }

    fn get_reputation(&self, agent: &AgentDid) -> Result<AgentReputation, SdkError> {
        let auth = self.build_auth(&self.config.requester_did, "", Some(AGENTS_READ_SCOPE))?;
        let profile = self
            .service_client
            .get_agent_profile(agent.as_str(), &auth)?;
        let profile_did = AgentDid::parse(profile.did).map_err(|_| {
            SdkError::TransportFailure("service returned invalid did in agent profile response")
        })?;
        let score = u32::try_from(profile.reputation_score).map_err(|_| {
            SdkError::TransportFailure("service returned reputation score outside u32 range")
        })?;
        Ok(AgentReputation {
            did: profile_did,
            score,
        })
    }
}
