use crate::{
    AgentDid, AllowAllBridgePolicy, BridgeAdapterEngine, BridgeInboundEnvelope, BridgePlatform,
    CanonicalMessageEnvelope, NormalizedInboundMessage, PassThroughBridgeAdapter,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegramBridgeConfig {
    pub bridge_agent_did: String,
    pub authorized_listener_dids: BTreeSet<String>,
    pub channel_routes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegramInboundRequest {
    pub listener_did: String,
    pub observed_at_unix: u64,
    pub inbound: BridgeInboundEnvelope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegramBridgeEngine {
    config: TelegramBridgeConfig,
    bridge: BridgeAdapterEngine<PassThroughBridgeAdapter, AllowAllBridgePolicy>,
}

impl TelegramBridgeEngine {
    pub fn new(config: TelegramBridgeConfig) -> Result<Self, TelegramBridgeError> {
        validate_did(&config.bridge_agent_did)?;
        if config.authorized_listener_dids.is_empty() {
            return Err(TelegramBridgeError::EmptyField("authorized_listener_dids"));
        }
        for listener_did in &config.authorized_listener_dids {
            validate_did(listener_did)?;
        }
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

        Ok(Self { config, bridge })
    }

    pub fn process_inbound(
        &self,
        request: &TelegramInboundRequest,
    ) -> Result<NormalizedInboundMessage, TelegramBridgeError> {
        self.validate_inbound_request(request)?;

        self.bridge
            .process_inbound(&request.inbound, request.observed_at_unix)
            .map_err(|error| TelegramBridgeError::Bridge(error.to_string()))
    }

    pub fn process_inbound_to_envelope(
        &self,
        request: &TelegramInboundRequest,
        recipient_keys: Vec<String>,
        expires: &str,
        nonce: u64,
    ) -> Result<CanonicalMessageEnvelope, TelegramBridgeError> {
        self.validate_inbound_request(request)?;
        self.bridge
            .process_inbound_to_envelope(
                &request.inbound,
                request.observed_at_unix,
                recipient_keys,
                expires,
                nonce,
            )
            .map_err(|error| TelegramBridgeError::Bridge(error.to_string()))
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelegramBridgeError {
    EmptyField(&'static str),
    InvalidDid(String),
    UnauthorizedListener(String),
    UnknownRouteChannel(String),
    RouteTargetMismatch {
        external_channel_id: String,
        expected_target_did: String,
        provided_target_did: String,
    },
    Bridge(String),
}

impl fmt::Display for TelegramBridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::UnauthorizedListener(value) => write!(f, "unauthorized listener did: {value}"),
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
            channel_routes,
        }
    }

    #[test]
    fn constructor_rejects_empty_listener_allowlist() {
        let mut config = config();
        config.authorized_listener_dids.clear();
        assert_eq!(
            TelegramBridgeEngine::new(config),
            Err(TelegramBridgeError::EmptyField("authorized_listener_dids"))
        );
    }

    #[test]
    fn constructor_rejects_invalid_target_did_in_route_map() {
        let mut config = config();
        config
            .channel_routes
            .insert("telegram:channel:ops".to_owned(), "bad-did".to_owned());
        assert_eq!(
            TelegramBridgeEngine::new(config),
            Err(TelegramBridgeError::InvalidDid(
                "invalid agent did prefix: bad-did".to_owned()
            ))
        );
    }
}
