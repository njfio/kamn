use crate::{
    AgentDid, AllowAllBridgePolicy, BridgeAdapterEngine, BridgeInboundEnvelope, BridgePlatform,
    CanonicalMessageEnvelope, NormalizedInboundMessage, PassThroughBridgeAdapter,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Mutex;

/// Telegram bridge configuration for listener authorization and channel routing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegramBridgeConfig {
    /// DID used by the bridge agent identity.
    pub bridge_agent_did: String,
    /// Listener DIDs allowed to submit inbound webhook payloads.
    pub authorized_listener_dids: BTreeSet<String>,
    /// Shared webhook token expected on inbound requests.
    pub webhook_token: String,
    /// Mapping from external Telegram channel id to target DID.
    pub channel_routes: BTreeMap<String, String>,
}

/// Normalized inbound webhook request metadata for Telegram bridge processing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegramInboundRequest {
    /// DID of the listener submitting the inbound payload.
    pub listener_did: String,
    /// Webhook token presented by the listener.
    pub webhook_token: String,
    /// Monotonic checkpoint value from external channel stream.
    pub checkpoint: u64,
    /// Unix timestamp when listener observed the payload.
    pub observed_at_unix: u64,
    /// Inbound envelope normalized for bridge adapter processing.
    pub inbound: BridgeInboundEnvelope,
}

/// Telegram bridge engine that validates inbound requests and normalizes payloads.
#[derive(Debug)]
pub struct TelegramBridgeEngine {
    config: TelegramBridgeConfig,
    bridge: BridgeAdapterEngine<PassThroughBridgeAdapter, AllowAllBridgePolicy>,
    channel_checkpoints: Mutex<BTreeMap<String, u64>>,
}

impl TelegramBridgeEngine {
    /// Creates a Telegram bridge engine after validating configuration shape and DIDs.
    pub fn new(config: TelegramBridgeConfig) -> Result<Self, TelegramBridgeError> {
        validate_did(&config.bridge_agent_did)?;
        if config.authorized_listener_dids.is_empty() {
            return Err(TelegramBridgeError::EmptyField("authorized_listener_dids"));
        }
        for listener_did in &config.authorized_listener_dids {
            validate_did(listener_did)?;
        }
        validate_non_empty("webhook_token", &config.webhook_token)?;
        if config.channel_routes.is_empty() {
            return Err(TelegramBridgeError::EmptyField("channel_routes"));
        }
        for (external_channel_id, target_did) in &config.channel_routes {
            validate_non_empty("channel_routes.external_channel_id", external_channel_id)?;
            validate_did(target_did)?;
        }

        let adapter =
            PassThroughBridgeAdapter::new(BridgePlatform::Telegram, &config.bridge_agent_did)
                .map_err(|error| TelegramBridgeError::Bridge(error.to_string()))?;
        let bridge = BridgeAdapterEngine::new(adapter, AllowAllBridgePolicy::new());

        Ok(Self {
            config,
            bridge,
            channel_checkpoints: Mutex::new(BTreeMap::new()),
        })
    }

    /// Validates and processes an inbound request into normalized bridge message form.
    pub fn process_inbound(
        &self,
        request: &TelegramInboundRequest,
    ) -> Result<NormalizedInboundMessage, TelegramBridgeError> {
        self.validate_inbound_request(request)?;

        let normalized = self
            .bridge
            .process_inbound(&request.inbound, request.observed_at_unix)
            .map_err(|error| TelegramBridgeError::Bridge(error.to_string()))?;
        self.record_checkpoint(&request.inbound.external_channel_id, request.checkpoint)?;
        Ok(normalized)
    }

    /// Validates and processes an inbound request into canonical message-envelope form.
    pub fn process_inbound_to_envelope(
        &self,
        request: &TelegramInboundRequest,
        recipient_keys: Vec<String>,
        expires: &str,
        nonce: u64,
    ) -> Result<CanonicalMessageEnvelope, TelegramBridgeError> {
        self.validate_inbound_request(request)?;
        let envelope = self
            .bridge
            .process_inbound_to_envelope(
                &request.inbound,
                request.observed_at_unix,
                recipient_keys,
                expires,
                nonce,
            )
            .map_err(|error| TelegramBridgeError::Bridge(error.to_string()))?;
        self.record_checkpoint(&request.inbound.external_channel_id, request.checkpoint)?;
        Ok(envelope)
    }

    fn validate_inbound_request(
        &self,
        request: &TelegramInboundRequest,
    ) -> Result<(), TelegramBridgeError> {
        validate_did(&request.listener_did)?;
        if !self
            .config
            .authorized_listener_dids
            .contains(&request.listener_did)
        {
            return Err(TelegramBridgeError::UnauthorizedListener(
                request.listener_did.clone(),
            ));
        }
        validate_non_empty(
            "telegram_inbound_request.webhook_token",
            &request.webhook_token,
        )?;
        if request.webhook_token != self.config.webhook_token {
            return Err(TelegramBridgeError::InvalidWebhookToken);
        }

        let external_channel_id = request.inbound.external_channel_id.clone();
        let expected_target_did = self
            .config
            .channel_routes
            .get(&external_channel_id)
            .ok_or_else(|| TelegramBridgeError::UnknownRouteChannel(external_channel_id.clone()))?;

        if expected_target_did != &request.inbound.target_agent_did {
            return Err(TelegramBridgeError::RouteTargetMismatch {
                external_channel_id,
                expected_target_did: expected_target_did.clone(),
                provided_target_did: request.inbound.target_agent_did.clone(),
            });
        }
        Ok(())
    }

