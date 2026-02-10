use std::{collections::BTreeSet, fmt};

const AGENT_DID_PREFIX: &str = "kamn:did:agent:";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentDid(String);

impl AgentDid {
    pub fn parse(value: &str) -> Result<Self, AgentDidError> {
        if !value.starts_with(AGENT_DID_PREFIX) {
            return Err(AgentDidError::InvalidPrefix(value.to_owned()));
        }
        let method_specific_id = &value[AGENT_DID_PREFIX.len()..];
        if method_specific_id.is_empty() {
            return Err(AgentDidError::MissingMethodSpecificId);
        }
        if !method_specific_id
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
        {
            return Err(AgentDidError::InvalidCharacter(
                method_specific_id.to_owned(),
            ));
        }

        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn method_specific_id(&self) -> &str {
        &self.0[AGENT_DID_PREFIX.len()..]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentDidError {
    InvalidPrefix(String),
    MissingMethodSpecificId,
    InvalidCharacter(String),
}

impl fmt::Display for AgentDidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefix(value) => write!(f, "invalid agent did prefix: {value}"),
            Self::MissingMethodSpecificId => {
                write!(f, "agent did method-specific id must not be empty")
            }
            Self::InvalidCharacter(value) => {
                write!(f, "agent did has invalid characters: {value}")
            }
        }
    }
}

