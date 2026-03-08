use crate::{AgentDid, SdkError};

const LIVE_CHAIN_ID_ENV: &str = "KAMN_SDK_LIVE_CHAIN_ID";
const LIVE_CHAIN_VERSION_ENV: &str = "KAMN_SDK_LIVE_CHAIN_VERSION";
const LIVE_REQUESTER_DID_ENV: &str = "KAMN_SDK_LIVE_REQUESTER_DID";
const DEFAULT_LIVE_CHAIN_ID: &str = "kamn-sdk-live";
const DEFAULT_LIVE_CHAIN_VERSION: &str = "1";
const DEFAULT_LIVE_REQUESTER_DID: &str = "kamn:did:agent:live-sdk";

pub(crate) const AGENTS_READ_SCOPE: &str = "agents:read";
pub(crate) const AGENTS_WRITE_SCOPE: &str = "agents:write";
pub(crate) const CHANNELS_WRITE_SCOPE: &str = "channels:write";
pub(crate) const CONTENT_READ_SCOPE: &str = "content:read";
pub(crate) const CONTENT_WRITE_SCOPE: &str = "content:write";
pub(crate) const ESCROW_WRITE_SCOPE: &str = "escrow:write";
pub(crate) const MESSAGES_WRITE_SCOPE: &str = "messages:write";
pub(crate) const TASKS_READ_SCOPE: &str = "tasks:read";
pub(crate) const TASKS_WRITE_SCOPE: &str = "tasks:write";

/// Configuration for the live transport client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveTransportConfig {
    /// Service API HTTP(S) endpoint used for live transport operations.
    pub endpoint: String,
    pub(crate) chain_id: String,
    pub(crate) chain_version: String,
    pub(crate) requester_did: AgentDid,
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
            AgentDid::parse(&requester_did_raw).map_err(|_| SdkError::InvalidInput {
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

fn env_var_or_default(name: &str, default: &str) -> String {
    match std::env::var(name) {
        Ok(value) if !value.trim().is_empty() => value,
        _ => default.to_owned(),
    }
}
