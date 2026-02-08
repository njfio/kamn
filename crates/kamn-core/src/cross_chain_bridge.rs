use crate::{
    AgentDid, AllowAllBridgePolicy, BridgeAdapterEngine, BridgeInboundEnvelope,
    BridgeOutboundEnvelope, BridgeOutboundRequest, BridgePlatform, CanonicalMessageEnvelope,
    NormalizedInboundMessage, PassThroughBridgeAdapter,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CrossChainNetwork {
    Ethereum,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossChainBridgeConfig {
    pub bridge_agent_did: String,
    pub authorized_listener_dids: BTreeSet<String>,
    pub authorized_approver_dids: BTreeSet<String>,
    pub required_approvals: usize,
    pub ethereum_routes: BTreeMap<String, String>,
    pub solana_routes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossChainInboundRequest {
    pub listener_did: String,
    pub observed_at_unix: u64,
    pub chain: CrossChainNetwork,
    pub inbound: BridgeInboundEnvelope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossChainOutboundApproval {
    pub chain: CrossChainNetwork,
    pub request_id: String,
    pub required_approvals: usize,
    pub approved_by: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossChainOutboundDispatch {
    pub envelope: BridgeOutboundEnvelope,
    pub approval: CrossChainOutboundApproval,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossChainBridgeEngine {
    config: CrossChainBridgeConfig,
    ethereum_bridge: BridgeAdapterEngine<PassThroughBridgeAdapter, AllowAllBridgePolicy>,
    solana_bridge: BridgeAdapterEngine<PassThroughBridgeAdapter, AllowAllBridgePolicy>,
}

impl CrossChainBridgeEngine {
    pub fn new(config: CrossChainBridgeConfig) -> Result<Self, CrossChainBridgeError> {
        validate_did(&config.bridge_agent_did)?;

        if config.authorized_listener_dids.is_empty() {
            return Err(CrossChainBridgeError::EmptyField(
                "authorized_listener_dids",
            ));
        }
        for listener_did in &config.authorized_listener_dids {
            validate_did(listener_did)?;
        }

        if config.authorized_approver_dids.is_empty() {
            return Err(CrossChainBridgeError::EmptyField(
                "authorized_approver_dids",
            ));
        }
        for approver_did in &config.authorized_approver_dids {
            validate_did(approver_did)?;
        }
        if config.required_approvals == 0
            || config.required_approvals > config.authorized_approver_dids.len()
        {
            return Err(CrossChainBridgeError::InvalidRequiredApprovals {
                required: config.required_approvals,
                approver_count: config.authorized_approver_dids.len(),
            });
        }

        if config.ethereum_routes.is_empty() {
            return Err(CrossChainBridgeError::EmptyField("ethereum_routes"));
        }
        for (channel_id, target_did) in &config.ethereum_routes {
            validate_non_empty("ethereum_routes.channel_id", channel_id)?;
            validate_did(target_did)?;
        }

        if config.solana_routes.is_empty() {
            return Err(CrossChainBridgeError::EmptyField("solana_routes"));
        }
        for (channel_id, target_did) in &config.solana_routes {
            validate_non_empty("solana_routes.channel_id", channel_id)?;
            validate_did(target_did)?;
        }

        let ethereum_adapter = PassThroughBridgeAdapter::new(
            BridgePlatform::Custom(CrossChainNetwork::Ethereum.label().to_owned()),
            &config.bridge_agent_did,
        )
        .map_err(|error| CrossChainBridgeError::Bridge(error.to_string()))?;

        let solana_adapter = PassThroughBridgeAdapter::new(
            BridgePlatform::Custom(CrossChainNetwork::Solana.label().to_owned()),
            &config.bridge_agent_did,
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

    pub fn process_inbound(
        &self,
        request: &CrossChainInboundRequest,
    ) -> Result<NormalizedInboundMessage, CrossChainBridgeError> {
        self.validate_inbound_request(request)?;
        self.bridge_engine(request.chain)
            .process_inbound(&request.inbound, request.observed_at_unix)
            .map_err(|error| CrossChainBridgeError::Bridge(error.to_string()))
    }

    pub fn process_inbound_to_envelope(
        &self,
        request: &CrossChainInboundRequest,
        recipient_keys: Vec<String>,
        expires: &str,
        nonce: u64,
    ) -> Result<CanonicalMessageEnvelope, CrossChainBridgeError> {
        self.validate_inbound_request(request)?;
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
    ) -> Result<(), CrossChainBridgeError> {
        validate_did(&request.listener_did)?;
        if !self
            .config
            .authorized_listener_dids
            .contains(&request.listener_did)
        {
            return Err(CrossChainBridgeError::UnauthorizedListener(
                request.listener_did.clone(),
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

        if expected_target_did != &request.inbound.target_agent_did {
            return Err(CrossChainBridgeError::RouteTargetMismatch {
                chain: request.chain,
                external_channel_id: channel_id,
                expected_target_did: expected_target_did.clone(),
                provided_target_did: request.inbound.target_agent_did.clone(),
            });
        }
        Ok(())
    }

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

    fn route_map(&self, chain: CrossChainNetwork) -> &BTreeMap<String, String> {
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
            validate_did(&approver_did)?;
            if !self.config.authorized_approver_dids.contains(&approver_did) {
                return Err(CrossChainBridgeError::UnauthorizedApprover(approver_did));
            }
            if !approved_by.insert(approver_did.clone()) {
                return Err(CrossChainBridgeError::DuplicateApproval(approver_did));
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrossChainBridgeError {
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
    UnknownRouteChannel {
        chain: CrossChainNetwork,
        channel_id: String,
    },
    RouteTargetMismatch {
        chain: CrossChainNetwork,
        external_channel_id: String,
        expected_target_did: String,
        provided_target_did: String,
    },
    Bridge(String),
}

impl fmt::Display for CrossChainBridgeError {
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

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), CrossChainBridgeError> {
    if value.trim().is_empty() {
        return Err(CrossChainBridgeError::EmptyField(field));
    }
    Ok(())
}

fn validate_did(value: &str) -> Result<(), CrossChainBridgeError> {
    AgentDid::parse(value).map_err(|error| CrossChainBridgeError::InvalidDid(error.to_string()))?;
    Ok(())
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