    fn record_checkpoint(
        &self,
        external_channel_id: &str,
        checkpoint: u64,
    ) -> Result<(), TelegramBridgeError> {
        let mut guard = self
            .channel_checkpoints
            .lock()
            .map_err(|_| TelegramBridgeError::CheckpointStateUnavailable)?;
        if let Some(last_checkpoint) = guard.get(external_channel_id).copied() {
            if checkpoint <= last_checkpoint {
                return Err(TelegramBridgeError::NonMonotonicCheckpoint {
                    external_channel_id: external_channel_id.to_owned(),
                    last_checkpoint,
                    received_checkpoint: checkpoint,
                });
            }
        }
        guard.insert(external_channel_id.to_owned(), checkpoint);
        Ok(())
    }
}

/// Errors emitted by Telegram bridge configuration and inbound processing paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelegramBridgeError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// DID failed validation.
    InvalidDid(String),
    /// Listener DID is not authorized.
    UnauthorizedListener(String),
    /// Webhook token did not match configured value.
    InvalidWebhookToken,
    /// Checkpoint is not strictly monotonic for a route channel.
    NonMonotonicCheckpoint {
        /// External route channel identifier.
        external_channel_id: String,
        /// Previously accepted checkpoint.
        last_checkpoint: u64,
        /// Newly received checkpoint.
        received_checkpoint: u64,
    },
    /// Checkpoint state lock could not be acquired.
    CheckpointStateUnavailable,
    /// Route channel id does not exist in configured mapping.
    UnknownRouteChannel(String),
    /// Inbound envelope target DID does not match configured route target DID.
    RouteTargetMismatch {
        /// External route channel identifier.
        external_channel_id: String,
        /// Expected route target DID from configuration.
        expected_target_did: String,
        /// Target DID present in inbound payload.
        provided_target_did: String,
    },
    /// Underlying bridge-adapter processing error.
    Bridge(String),
}

impl fmt::Display for TelegramBridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::UnauthorizedListener(value) => write!(f, "unauthorized listener did: {value}"),
            Self::InvalidWebhookToken => write!(f, "invalid telegram webhook token"),
            Self::NonMonotonicCheckpoint {
                external_channel_id,
                last_checkpoint,
                received_checkpoint,
            } => write!(
                f,
                "non-monotonic telegram checkpoint for {external_channel_id}: last {last_checkpoint}, received {received_checkpoint}"
            ),
            Self::CheckpointStateUnavailable => {
                write!(f, "telegram checkpoint state is unavailable")
            }
            Self::UnknownRouteChannel(value) => {
                write!(f, "telegram channel route not found: {value}")
            }
            Self::RouteTargetMismatch {
                external_channel_id,
                expected_target_did,
                provided_target_did,
            } => write!(
                f,
                "telegram channel route target mismatch for {external_channel_id}, expected {expected_target_did}, got {provided_target_did}"
            ),
            Self::Bridge(value) => write!(f, "telegram bridge error: {value}"),
        }
    }
}

impl std::error::Error for TelegramBridgeError {}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), TelegramBridgeError> {
    if value.trim().is_empty() {
        return Err(TelegramBridgeError::EmptyField(field));
    }
    Ok(())
}

fn validate_did(value: &str) -> Result<(), TelegramBridgeError> {
    AgentDid::parse(value).map_err(|error| TelegramBridgeError::InvalidDid(error.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{TelegramBridgeConfig, TelegramBridgeEngine, TelegramBridgeError};
    use std::collections::{BTreeMap, BTreeSet};

    fn config() -> TelegramBridgeConfig {
        let mut channel_routes = BTreeMap::new();
        channel_routes.insert(
            "telegram:channel:ops".to_owned(),
            "kamn:did:agent:target-1".to_owned(),
        );
        TelegramBridgeConfig {
            bridge_agent_did: "kamn:did:agent:bridge-telegram-1".to_owned(),
            authorized_listener_dids: ["kamn:did:agent:listener-1".to_owned()]
                .into_iter()
                .collect::<BTreeSet<_>>(),
            webhook_token: "telegram-webhook-token-valid".to_owned(),
            channel_routes,
        }
    }

    #[test]
    fn constructor_rejects_empty_listener_allowlist() {
        let mut config = config();
        config.authorized_listener_dids.clear();
        let error =
            TelegramBridgeEngine::new(config).expect_err("empty listener allowlist must fail");
        assert_eq!(
            error,
            TelegramBridgeError::EmptyField("authorized_listener_dids")
        );
    }

    #[test]
    fn constructor_rejects_invalid_target_did_in_route_map() {
        let mut config = config();
        config
            .channel_routes
            .insert("telegram:channel:ops".to_owned(), "bad-did".to_owned());
        let error =
            TelegramBridgeEngine::new(config).expect_err("invalid route target DID must fail");
        assert_eq!(
            error,
            TelegramBridgeError::InvalidDid("invalid agent did prefix: bad-did".to_owned())
        );
    }

    #[test]
    fn constructor_rejects_empty_webhook_token() {
        let mut config = config();
        config.webhook_token.clear();
        let error = TelegramBridgeEngine::new(config).expect_err("empty webhook token must fail");
        assert_eq!(error, TelegramBridgeError::EmptyField("webhook_token"));
    }
}
