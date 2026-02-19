//! Cross-chain ingress/egress bridge routing and approval contracts.
//!
//! This module validates listener and approver identities, enforces configured
//! route maps, and normalizes bridge payloads for Ethereum and Solana lanes.

use crate::{
    AgentDid, AllowAllBridgePolicy, BridgeAdapterEngine, BridgeInboundEnvelope,
    BridgeOutboundEnvelope, BridgeOutboundRequest, BridgePlatform, CanonicalMessageEnvelope,
    NormalizedInboundMessage, PassThroughBridgeAdapter,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

const CROSS_CHAIN_INVALID_BRIDGE_AGENT_DID_REASON_CODE: &str =
    "cross_chain_bridge_invalid_bridge_agent_did";
const CROSS_CHAIN_INVALID_LISTENER_DID_REASON_CODE: &str =
    "cross_chain_bridge_invalid_listener_did";
const CROSS_CHAIN_INVALID_APPROVER_DID_REASON_CODE: &str =
    "cross_chain_bridge_invalid_approver_did";
const CROSS_CHAIN_INVALID_ETHEREUM_ROUTE_TARGET_DID_REASON_CODE: &str =
    "cross_chain_bridge_invalid_route_target_did";
const CROSS_CHAIN_INVALID_SOLANA_ROUTE_TARGET_DID_REASON_CODE: &str =
    "cross_chain_bridge_invalid_route_target_did";
const CROSS_CHAIN_INVALID_INBOUND_LISTENER_DID_REASON_CODE: &str =
    "cross_chain_bridge_invalid_inbound_listener_did";
const CROSS_CHAIN_INVALID_INBOUND_TARGET_DID_REASON_CODE: &str =
    "cross_chain_bridge_invalid_inbound_target_did";
const CROSS_CHAIN_INVALID_OUTBOUND_APPROVER_DID_REASON_CODE: &str =
    "cross_chain_bridge_invalid_outbound_approver_did";

/// Supported external networks for cross-chain bridge routes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CrossChainNetwork {
    /// Ethereum bridge lane.
    Ethereum,
    /// Solana bridge lane.
    Solana,
}

impl CrossChainNetwork {
    fn label(self) -> &'static str {
        match self {
            Self::Ethereum => "ethereum",
            Self::Solana => "solana",
        }
    }
}

/// Static bridge configuration for listener/approver controls and routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossChainBridgeConfig {
    /// DID used as the bridge service identity.
    pub bridge_agent_did: String,
    /// Listener DIDs authorized to submit inbound observations.
    pub authorized_listener_dids: BTreeSet<String>,
    /// Approver DIDs authorized for outbound dispatch quorum.
    pub authorized_approver_dids: BTreeSet<String>,
    /// Required unique approvals before dispatching outbound envelopes.
    pub required_approvals: usize,
    /// Route map from external Ethereum channel id to target DID.
    pub ethereum_routes: BTreeMap<String, String>,
    /// Route map from external Solana channel id to target DID.
    pub solana_routes: BTreeMap<String, String>,
}

/// Inbound bridge observation submitted by an authorized listener.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossChainInboundRequest {
    /// Listener DID that observed the inbound event.
    pub listener_did: String,
    /// Unix timestamp when the event was observed.
    pub observed_at_unix: u64,
    /// External network lane used for routing.
    pub chain: CrossChainNetwork,
    /// Raw inbound bridge envelope.
    pub inbound: BridgeInboundEnvelope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CrossChainBridgeConfigValidated {
    bridge_agent_did: AgentDid,
    authorized_listener_dids: BTreeSet<AgentDid>,
    authorized_approver_dids: BTreeSet<AgentDid>,
    required_approvals: usize,
    ethereum_routes: BTreeMap<String, AgentDid>,
    solana_routes: BTreeMap<String, AgentDid>,
}

impl TryFrom<&CrossChainBridgeConfig> for CrossChainBridgeConfigValidated {
    type Error = CrossChainBridgeError;

