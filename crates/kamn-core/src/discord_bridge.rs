use crate::{
    AgentDid, AllowAllBridgePolicy, BridgeAdapterEngine, BridgeInboundEnvelope,
    BridgeOutboundEnvelope, BridgeOutboundRequest, BridgePlatform, CanonicalMessageEnvelope,
    NormalizedInboundMessage, PassThroughBridgeAdapter,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

const DISCORD_BRIDGE_INVALID_BRIDGE_AGENT_DID_REASON_CODE: &str =
    "discord_bridge_invalid_bridge_agent_did";
const DISCORD_BRIDGE_INVALID_LISTENER_DID_REASON_CODE: &str = "discord_bridge_invalid_listener_did";
const DISCORD_BRIDGE_INVALID_APPROVER_DID_REASON_CODE: &str = "discord_bridge_invalid_approver_did";
const DISCORD_BRIDGE_INVALID_ROUTE_TARGET_DID_REASON_CODE: &str =
    "discord_bridge_invalid_route_target_did";
const DISCORD_BRIDGE_INVALID_INBOUND_LISTENER_DID_REASON_CODE: &str =
    "discord_bridge_invalid_inbound_listener_did";
const DISCORD_BRIDGE_INVALID_INBOUND_TARGET_DID_REASON_CODE: &str =
    "discord_bridge_invalid_inbound_target_did";
const DISCORD_BRIDGE_INVALID_OUTBOUND_APPROVER_DID_REASON_CODE: &str =
    "discord_bridge_invalid_outbound_approver_did";

/// Discord bridge configuration for listeners, approvers, and channel routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscordBridgeConfig {
    /// DID used by the bridge agent identity.
    pub bridge_agent_did: String,
    /// Listener DIDs allowed to submit inbound events.
    pub authorized_listener_dids: BTreeSet<String>,
    /// Approver DIDs allowed to approve outbound dispatches.
    pub authorized_approver_dids: BTreeSet<String>,
    /// Number of unique approvals required per outbound request.
    pub required_approvals: usize,
    /// Mapping from external channel id to target DID.
    pub channel_routes: BTreeMap<String, String>,
}

/// Inbound request metadata for Discord listener submissions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscordInboundRequest {
    /// Listener DID submitting the inbound request.
    pub listener_did: String,
    /// Unix timestamp when event was observed.
    pub observed_at_unix: u64,
    /// Normalized inbound bridge envelope.
    pub inbound: BridgeInboundEnvelope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DiscordBridgeConfigValidated {
    bridge_agent_did: AgentDid,
    authorized_listener_dids: BTreeSet<AgentDid>,
    authorized_approver_dids: BTreeSet<AgentDid>,
    required_approvals: usize,
    channel_routes: BTreeMap<String, AgentDid>,
}

impl TryFrom<&DiscordBridgeConfig> for DiscordBridgeConfigValidated {
    type Error = DiscordBridgeError;