impl std::error::Error for AgentDidError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentDidMetadata {
    pub agent_type: String,
    pub model_family: String,
    pub capabilities: Vec<String>,
    pub operator: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DidVerificationMethod {
    pub id: String,
    pub type_name: String,
    pub controller: String,
    pub public_key_multibase: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DidService {
    pub id: String,
    pub type_name: String,
    pub service_endpoint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DidDocument {
    pub context: Vec<String>,
    pub id: String,
    pub controller: String,
    pub verification_method: Vec<DidVerificationMethod>,
    pub authentication: Vec<String>,
    pub assertion_method: Vec<String>,
    pub service: Vec<DidService>,
    pub metadata: AgentDidMetadata,
}

pub trait FederatedDidTrustStore {
    fn is_trusted(&self, network: &str, subject_did: &str) -> bool;
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryFederatedDidTrustStore {
    entries: BTreeSet<(String, String)>,
}

impl InMemoryFederatedDidTrustStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_entries<I, N, D>(entries: I) -> Self
    where
        I: IntoIterator<Item = (N, D)>,
        N: Into<String>,
        D: Into<String>,
    {
        let mut trust_store = Self::new();
        for (network, subject_did) in entries {
            trust_store.insert(network.into().as_str(), subject_did.into().as_str());
        }
        trust_store
    }

    pub fn insert(&mut self, network: &str, subject_did: &str) {
        self.entries
            .insert((network.trim().to_owned(), subject_did.trim().to_owned()));
    }
}

impl FederatedDidTrustStore for InMemoryFederatedDidTrustStore {
    fn is_trusted(&self, network: &str, subject_did: &str) -> bool {
        self.entries
            .contains(&(network.trim().to_owned(), subject_did.trim().to_owned()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FederatedDidHandshakeInput {
    pub handshake_id: String,
    pub subject_did: String,
    pub local_network: String,
    pub remote_network: String,
    pub resolver_version: String,
    pub signature_policy_passed: bool,
    pub nonce_monotonic: bool,
    pub downgrade_detected: bool,
    pub partition_sequence_monotonic: bool,
    pub required_quorum: u16,
    pub received_quorum: u16,
}

impl FederatedDidHandshakeInput {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        handshake_id: &str,
        subject_did: &str,
        local_network: &str,
        remote_network: &str,
        resolver_version: &str,
        signature_policy_passed: bool,
        nonce_monotonic: bool,
        downgrade_detected: bool,
        partition_sequence_monotonic: bool,
        required_quorum: u16,
        received_quorum: u16,
    ) -> Result<Self, FederatedDidHandshakeError> {
        if handshake_id.trim().is_empty() {
            return Err(FederatedDidHandshakeError::EmptyField("handshake_id"));
        }
        if subject_did.trim().is_empty() {
            return Err(FederatedDidHandshakeError::EmptyField("subject_did"));
        }
        if local_network.trim().is_empty() {
            return Err(FederatedDidHandshakeError::EmptyField("local_network"));
        }
        if remote_network.trim().is_empty() {
            return Err(FederatedDidHandshakeError::EmptyField("remote_network"));
        }
        if required_quorum == 0 {
            return Err(FederatedDidHandshakeError::InvalidRequiredQuorum {
                required: required_quorum,
            });
        }

        Ok(Self {
            handshake_id: handshake_id.trim().to_owned(),
            subject_did: subject_did.trim().to_owned(),
            local_network: local_network.trim().to_owned(),
            remote_network: remote_network.trim().to_owned(),
            resolver_version: resolver_version.trim().to_owned(),
            signature_policy_passed,
            nonce_monotonic,
            downgrade_detected,
            partition_sequence_monotonic,
            required_quorum,
            received_quorum,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FederatedDidHandshakeDecision {
    pub handshake_id: String,
    pub subject_did: String,
    pub local_network: String,
    pub remote_network: String,
}

impl FederatedDidHandshakeDecision {
    pub fn reason_code(&self) -> &'static str {
        "federated_did_handshake_ok"
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FederatedDidHandshakeError {
    EmptyField(&'static str),
    InvalidRequiredQuorum {
        required: u16,
    },
    ResolverVersionMissing {
        handshake_id: String,
    },
    TrustStoreMiss {
        subject_did: String,
        network: String,
    },
    SignaturePolicyFailed {
        handshake_id: String,
    },
    QuorumShortfall {
        required: u16,
        received: u16,
    },
    NonceReplayDetected {
        handshake_id: String,
    },
    PartitionSequenceReplayDetected {
        handshake_id: String,
    },
    DowngradeDetected {
        handshake_id: String,
    },
}

impl FederatedDidHandshakeError {
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::EmptyField(_) => "federated_did_handshake_invalid_input",
            Self::InvalidRequiredQuorum { .. } => "federated_did_handshake_invalid_quorum",
            Self::ResolverVersionMissing { .. } => "federated_did_handshake_resolver_missing",
            Self::TrustStoreMiss { .. } => "federated_did_handshake_trust_store_miss",
            Self::SignaturePolicyFailed { .. } => "federated_did_handshake_signature_policy_failed",
            Self::QuorumShortfall { .. } => "federated_did_handshake_quorum_shortfall",
            Self::NonceReplayDetected { .. } => "federated_did_handshake_nonce_replay",
            Self::PartitionSequenceReplayDetected { .. } => {
                "federated_did_handshake_partition_replay"
            }
            Self::DowngradeDetected { .. } => "federated_did_handshake_downgrade_detected",
        }
    }
}

impl fmt::Display for FederatedDidHandshakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "federated did handshake field is empty: {field}"),
            Self::InvalidRequiredQuorum { required } => {
                write!(f, "invalid required quorum for federated did handshake: {required}")
            }
            Self::ResolverVersionMissing { handshake_id } => write!(
                f,
                "resolver version missing for federated did handshake: {handshake_id}"
            ),
            Self::TrustStoreMiss {
                subject_did,
                network,
            } => write!(
                f,
                "federated did handshake trust-store miss for did {subject_did} on network {network}"
            ),
            Self::SignaturePolicyFailed { handshake_id } => write!(
                f,
                "federated did handshake signature policy failed: {handshake_id}"
            ),
            Self::QuorumShortfall { required, received } => write!(
                f,
                "federated did handshake quorum shortfall: required {required}, received {received}"
            ),
            Self::NonceReplayDetected { handshake_id } => {
                write!(f, "federated did handshake nonce replay detected: {handshake_id}")
            }
            Self::PartitionSequenceReplayDetected { handshake_id } => write!(
                f,
                "federated did handshake partition sequence replay detected: {handshake_id}"
            ),
            Self::DowngradeDetected { handshake_id } => write!(
                f,
                "federated did handshake downgrade detected: {handshake_id}"
            ),
        }
    }
}

impl std::error::Error for FederatedDidHandshakeError {}

#[derive(Debug, Clone)]
pub struct FederatedDidHandshakeEvaluator<T: FederatedDidTrustStore> {
    trust_store: T,
}

impl<T: FederatedDidTrustStore> FederatedDidHandshakeEvaluator<T> {
    pub fn new(trust_store: T) -> Self {
        Self { trust_store }
    }

    pub fn evaluate(
        &mut self,
        input: FederatedDidHandshakeInput,
    ) -> Result<FederatedDidHandshakeDecision, FederatedDidHandshakeError> {
        if !self
            .trust_store
            .is_trusted(&input.remote_network, &input.subject_did)
        {
            return Err(FederatedDidHandshakeError::TrustStoreMiss {
                subject_did: input.subject_did,
                network: input.remote_network,
            });
        }
        if input.resolver_version.trim().is_empty() {
            return Err(FederatedDidHandshakeError::ResolverVersionMissing {
                handshake_id: input.handshake_id,
            });
        }
        if !input.signature_policy_passed {
            return Err(FederatedDidHandshakeError::SignaturePolicyFailed {
                handshake_id: input.handshake_id,
            });
        }
        if input.received_quorum < input.required_quorum {
            return Err(FederatedDidHandshakeError::QuorumShortfall {
                required: input.required_quorum,
                received: input.received_quorum,
            });
        }
        if !input.nonce_monotonic {
            return Err(FederatedDidHandshakeError::NonceReplayDetected {
                handshake_id: input.handshake_id,
            });
        }
        if !input.partition_sequence_monotonic {
            return Err(
                FederatedDidHandshakeError::PartitionSequenceReplayDetected {
                    handshake_id: input.handshake_id,
                },
            );
        }
        if input.downgrade_detected {
            return Err(FederatedDidHandshakeError::DowngradeDetected {
                handshake_id: input.handshake_id,
            });
        }

        Ok(FederatedDidHandshakeDecision {
            handshake_id: input.handshake_id,
            subject_did: input.subject_did,
            local_network: input.local_network,
            remote_network: input.remote_network,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DidDocumentError {
    EmptyPublicKey,
    EmptyAgentType,
    EmptyModelFamily,
    MissingCapabilities,
    InvalidCapability,
    InvalidServiceEndpoint(String),
    InvalidVerificationMethodAlgorithm(String),
}

impl fmt::Display for DidDocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPublicKey => write!(f, "public key must not be empty"),
            Self::EmptyAgentType => write!(f, "agent_type must not be empty"),
            Self::EmptyModelFamily => write!(f, "model_family must not be empty"),
            Self::MissingCapabilities => write!(f, "at least one capability is required"),
            Self::InvalidCapability => write!(f, "capability entries must not be empty"),
            Self::InvalidServiceEndpoint(message) => {
                write!(f, "invalid service endpoint: {message}")
            }
            Self::InvalidVerificationMethodAlgorithm(message) => {
                write!(f, "invalid verification method algorithm: {message}")
            }
        }
    }
}

impl std::error::Error for DidDocumentError {}

pub fn canonical_service_endpoint(raw_endpoint: &str) -> Result<String, DidDocumentError> {
    let trimmed = raw_endpoint.trim();
    if trimmed.is_empty() {
        return Err(DidDocumentError::InvalidServiceEndpoint(
            "service endpoint must not be empty".to_owned(),
        ));
    }
    if trimmed.contains('?') || trimmed.contains('#') {
        return Err(DidDocumentError::InvalidServiceEndpoint(
            "service endpoint must not include query or fragment".to_owned(),
        ));
    }

    let (scheme, remainder) = trimmed.split_once("://").ok_or_else(|| {
        DidDocumentError::InvalidServiceEndpoint(
            "service endpoint must include scheme://authority/path".to_owned(),
        )
    })?;
    if !scheme.eq_ignore_ascii_case("kamn") {
        return Err(DidDocumentError::InvalidServiceEndpoint(
            "service endpoint scheme must be kamn".to_owned(),
        ));
    }

    let (authority, path) = remainder.split_once('/').ok_or_else(|| {
        DidDocumentError::InvalidServiceEndpoint(
            "service endpoint must include authority and path".to_owned(),
        )
    })?;
    if !authority.eq_ignore_ascii_case("messaging") {
        return Err(DidDocumentError::InvalidServiceEndpoint(
            "service endpoint authority must be messaging".to_owned(),
        ));
    }
    if path.is_empty() || path.contains('/') {
        return Err(DidDocumentError::InvalidServiceEndpoint(
            "service endpoint path must be a single segment".to_owned(),
        ));
    }

    let normalized_path = path.to_ascii_lowercase();
    if !normalized_path
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
    {
        return Err(DidDocumentError::InvalidServiceEndpoint(
            "service endpoint path contains invalid characters".to_owned(),
        ));
    }

    Ok(format!("kamn://messaging/{normalized_path}"))
}

pub fn validate_did_verification_method_algorithms(
    algorithms: &[String],
) -> Result<(), DidDocumentError> {
    if algorithms.is_empty() {
        return Err(DidDocumentError::InvalidVerificationMethodAlgorithm(
            "at least one verification method algorithm is required".to_owned(),
        ));
    }

    let mut normalized_algorithms: Vec<String> = Vec::with_capacity(algorithms.len());
    for algorithm in algorithms {
        let normalized = algorithm.trim();
        if normalized.is_empty() {
            return Err(DidDocumentError::InvalidVerificationMethodAlgorithm(
                "verification method algorithm entries must not be empty".to_owned(),
            ));
        }
        if normalized != "Multikey" && normalized != "MultikeyV2" {
            return Err(DidDocumentError::InvalidVerificationMethodAlgorithm(
                format!("unsupported verification method algorithm: {normalized}"),
            ));
        }
        normalized_algorithms.push(normalized.to_owned());
    }

    let first_algorithm = &normalized_algorithms[0];
    if normalized_algorithms
        .iter()
        .any(|algorithm| algorithm != first_algorithm)
    {
        return Err(DidDocumentError::InvalidVerificationMethodAlgorithm(
            "mixed verification method algorithms are not allowed".to_owned(),
        ));
    }

    Ok(())
}

pub fn canonical_did_document(
    did: &AgentDid,
    public_key_multibase: &str,
    metadata: AgentDidMetadata,
) -> Result<DidDocument, DidDocumentError> {
    if public_key_multibase.trim().is_empty() {
        return Err(DidDocumentError::EmptyPublicKey);
    }
    if metadata.agent_type.trim().is_empty() {
        return Err(DidDocumentError::EmptyAgentType);
    }
    if metadata.model_family.trim().is_empty() {
        return Err(DidDocumentError::EmptyModelFamily);
    }
    if metadata.capabilities.is_empty() {
        return Err(DidDocumentError::MissingCapabilities);
    }
    if metadata
        .capabilities
        .iter()
        .any(|capability| capability.trim().is_empty())
    {
        return Err(DidDocumentError::InvalidCapability);
    }

    let key_id = format!("{}#keys-1", did.as_str());
    let service_id = format!("{}#messaging", did.as_str());
    let service_endpoint =
        canonical_service_endpoint(&format!("kamn://messaging/{}", did.method_specific_id()))?;
    let algorithm_set = vec!["Multikey".to_owned()];
    validate_did_verification_method_algorithms(&algorithm_set)?;

    Ok(DidDocument {
        context: vec![
            "https://www.w3.org/ns/did/v1.1".to_owned(),
            "https://kamn.network/context/v1".to_owned(),
        ],
        id: did.as_str().to_owned(),
        controller: did.as_str().to_owned(),
        verification_method: vec![DidVerificationMethod {
            id: key_id.clone(),
            type_name: "Multikey".to_owned(),
            controller: did.as_str().to_owned(),
            public_key_multibase: public_key_multibase.to_owned(),
        }],
        authentication: vec![key_id.clone()],
        assertion_method: vec![key_id],
        service: vec![DidService {
            id: service_id,
            type_name: "KAMNMessaging".to_owned(),
            service_endpoint,
        }],
        metadata,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        canonical_did_document, canonical_service_endpoint,
        validate_did_verification_method_algorithms, AgentDid, AgentDidError, AgentDidMetadata,
        DidDocumentError,
    };

    fn metadata() -> AgentDidMetadata {
        AgentDidMetadata {
            agent_type: "autonomous".to_owned(),
            model_family: "claude-4".to_owned(),
            capabilities: vec!["text".to_owned()],
            operator: None,
        }
    }

    #[test]
    fn parse_rejects_invalid_characters() {
        assert_eq!(
            AgentDid::parse("kamn:did:agent:Agent_1"),
            Err(AgentDidError::InvalidCharacter("Agent_1".to_owned()))
        );
    }

    #[test]
    fn canonical_document_requires_capabilities() {
        let did = match AgentDid::parse("kamn:did:agent:agent-1") {
            Ok(value) => value,
            Err(error) => panic!("did parse failed: {error}"),
        };
        let mut invalid = metadata();
        invalid.capabilities.clear();
        assert_eq!(
            canonical_did_document(&did, "z6Mkey", invalid),
            Err(DidDocumentError::MissingCapabilities)
        );
    }

    #[test]
    fn canonical_service_endpoint_normalizes_scheme_authority_and_path() {
        assert_eq!(
            canonical_service_endpoint("  KAMN://MESSAGING/Agent_1  "),
            Ok("kamn://messaging/agent_1".to_owned())
        );
    }

    #[test]
    fn canonical_service_endpoint_rejects_query_and_fragment() {
        assert_eq!(
            canonical_service_endpoint("kamn://messaging/agent-1?channel=dm"),
            Err(DidDocumentError::InvalidServiceEndpoint(
                "service endpoint must not include query or fragment".to_owned()
            ))
        );
    }

    #[test]
    fn validate_did_verification_method_algorithms_accepts_uniform_multikey_set() {
        let algorithms = vec!["Multikey".to_owned(), "Multikey".to_owned()];
        assert_eq!(
            validate_did_verification_method_algorithms(&algorithms),
            Ok(())
        );
    }

    #[test]
    fn validate_did_verification_method_algorithms_rejects_mixed_algorithms() {
        let algorithms = vec!["Multikey".to_owned(), "MultikeyV2".to_owned()];
        assert_eq!(
            validate_did_verification_method_algorithms(&algorithms),
            Err(DidDocumentError::InvalidVerificationMethodAlgorithm(
                "mixed verification method algorithms are not allowed".to_owned()
            ))
        );
    }
}