    fn try_from(config: &CrossChainBridgeConfig) -> Result<Self, Self::Error> {
        let bridge_agent_did = parse_agent_did(
            config.bridge_agent_did.as_str(),
            "bridge_agent_did",
            CROSS_CHAIN_INVALID_BRIDGE_AGENT_DID_REASON_CODE,
        )?;

        if config.authorized_listener_dids.is_empty() {
            return Err(CrossChainBridgeError::EmptyField(
                "authorized_listener_dids",
            ));
        }
        let mut authorized_listener_dids = BTreeSet::new();
        for listener_did in &config.authorized_listener_dids {
            authorized_listener_dids.insert(parse_agent_did(
                listener_did.as_str(),
                "authorized_listener_dids[]",
                CROSS_CHAIN_INVALID_LISTENER_DID_REASON_CODE,
            )?);
        }

        if config.authorized_approver_dids.is_empty() {
            return Err(CrossChainBridgeError::EmptyField(
                "authorized_approver_dids",
            ));
        }
        let mut authorized_approver_dids = BTreeSet::new();
        for approver_did in &config.authorized_approver_dids {
            authorized_approver_dids.insert(parse_agent_did(
                approver_did.as_str(),
                "authorized_approver_dids[]",
                CROSS_CHAIN_INVALID_APPROVER_DID_REASON_CODE,
            )?);
        }

        if config.required_approvals == 0
            || config.required_approvals > authorized_approver_dids.len()
        {
            return Err(CrossChainBridgeError::InvalidRequiredApprovals {
                required: config.required_approvals,
                approver_count: authorized_approver_dids.len(),
            });
        }

        if config.ethereum_routes.is_empty() {
            return Err(CrossChainBridgeError::EmptyField("ethereum_routes"));
        }
        let mut ethereum_routes = BTreeMap::new();
        for (channel_id, target_did) in &config.ethereum_routes {
            validate_non_empty("ethereum_routes.channel_id", channel_id)?;
            ethereum_routes.insert(
                channel_id.clone(),
                parse_agent_did(
                    target_did.as_str(),
                    "ethereum_routes.target_did",
                    CROSS_CHAIN_INVALID_ETHEREUM_ROUTE_TARGET_DID_REASON_CODE,
                )?,
            );
        }

        if config.solana_routes.is_empty() {
            return Err(CrossChainBridgeError::EmptyField("solana_routes"));
        }
        let mut solana_routes = BTreeMap::new();
        for (channel_id, target_did) in &config.solana_routes {
            validate_non_empty("solana_routes.channel_id", channel_id)?;
            solana_routes.insert(
                channel_id.clone(),
                parse_agent_did(
                    target_did.as_str(),
                    "solana_routes.target_did",
                    CROSS_CHAIN_INVALID_SOLANA_ROUTE_TARGET_DID_REASON_CODE,
                )?,
            );
        }

        Ok(Self {
            bridge_agent_did,
            authorized_listener_dids,
            authorized_approver_dids,
            required_approvals: config.required_approvals,
            ethereum_routes,
            solana_routes,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CrossChainInboundRequestValidated {
    listener_did: AgentDid,
    target_agent_did: AgentDid,
}

impl TryFrom<&CrossChainInboundRequest> for CrossChainInboundRequestValidated {
    type Error = CrossChainBridgeError;

    fn try_from(request: &CrossChainInboundRequest) -> Result<Self, Self::Error> {
        let listener_did = parse_agent_did(
            request.listener_did.as_str(),
            "cross_chain_inbound_request.listener_did",
            CROSS_CHAIN_INVALID_INBOUND_LISTENER_DID_REASON_CODE,
        )?;
        let target_agent_did = parse_agent_did(
            request.inbound.target_agent_did.as_str(),
            "cross_chain_inbound_request.inbound.target_agent_did",
            CROSS_CHAIN_INVALID_INBOUND_TARGET_DID_REASON_CODE,
        )?;
        Ok(Self {
            listener_did,
            target_agent_did,
        })
    }
}

/// Approval quorum material for an outbound dispatch request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossChainOutboundApproval {
    /// External network lane being dispatched.
    pub chain: CrossChainNetwork,
    /// Outbound request identifier.
    pub request_id: String,
    /// Required approval threshold for dispatch.
    pub required_approvals: usize,
    /// Unique approver DIDs that approved this dispatch.
    pub approved_by: BTreeSet<String>,
}

/// Outbound bridge envelope paired with approval quorum evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossChainOutboundDispatch {
    /// Final outbound envelope produced by bridge adapter processing.
    pub envelope: BridgeOutboundEnvelope,
    /// Approval proof material used for dispatch authorization.
    pub approval: CrossChainOutboundApproval,
}

/// Engine coordinating inbound normalization and outbound gated dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossChainBridgeEngine {
    config: CrossChainBridgeConfigValidated,
    ethereum_bridge: BridgeAdapterEngine<PassThroughBridgeAdapter, AllowAllBridgePolicy>,
    solana_bridge: BridgeAdapterEngine<PassThroughBridgeAdapter, AllowAllBridgePolicy>,
}

impl CrossChainBridgeEngine {
    /// Constructs a bridge engine from validated configuration.
    pub fn new(config: CrossChainBridgeConfig) -> Result<Self, CrossChainBridgeError> {
        let config = CrossChainBridgeConfigValidated::try_from(&config)?;

        let ethereum_adapter = PassThroughBridgeAdapter::new(
            BridgePlatform::Custom(CrossChainNetwork::Ethereum.label().to_owned()),
            config.bridge_agent_did.as_str(),
        )
        .map_err(|error| CrossChainBridgeError::Bridge(error.to_string()))?;

        let solana_adapter = PassThroughBridgeAdapter::new(
            BridgePlatform::Custom(CrossChainNetwork::Solana.label().to_owned()),
            config.bridge_agent_did.as_str(),
        )
        .map_err(|error| CrossChainBridgeError::Bridge(error.to_string()))?;

        Ok(Self {
            config,
            ethereum_bridge: BridgeAdapterEngine::new(
                ethereum_adapter,
                AllowAllBridgePolicy::new(),
            ),
            solana_bridge: BridgeAdapterEngine::new(solana_adapter, AllowAllBridgePolicy::new()),
        })
    }