    fn try_from(config: &DiscordBridgeConfig) -> Result<Self, Self::Error> {
        let bridge_agent_did = parse_agent_did(
            config.bridge_agent_did.as_str(),
            "bridge_agent_did",
            DISCORD_BRIDGE_INVALID_BRIDGE_AGENT_DID_REASON_CODE,
        )?;
        if config.authorized_listener_dids.is_empty() {
            return Err(DiscordBridgeError::EmptyField("authorized_listener_dids"));
        }
        let mut authorized_listener_dids = BTreeSet::new();
        for listener_did in &config.authorized_listener_dids {
            authorized_listener_dids.insert(parse_agent_did(
                listener_did.as_str(),
                "authorized_listener_dids[]",
                DISCORD_BRIDGE_INVALID_LISTENER_DID_REASON_CODE,
            )?);
        }

        if config.authorized_approver_dids.is_empty() {
            return Err(DiscordBridgeError::EmptyField("authorized_approver_dids"));
        }
        let mut authorized_approver_dids = BTreeSet::new();
        for approver_did in &config.authorized_approver_dids {
            authorized_approver_dids.insert(parse_agent_did(
                approver_did.as_str(),
                "authorized_approver_dids[]",
                DISCORD_BRIDGE_INVALID_APPROVER_DID_REASON_CODE,
            )?);
        }
        if config.required_approvals == 0
            || config.required_approvals > authorized_approver_dids.len()
        {
            return Err(DiscordBridgeError::InvalidRequiredApprovals {
                required: config.required_approvals,
                approver_count: authorized_approver_dids.len(),
            });
        }

        if config.channel_routes.is_empty() {
            return Err(DiscordBridgeError::EmptyField("channel_routes"));
        }
        let mut channel_routes = BTreeMap::new();
        for (external_channel_id, target_did) in &config.channel_routes {
            validate_non_empty("channel_routes.external_channel_id", external_channel_id)?;
            channel_routes.insert(
                external_channel_id.clone(),
                parse_agent_did(
                    target_did.as_str(),
                    "channel_routes.target_did",
                    DISCORD_BRIDGE_INVALID_ROUTE_TARGET_DID_REASON_CODE,
                )?,
            );
        }

        Ok(Self {
            bridge_agent_did,
            authorized_listener_dids,
            authorized_approver_dids,
            required_approvals: config.required_approvals,
            channel_routes,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DiscordInboundRequestValidated {
    listener_did: AgentDid,
    target_agent_did: AgentDid,
}

impl TryFrom<&DiscordInboundRequest> for DiscordInboundRequestValidated {
    type Error = DiscordBridgeError;

    fn try_from(request: &DiscordInboundRequest) -> Result<Self, Self::Error> {
        let listener_did = parse_agent_did(
            request.listener_did.as_str(),
            "discord_inbound_request.listener_did",
            DISCORD_BRIDGE_INVALID_INBOUND_LISTENER_DID_REASON_CODE,
        )?;
        let target_agent_did = parse_agent_did(
            request.inbound.target_agent_did.as_str(),
            "discord_inbound_request.inbound.target_agent_did",
            DISCORD_BRIDGE_INVALID_INBOUND_TARGET_DID_REASON_CODE,
        )?;
        Ok(Self {
            listener_did,
            target_agent_did,
        })
    }
}

/// Approval evidence for an outbound Discord dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscordOutboundApproval {
    /// Outbound request identifier.
    pub request_id: String,
    /// Required number of approvals.
    pub required_approvals: usize,
    /// DID set that approved this request.
    pub approved_by: BTreeSet<String>,
}

/// Outbound dispatch payload coupled with approval evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscordOutboundDispatch {
    /// Outbound envelope for platform dispatch.
    pub envelope: BridgeOutboundEnvelope,
    /// Approval summary for dispatch authorization.
    pub approval: DiscordOutboundApproval,
}

/// Discord bridge engine for inbound normalization and outbound approval-gated dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscordBridgeEngine {
    config: DiscordBridgeConfigValidated,
    bridge: BridgeAdapterEngine<PassThroughBridgeAdapter, AllowAllBridgePolicy>,
}

impl DiscordBridgeEngine {
    /// Creates a Discord bridge engine after validating config and route policy.
    pub fn new(config: DiscordBridgeConfig) -> Result<Self, DiscordBridgeError> {
        let config = DiscordBridgeConfigValidated::try_from(&config)?;

        let adapter = PassThroughBridgeAdapter::new(
            BridgePlatform::Discord,
            config.bridge_agent_did.as_str(),
        )
        .map_err(|error| DiscordBridgeError::Bridge(error.to_string()))?;
        let bridge = BridgeAdapterEngine::new(adapter, AllowAllBridgePolicy::new());
        Ok(Self { config, bridge })
    }

    /// Validates and processes inbound requests into normalized message form.
    pub fn process_inbound(
        &self,
        request: &DiscordInboundRequest,
    ) -> Result<NormalizedInboundMessage, DiscordBridgeError> {
        let _ = self.validate_inbound_request(request)?;

        self.bridge
            .process_inbound(&request.inbound, request.observed_at_unix)
            .map_err(|error| DiscordBridgeError::Bridge(error.to_string()))
    }

    /// Validates and processes inbound requests into canonical message-envelope form.
    pub fn process_inbound_to_envelope(
        &self,
        request: &DiscordInboundRequest,
        recipient_keys: Vec<String>,
        expires: &str,
        nonce: u64,
    ) -> Result<CanonicalMessageEnvelope, DiscordBridgeError> {
        let _ = self.validate_inbound_request(request)?;
        self.bridge
            .process_inbound_to_envelope(
                &request.inbound,
                request.observed_at_unix,
                recipient_keys,
                expires,
                nonce,
            )
            .map_err(|error| DiscordBridgeError::Bridge(error.to_string()))
    }

