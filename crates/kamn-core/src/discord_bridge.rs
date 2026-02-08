use crate::{
    AgentDid, AllowAllBridgePolicy, BridgeAdapterEngine, BridgeInboundEnvelope,
    BridgeOutboundEnvelope, BridgeOutboundRequest, BridgePlatform, CanonicalMessageEnvelope,
    NormalizedInboundMessage, PassThroughBridgeAdapter,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscordBridgeConfig {
    pub bridge_agent_did: String,
    pub authorized_listener_dids: BTreeSet<String>,
    pub authorized_approver_dids: BTreeSet<String>,
    pub required_approvals: usize,
    pub channel_routes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscordInboundRequest {
    pub listener_did: String,
    pub inbound: BridgeInboundEnvelope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscordOutboundApproval {
    pub request_id: String,
    pub required_approvals: usize,
    pub approved_by: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscordOutboundDispatch {
    pub envelope: BridgeOutboundEnvelope,
    pub approval: DiscordOutboundApproval,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscordBridgeEngine {
    config: DiscordBridgeConfig,
    bridge: BridgeAdapterEngine<PassThroughBridgeAdapter, AllowAllBridgePolicy>,
}

impl DiscordBridgeEngine {
    pub fn new(config: DiscordBridgeConfig) -> Result<Self, DiscordBridgeError> {
        validate_did(&config.bridge_agent_did)?;
        if config.authorized_listener_dids.is_empty() {
            return Err(DiscordBridgeError::EmptyField("authorized_listener_dids"));
        }
        for listener_did in &config.authorized_listener_dids {
            validate_did(listener_did)?;
        }

        if config.authorized_approver_dids.is_empty() {
            return Err(DiscordBridgeError::EmptyField("authorized_approver_dids"));
        }
        for approver_did in &config.authorized_approver_dids {
            validate_did(approver_did)?;
        }
        if config.required_approvals == 0
            || config.required_approvals > config.authorized_approver_dids.len()
        {
            return Err(DiscordBridgeError::InvalidRequiredApprovals {
                required: config.required_approvals,
                approver_count: config.authorized_approver_dids.len(),
            });
        }

        if config.channel_routes.is_empty() {
            return Err(DiscordBridgeError::EmptyField("channel_routes"));
        }
        for (external_channel_id, target_did) in &config.channel_routes {
            validate_non_empty("channel_routes.external_channel_id", external_channel_id)?;
            validate_did(target_did)?;
        }

        let adapter =
            PassThroughBridgeAdapter::new(BridgePlatform::Discord, &config.bridge_agent_did)
                .map_err(|error| DiscordBridgeError::Bridge(error.to_string()))?;
        let bridge = BridgeAdapterEngine::new(adapter, AllowAllBridgePolicy::new());
        Ok(Self { config, bridge })
    }

    pub fn process_inbound(
        &self,
        request: &DiscordInboundRequest,
    ) -> Result<NormalizedInboundMessage, DiscordBridgeError> {
        self.validate_inbound_request(request)?;

        self.bridge
            .process_inbound(&request.inbound)
            .map_err(|error| DiscordBridgeError::Bridge(error.to_string()))
    }

    pub fn process_inbound_to_envelope(
        &self,
        request: &DiscordInboundRequest,
        recipient_keys: Vec<String>,
        expires: &str,
        nonce: u64,
    ) -> Result<CanonicalMessageEnvelope, DiscordBridgeError> {
        self.validate_inbound_request(request)?;
        self.bridge
            .process_inbound_to_envelope(&request.inbound, recipient_keys, expires, nonce)
            .map_err(|error| DiscordBridgeError::Bridge(error.to_string()))
    }

    fn validate_inbound_request(
        &self,
        request: &DiscordInboundRequest,
    ) -> Result<(), DiscordBridgeError> {
        validate_did(&request.listener_did)?;
        if !self
            .config
            .authorized_listener_dids
            .contains(&request.listener_did)
        {
            return Err(DiscordBridgeError::UnauthorizedListener(
                request.listener_did.clone(),
            ));
        }

        let external_channel_id = request.inbound.external_channel_id.clone();
        let expected_target_did = self
            .config
            .channel_routes
            .get(&external_channel_id)
            .ok_or_else(|| DiscordBridgeError::UnknownRouteChannel(external_channel_id.clone()))?;

        if expected_target_did != &request.inbound.target_agent_did {
            return Err(DiscordBridgeError::RouteTargetMismatch {
                external_channel_id,
                expected_target_did: expected_target_did.clone(),
                provided_target_did: request.inbound.target_agent_did.clone(),
            });
        }
        Ok(())
    }

    pub fn process_outbound_with_approvals(
        &self,
        request: &BridgeOutboundRequest,
        approver_dids: Vec<String>,
    ) -> Result<DiscordOutboundDispatch, DiscordBridgeError> {
        self.validate_outbound_channel(&request.destination_channel_id)?;
        let approved_by = self.validate_approvals(approver_dids)?;
        let envelope = self
            .bridge
            .process_outbound(request)
            .map_err(|error| DiscordBridgeError::Bridge(error.to_string()))?;

        Ok(DiscordOutboundDispatch {
            envelope,
            approval: DiscordOutboundApproval {
                request_id: request.request_id.clone(),
                required_approvals: self.config.required_approvals,
                approved_by,
            },
        })
    }

    fn validate_outbound_channel(
        &self,
        destination_channel_id: &str,
    ) -> Result<(), DiscordBridgeError> {
        validate_non_empty(
            "bridge_outbound_request.destination_channel_id",
            destination_channel_id,
        )?;
        if !self
            .config
            .channel_routes
            .contains_key(destination_channel_id)
        {
            return Err(DiscordBridgeError::UnknownRouteChannel(
                destination_channel_id.to_owned(),
            ));
        }
        Ok(())
    }

    fn validate_approvals(
        &self,
        approver_dids: Vec<String>,
    ) -> Result<BTreeSet<String>, DiscordBridgeError> {
        let mut approved_by = BTreeSet::new();
        for approver_did in approver_dids {
            validate_did(&approver_did)?;
            if !self.config.authorized_approver_dids.contains(&approver_did) {
                return Err(DiscordBridgeError::UnauthorizedApprover(approver_did));
            }
            if !approved_by.insert(approver_did.clone()) {
                return Err(DiscordBridgeError::DuplicateApproval(approver_did));
            }
        }
        if approved_by.len() < self.config.required_approvals {
            return Err(DiscordBridgeError::InsufficientApprovals {
                required: self.config.required_approvals,
                provided: approved_by.len(),
            });
        }
        Ok(approved_by)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscordBridgeError {
    EmptyField(&'static str),
    InvalidDid(String),
    InvalidRequiredApprovals {
        required: usize,
        approver_count: usize,
    },
    UnauthorizedListener(String),
    UnauthorizedApprover(String),
    DuplicateApproval(String),
    InsufficientApprovals {
        required: usize,
        provided: usize,
    },
    UnknownRouteChannel(String),
    RouteTargetMismatch {
        external_channel_id: String,
        expected_target_did: String,
        provided_target_did: String,
    },
    Bridge(String),
}

impl fmt::Display for DiscordBridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::InvalidRequiredApprovals {
                required,
                approver_count,
            } => write!(
                f,
                "invalid required approvals {required}, approver count {approver_count}"
            ),
            Self::UnauthorizedListener(value) => write!(f, "unauthorized listener did: {value}"),
            Self::UnauthorizedApprover(value) => write!(f, "unauthorized approver did: {value}"),
            Self::DuplicateApproval(value) => write!(f, "duplicate approval from: {value}"),
            Self::InsufficientApprovals { required, provided } => write!(
                f,
                "insufficient approvals: required {required}, provided {provided}"
            ),
            Self::UnknownRouteChannel(value) => {
                write!(f, "discord channel route not found: {value}")
            }
            Self::RouteTargetMismatch {
                external_channel_id,
                expected_target_did,
                provided_target_did,
            } => write!(
                f,
                "discord channel route target mismatch for {external_channel_id}, expected {expected_target_did}, got {provided_target_did}"
            ),
            Self::Bridge(value) => write!(f, "discord bridge error: {value}"),
        }
    }
}

impl std::error::Error for DiscordBridgeError {}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), DiscordBridgeError> {
    if value.trim().is_empty() {
        return Err(DiscordBridgeError::EmptyField(field));
    }
    Ok(())
}

fn validate_did(value: &str) -> Result<(), DiscordBridgeError> {
    AgentDid::parse(value).map_err(|error| DiscordBridgeError::InvalidDid(error.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{DiscordBridgeConfig, DiscordBridgeEngine, DiscordBridgeError};
    use std::collections::{BTreeMap, BTreeSet};

    fn config() -> DiscordBridgeConfig {
        let mut channel_routes = BTreeMap::new();
        channel_routes.insert(
            "discord:channel:ops".to_owned(),
            "kamn:did:agent:target-1".to_owned(),
        );
        DiscordBridgeConfig {
            bridge_agent_did: "kamn:did:agent:bridge-discord-1".to_owned(),
            authorized_listener_dids: ["kamn:did:agent:listener-1".to_owned()]
                .into_iter()
                .collect::<BTreeSet<_>>(),
            authorized_approver_dids: [
                "kamn:did:agent:approver-1".to_owned(),
                "kamn:did:agent:approver-2".to_owned(),
            ]
            .into_iter()
            .collect::<BTreeSet<_>>(),
            required_approvals: 2,
            channel_routes,
        }
    }

    #[test]
    fn constructor_rejects_empty_approver_allowlist() {
        let mut config = config();
        config.authorized_approver_dids.clear();
        assert_eq!(
            DiscordBridgeEngine::new(config),
            Err(DiscordBridgeError::EmptyField("authorized_approver_dids"))
        );
    }

    #[test]
    fn constructor_rejects_invalid_required_approvals_threshold() {
        let mut config = config();
        config.required_approvals = 3;
        assert_eq!(
            DiscordBridgeEngine::new(config),
            Err(DiscordBridgeError::InvalidRequiredApprovals {
                required: 3,
                approver_count: 2,
            })
        );
    }
}