    /// Validates and normalizes an inbound bridge request.
    pub fn process_inbound(
        &self,
        request: &CrossChainInboundRequest,
    ) -> Result<NormalizedInboundMessage, CrossChainBridgeError> {
        let _ = self.validate_inbound_request(request)?;
        self.bridge_engine(request.chain)
            .process_inbound(&request.inbound, request.observed_at_unix)
            .map_err(|error| CrossChainBridgeError::Bridge(error.to_string()))
    }

    /// Converts a validated inbound request directly into canonical envelope form.
    pub fn process_inbound_to_envelope(
        &self,
        request: &CrossChainInboundRequest,
        recipient_keys: Vec<String>,
        expires: &str,
        nonce: u64,
    ) -> Result<CanonicalMessageEnvelope, CrossChainBridgeError> {
        let _ = self.validate_inbound_request(request)?;
        self.bridge_engine(request.chain)
            .process_inbound_to_envelope(
                &request.inbound,
                request.observed_at_unix,
                recipient_keys,
                expires,
                nonce,
            )
            .map_err(|error| CrossChainBridgeError::Bridge(error.to_string()))
    }

    fn validate_inbound_request(
        &self,
        request: &CrossChainInboundRequest,
    ) -> Result<CrossChainInboundRequestValidated, CrossChainBridgeError> {
        let validated = CrossChainInboundRequestValidated::try_from(request)?;
        if !self
            .config
            .authorized_listener_dids
            .contains(&validated.listener_did)
        {
            return Err(CrossChainBridgeError::UnauthorizedListener(
                validated.listener_did.as_str().to_owned(),
            ));
        }

        let routes = self.route_map(request.chain);
        let channel_id = request.inbound.external_channel_id.clone();
        let expected_target_did =
            routes
                .get(&channel_id)
                .ok_or_else(|| CrossChainBridgeError::UnknownRouteChannel {
                    chain: request.chain,
                    channel_id: channel_id.clone(),
                })?;

        if expected_target_did != &validated.target_agent_did {
            return Err(CrossChainBridgeError::RouteTargetMismatch {
                chain: request.chain,
                external_channel_id: channel_id,
                expected_target_did: expected_target_did.as_str().to_owned(),
                provided_target_did: validated.target_agent_did.as_str().to_owned(),
            });
        }
        Ok(validated)
    }