    fn validate_inbound_request(
        &self,
        request: &DiscordInboundRequest,
    ) -> Result<DiscordInboundRequestValidated, DiscordBridgeError> {
        let validated = DiscordInboundRequestValidated::try_from(request)?;
        if !self
            .config
            .authorized_listener_dids
            .contains(&validated.listener_did)
        {
            return Err(DiscordBridgeError::UnauthorizedListener(
                validated.listener_did.as_str().to_owned(),
            ));
        }

        let external_channel_id = request.inbound.external_channel_id.clone();
        let expected_target_did = self
            .config
            .channel_routes
            .get(&external_channel_id)
            .ok_or_else(|| DiscordBridgeError::UnknownRouteChannel(external_channel_id.clone()))?;

        if expected_target_did != &validated.target_agent_did {
            return Err(DiscordBridgeError::RouteTargetMismatch {
                external_channel_id,
                expected_target_did: expected_target_did.as_str().to_owned(),
                provided_target_did: validated.target_agent_did.as_str().to_owned(),
            });
        }
        Ok(validated)
    }

    /// Processes outbound request after validating route and required approver set.
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
            let validated_approver_did = parse_agent_did(
                approver_did.as_str(),
                "approver_dids[]",
                DISCORD_BRIDGE_INVALID_OUTBOUND_APPROVER_DID_REASON_CODE,
            )?;
            if !self
                .config
                .authorized_approver_dids
                .contains(&validated_approver_did)
            {
                return Err(DiscordBridgeError::UnauthorizedApprover(
                    validated_approver_did.as_str().to_owned(),
                ));
            }
            let canonical_approver_did = validated_approver_did.as_str().to_owned();
            if !approved_by.insert(canonical_approver_did.clone()) {
                return Err(DiscordBridgeError::DuplicateApproval(
                    canonical_approver_did,
                ));
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

/// Errors emitted by Discord bridge validation and dispatch flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscordBridgeError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// DID failed validation.
    InvalidDid {
        /// Input field carrying the DID value.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
        /// Canonical parser detail.
        detail: String,
    },
    /// Required approval count is invalid for configured approver set.
    InvalidRequiredApprovals {
        /// Required approval count.
        required: usize,
        /// Number of configured approvers.
        approver_count: usize,
    },
    /// Listener DID is not authorized.
    UnauthorizedListener(String),
    /// Approver DID is not authorized.
    UnauthorizedApprover(String),
    /// Duplicate approval from same approver DID.
    DuplicateApproval(String),
    /// Approval set did not meet required threshold.
    InsufficientApprovals {
        /// Required approval count.
        required: usize,
        /// Provided unique approvals.
        provided: usize,
    },
    /// Route channel id is unknown.
    UnknownRouteChannel(String),
    /// Target DID in payload does not match route mapping.
    RouteTargetMismatch {
        /// External channel identifier.
        external_channel_id: String,
        /// Expected target DID.
        expected_target_did: String,
        /// Provided target DID.
        provided_target_did: String,
    },
    /// Underlying bridge adapter error.
    Bridge(String),
}

impl fmt::Display for DiscordBridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::InvalidDid {
                field,
                reason_code,
                detail,
            } => write!(f, "invalid did field {field}: {reason_code} ({detail})"),
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

impl DiscordBridgeError {
    /// Stable reason taxonomy for discord bridge errors.
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::EmptyField(_) => "discord_bridge_empty_field",
            Self::InvalidDid { reason_code, .. } => reason_code,
            Self::InvalidRequiredApprovals { .. } => "discord_bridge_invalid_required_approvals",
            Self::UnauthorizedListener(_) => "discord_bridge_unauthorized_listener",
            Self::UnauthorizedApprover(_) => "discord_bridge_unauthorized_approver",
            Self::DuplicateApproval(_) => "discord_bridge_duplicate_approval",
            Self::InsufficientApprovals { .. } => "discord_bridge_insufficient_approvals",
            Self::UnknownRouteChannel(_) => "discord_bridge_unknown_route_channel",
            Self::RouteTargetMismatch { .. } => "discord_bridge_route_target_mismatch",
            Self::Bridge(_) => "discord_bridge_adapter_error",
        }
    }
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), DiscordBridgeError> {
    if value.trim().is_empty() {
        return Err(DiscordBridgeError::EmptyField(field));
    }
    Ok(())
}

fn parse_agent_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<AgentDid, DiscordBridgeError> {
    AgentDid::parse(value).map_err(|error| DiscordBridgeError::InvalidDid {
        field,
        reason_code,
        detail: error.to_string(),
    })
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