    /// Validates approvals and dispatches an outbound bridge request.
    pub fn process_outbound_with_approvals(
        &self,
        chain: CrossChainNetwork,
        request: &BridgeOutboundRequest,
        approver_dids: Vec<String>,
    ) -> Result<CrossChainOutboundDispatch, CrossChainBridgeError> {
        self.validate_outbound_channel(chain, &request.destination_channel_id)?;
        let approved_by = self.validate_approvals(approver_dids)?;
        let envelope = self
            .bridge_engine(chain)
            .process_outbound(request)
            .map_err(|error| CrossChainBridgeError::Bridge(error.to_string()))?;

        Ok(CrossChainOutboundDispatch {
            envelope,
            approval: CrossChainOutboundApproval {
                chain,
                request_id: request.request_id.clone(),
                required_approvals: self.config.required_approvals,
                approved_by,
            },
        })
    }

    fn route_map(&self, chain: CrossChainNetwork) -> &BTreeMap<String, AgentDid> {
        match chain {
            CrossChainNetwork::Ethereum => &self.config.ethereum_routes,
            CrossChainNetwork::Solana => &self.config.solana_routes,
        }
    }

    fn bridge_engine(
        &self,
        chain: CrossChainNetwork,
    ) -> &BridgeAdapterEngine<PassThroughBridgeAdapter, AllowAllBridgePolicy> {
        match chain {
            CrossChainNetwork::Ethereum => &self.ethereum_bridge,
            CrossChainNetwork::Solana => &self.solana_bridge,
        }
    }

    fn validate_outbound_channel(
        &self,
        chain: CrossChainNetwork,
        destination_channel_id: &str,
    ) -> Result<(), CrossChainBridgeError> {
        validate_non_empty(
            "bridge_outbound_request.destination_channel_id",
            destination_channel_id,
        )?;
        if !self.route_map(chain).contains_key(destination_channel_id) {
            return Err(CrossChainBridgeError::UnknownRouteChannel {
                chain,
                channel_id: destination_channel_id.to_owned(),
            });
        }
        Ok(())
    }

    fn validate_approvals(
        &self,
        approver_dids: Vec<String>,
    ) -> Result<BTreeSet<String>, CrossChainBridgeError> {
        let mut approved_by = BTreeSet::new();
        for approver_did in approver_dids {
            let validated_approver_did = parse_agent_did(
                approver_did.as_str(),
                "approver_dids[]",
                CROSS_CHAIN_INVALID_OUTBOUND_APPROVER_DID_REASON_CODE,
            )?;
            if !self
                .config
                .authorized_approver_dids
                .contains(&validated_approver_did)
            {
                return Err(CrossChainBridgeError::UnauthorizedApprover(
                    validated_approver_did.as_str().to_owned(),
                ));
            }
            let canonical_approver_did = validated_approver_did.as_str().to_owned();
            if !approved_by.insert(canonical_approver_did.clone()) {
                return Err(CrossChainBridgeError::DuplicateApproval(
                    canonical_approver_did,
                ));
            }
        }
        if approved_by.len() < self.config.required_approvals {
            return Err(CrossChainBridgeError::InsufficientApprovals {
                required: self.config.required_approvals,
                provided: approved_by.len(),
            });
        }
        Ok(approved_by)
    }
}

/// Error surface for cross-chain bridge configuration and processing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrossChainBridgeError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// DID value failed validation.
    InvalidDid {
        /// Input field carrying the DID value.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
        /// Canonical parser detail.
        detail: String,
    },
    /// Required approvals value was invalid for configured approver set.
    InvalidRequiredApprovals {
        /// Required approval threshold requested.
        required: usize,
        /// Number of configured approvers.
        approver_count: usize,
    },
    /// Listener DID is not authorized.
    UnauthorizedListener(String),
    /// Approver DID is not authorized.
    UnauthorizedApprover(String),
    /// Duplicate approval from the same approver.
    DuplicateApproval(String),
    /// Provided approvals are below quorum threshold.
    InsufficientApprovals {
        /// Required approval threshold.
        required: usize,
        /// Unique approvals provided.
        provided: usize,
    },
    /// Route channel id was not found for the selected chain.
    UnknownRouteChannel {
        /// Chain lane where route lookup failed.
        chain: CrossChainNetwork,
        /// External channel identifier.
        channel_id: String,
    },
    /// Route target DID did not match inbound envelope target DID.
    RouteTargetMismatch {
        /// Chain lane where mismatch occurred.
        chain: CrossChainNetwork,
        /// External channel identifier.
        external_channel_id: String,
        /// Route table target DID.
        expected_target_did: String,
        /// Target DID provided by inbound envelope.
        provided_target_did: String,
    },
    /// Downstream bridge adapter error.
    Bridge(String),
}

impl fmt::Display for CrossChainBridgeError {
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
            Self::UnknownRouteChannel { chain, channel_id } => {
                write!(f, "{} route not found: {channel_id}", chain.label())
            }
            Self::RouteTargetMismatch {
                chain,
                external_channel_id,
                expected_target_did,
                provided_target_did,
            } => write!(
                f,
                "{} route target mismatch for {external_channel_id}, expected {expected_target_did}, got {provided_target_did}",
                chain.label()
            ),
            Self::Bridge(value) => write!(f, "cross-chain bridge error: {value}"),
        }
    }
}

impl std::error::Error for CrossChainBridgeError {}

impl CrossChainBridgeError {
    /// Stable reason taxonomy for cross-chain bridge errors.
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::EmptyField(_) => "cross_chain_bridge_empty_field",
            Self::InvalidDid { reason_code, .. } => reason_code,
            Self::InvalidRequiredApprovals { .. } => {
                "cross_chain_bridge_invalid_required_approvals"
            }
            Self::UnauthorizedListener(_) => "cross_chain_bridge_unauthorized_listener",
            Self::UnauthorizedApprover(_) => "cross_chain_bridge_unauthorized_approver",
            Self::DuplicateApproval(_) => "cross_chain_bridge_duplicate_approval",
            Self::InsufficientApprovals { .. } => "cross_chain_bridge_insufficient_approvals",
            Self::UnknownRouteChannel { .. } => "cross_chain_bridge_unknown_route_channel",
            Self::RouteTargetMismatch { .. } => "cross_chain_bridge_route_target_mismatch",
            Self::Bridge(_) => "cross_chain_bridge_adapter_error",
        }
    }
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), CrossChainBridgeError> {
    if value.trim().is_empty() {
        return Err(CrossChainBridgeError::EmptyField(field));
    }
    Ok(())
}

fn parse_agent_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<AgentDid, CrossChainBridgeError> {
    AgentDid::parse(value).map_err(|error| CrossChainBridgeError::InvalidDid {
        field,
        reason_code,
        detail: error.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::{CrossChainBridgeConfig, CrossChainBridgeEngine, CrossChainBridgeError};
    use std::collections::{BTreeMap, BTreeSet};

    fn config() -> CrossChainBridgeConfig {
        let mut ethereum_routes = BTreeMap::new();
        ethereum_routes.insert(
            "ethereum:sepolia:contract:escrow-v1".to_owned(),
            "kamn:did:agent:target-eth".to_owned(),
        );

        let mut solana_routes = BTreeMap::new();
        solana_routes.insert(
            "solana:devnet:program:task-v1".to_owned(),
            "kamn:did:agent:target-sol".to_owned(),
        );

        CrossChainBridgeConfig {
            bridge_agent_did: "kamn:did:agent:bridge-crosschain-1".to_owned(),
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
            ethereum_routes,
            solana_routes,
        }
    }

    #[test]
    fn constructor_rejects_empty_ethereum_routes() {
        let mut config = config();
        config.ethereum_routes.clear();
        assert_eq!(
            CrossChainBridgeEngine::new(config),
            Err(CrossChainBridgeError::EmptyField("ethereum_routes"))
        );
    }

    #[test]
    fn constructor_rejects_invalid_required_approvals_threshold() {
        let mut config = config();
        config.required_approvals = 3;
        assert_eq!(
            CrossChainBridgeEngine::new(config),
            Err(CrossChainBridgeError::InvalidRequiredApprovals {
                required: 3,
                approver_count: 2,
            })
        );
    }
}
