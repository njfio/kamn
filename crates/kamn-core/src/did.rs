//! DID identity parsing, canonical document construction, and federated trust-handshake contracts.

use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fmt};

const KAMN_DID_PREFIX: &str = "kamn:did:";
const AGENT_DID_PREFIX: &str = "kamn:did:agent:";
const AGENT_DID_KEY_BINDING_MARKER: &str = "--keyh-";
const AGENT_DID_KEY_BINDING_HEX_LEN: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// Canonical KAMN DID wrapper for non-agent and agent role identifiers.
pub struct KamnDid(String);

impl KamnDid {
    /// Parses and validates a generic KAMN DID shape.
    pub fn parse(value: &str) -> Result<Self, KamnDidError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(KamnDidError::EmptyValue);
        }
        if !trimmed.starts_with(KAMN_DID_PREFIX) {
            return Err(KamnDidError::InvalidPrefix(trimmed.to_owned()));
        }
        let segments = trimmed.split(':').collect::<Vec<_>>();
        if segments.len() < 4 || segments.iter().any(|segment| segment.is_empty()) {
            return Err(KamnDidError::InvalidShape(trimmed.to_owned()));
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Returns the full DID string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Errors returned when parsing or validating a [`KamnDid`].
pub enum KamnDidError {
    /// DID input was empty after trimming.
    EmptyValue,
    /// DID did not start with required KAMN DID prefix.
    InvalidPrefix(String),
    /// DID segments were malformed.
    InvalidShape(String),
}

impl fmt::Display for KamnDidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyValue => write!(f, "kamn did must not be empty"),
            Self::InvalidPrefix(value) => write!(f, "invalid kamn did prefix: {value}"),
            Self::InvalidShape(value) => write!(f, "invalid kamn did shape: {value}"),
        }
    }
}

impl std::error::Error for KamnDidError {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// Canonical KAMN agent DID wrapper.
pub struct AgentDid(String);

impl AgentDid {
    /// Parses and validates a KAMN agent DID.
    pub fn parse(value: impl AsRef<str>) -> Result<Self, AgentDidError> {
        let value = value.as_ref();
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

    /// Returns the full DID string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the method-specific identifier component.
    pub fn method_specific_id(&self) -> &str {
        &self.0[AGENT_DID_PREFIX.len()..]
    }

    /// Returns DID-embedded key-binding fingerprint when present.
    pub fn key_binding_fingerprint(&self) -> Option<&str> {
        let method_specific_id = self.method_specific_id();
        let (base, fingerprint) = method_specific_id.rsplit_once(AGENT_DID_KEY_BINDING_MARKER)?;
        if base.is_empty() {
            return None;
        }
        if fingerprint.len() != AGENT_DID_KEY_BINDING_HEX_LEN {
            return None;
        }
        if !fingerprint.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return None;
        }
        Some(fingerprint)
    }

    /// Verifies that DID-embedded key-binding fingerprint matches `public_key_hex`.
    pub fn ensure_public_key_hex_binding(
        &self,
        public_key_hex: &str,
    ) -> Result<(), AgentDidKeyBindingError> {
        let expected = self
            .key_binding_fingerprint()
            .ok_or(AgentDidKeyBindingError::MissingKeyBinding)?;
        let actual = agent_did_key_binding_fingerprint_for_public_key_hex(public_key_hex)?;
        if !crate::constant_time_eq::constant_time_eq_bytes(actual.as_bytes(), expected.as_bytes())
        {
            return Err(AgentDidKeyBindingError::KeyBindingMismatch {
                expected: expected.to_owned(),
                actual,
            });
        }
        Ok(())
    }

    /// Builds an agent DID with deterministic key-binding fingerprint suffix.
    pub fn with_public_key_hex_binding(
        method_specific_id: &str,
        public_key_hex: &str,
    ) -> Result<Self, AgentDidKeyBindingError> {
        let normalized_method_specific_id = method_specific_id.trim();
        if normalized_method_specific_id.is_empty() {
            return Err(AgentDidKeyBindingError::InvalidMethodSpecificId(
                "agent did method-specific id must not be empty".to_owned(),
            ));
        }
        if normalized_method_specific_id.contains(AGENT_DID_KEY_BINDING_MARKER) {
            return Err(AgentDidKeyBindingError::InvalidMethodSpecificId(
                "agent did method-specific id must not include key-binding marker".to_owned(),
            ));
        }
        if !normalized_method_specific_id
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
        {
            return Err(AgentDidKeyBindingError::InvalidMethodSpecificId(
                "agent did method-specific id has invalid characters".to_owned(),
            ));
        }
        let key_binding_fingerprint =
            agent_did_key_binding_fingerprint_for_public_key_hex(public_key_hex)?;
        let rendered = format!(
            "{AGENT_DID_PREFIX}{normalized_method_specific_id}{AGENT_DID_KEY_BINDING_MARKER}{key_binding_fingerprint}"
        );
        AgentDid::parse(rendered.as_str()).map_err(|error| {
            AgentDidKeyBindingError::InvalidMethodSpecificId(format!(
                "agent did key-binding rendering failed validation: {error}"
            ))
        })
    }
}

impl fmt::Display for AgentDid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Errors returned when parsing or validating an [`AgentDid`].
pub enum AgentDidError {
    /// DID did not start with the required KAMN agent prefix.
    InvalidPrefix(String),
    /// DID prefix was present but method-specific id was missing.
    MissingMethodSpecificId,
    /// Method-specific id contained unsupported characters.
    InvalidCharacter(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Errors returned when validating or generating DID/public-key bindings.
pub enum AgentDidKeyBindingError {
    /// DID does not include key-binding fingerprint suffix.
    MissingKeyBinding,
    /// DID method-specific-id input is invalid for binding generation.
    InvalidMethodSpecificId(String),
    /// Public key hex could not be decoded.
    InvalidPublicKeyHex,
    /// DID fingerprint does not match derived public-key fingerprint.
    KeyBindingMismatch {
        /// Fingerprint embedded in DID.
        expected: String,
        /// Fingerprint derived from provided public key.
        actual: String,
    },
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

impl fmt::Display for AgentDidKeyBindingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingKeyBinding => write!(f, "agent did key binding is missing"),
            Self::InvalidMethodSpecificId(reason) => {
                write!(f, "invalid agent did method-specific id: {reason}")
            }
            Self::InvalidPublicKeyHex => write!(f, "invalid public key hex for did key binding"),
            Self::KeyBindingMismatch { expected, actual } => write!(
                f,
                "agent did key binding mismatch: expected={expected}, actual={actual}"
            ),
        }
    }
}

impl std::error::Error for AgentDidError {}
impl std::error::Error for AgentDidKeyBindingError {}

fn decode_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn decode_hex_bytes(value: &str) -> Option<Vec<u8>> {
    let bytes = value.as_bytes();
    if bytes.is_empty() || !bytes.len().is_multiple_of(2) {
        return None;
    }
    let mut output = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        let high = decode_hex_nibble(chunk[0])?;
        let low = decode_hex_nibble(chunk[1])?;
        output.push((high << 4) | low);
    }
    Some(output)
}

fn encode_hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn agent_did_key_binding_fingerprint_for_public_key_hex(
    public_key_hex: &str,
) -> Result<String, AgentDidKeyBindingError> {
    let bytes = decode_hex_bytes(public_key_hex.trim())
        .ok_or(AgentDidKeyBindingError::InvalidPublicKeyHex)?;
    let digest = Sha256::digest(bytes.as_slice());
    let fingerprint_bytes = &digest[..(AGENT_DID_KEY_BINDING_HEX_LEN / 2)];
    Ok(encode_hex_lower(fingerprint_bytes))
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Metadata extension embedded in KAMN DID documents.
pub struct AgentDidMetadata {
    /// Agent type classification (for example `autonomous`).
    pub agent_type: String,
    /// Model family identifier for the agent.
    pub model_family: String,
    /// Declared capabilities supported by the agent.
    pub capabilities: Vec<String>,
    /// Optional operator DID or name associated with the agent.
    pub operator: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// DID verification method entry.
pub struct DidVerificationMethod {
    /// Verification method identifier.
    pub id: String,
    /// Verification method type name.
    pub type_name: String,
    /// DID controller for this method.
    pub controller: String,
    /// Multibase-encoded public key.
    pub public_key_multibase: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// DID service endpoint entry.
pub struct DidService {
    /// Service identifier.
    pub id: String,
    /// Service type name.
    pub type_name: String,
    /// Canonical service endpoint URI.
    pub service_endpoint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Canonical DID document representation used by KAMN.
pub struct DidDocument {
    /// Ordered JSON-LD context list.
    pub context: Vec<String>,
    /// DID identifier.
    pub id: String,
    /// DID controller identifier.
    pub controller: String,
    /// Verification methods published by the DID.
    pub verification_method: Vec<DidVerificationMethod>,
    /// Authentication method references.
    pub authentication: Vec<String>,
    /// Assertion method references.
    pub assertion_method: Vec<String>,
    /// Service endpoints attached to the DID.
    pub service: Vec<DidService>,
    /// KAMN-specific metadata extension.
    pub metadata: AgentDidMetadata,
}

/// Trust-store abstraction used by federated DID handshake evaluation.
pub trait FederatedDidTrustStore {
    /// Returns true when `subject_did` is trusted for `network`.
    fn is_trusted(&self, network: &str, subject_did: &str) -> bool;
}

#[derive(Debug, Clone, Default)]
/// In-memory trust store for federated DID evaluation.
pub struct InMemoryFederatedDidTrustStore {
    entries: BTreeSet<(String, String)>,
}

impl InMemoryFederatedDidTrustStore {
    /// Creates an empty trust store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a trust store from `(network, did)` entries.
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

    /// Inserts a trusted DID entry for a network.
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
/// Input envelope for federated DID handshake policy evaluation.
pub struct FederatedDidHandshakeInput {
    /// Unique handshake identifier.
    pub handshake_id: String,
    /// Subject DID being validated.
    pub subject_did: String,
    /// Local network identifier.
    pub local_network: String,
    /// Remote network identifier.
    pub remote_network: String,
    /// Resolver version used for DID resolution.
    pub resolver_version: String,
    /// Whether signature policy checks passed.
    pub signature_policy_passed: bool,
    /// Whether nonce progression is monotonic.
    pub nonce_monotonic: bool,
    /// Whether downgrade signal was detected.
    pub downgrade_detected: bool,
    /// Whether partition sequence is monotonic.
    pub partition_sequence_monotonic: bool,
    /// Required quorum for federated acceptance.
    pub required_quorum: u16,
    /// Quorum received during evaluation.
    pub received_quorum: u16,
}

impl FederatedDidHandshakeInput {
    #[allow(clippy::too_many_arguments)]
    /// Constructs and validates a federated DID handshake input payload.
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
/// Positive federated DID handshake decision payload.
pub struct FederatedDidHandshakeDecision {
    /// Handshake identifier.
    pub handshake_id: String,
    /// Subject DID that was accepted.
    pub subject_did: String,
    /// Local network identifier.
    pub local_network: String,
    /// Remote network identifier.
    pub remote_network: String,
}

impl FederatedDidHandshakeDecision {
    /// Returns canonical reason code for successful decisions.
    pub fn reason_code(&self) -> &'static str {
        "federated_did_handshake_ok"
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Federated DID handshake failure taxonomy.
pub enum FederatedDidHandshakeError {
    /// One of the required string fields was empty.
    EmptyField(&'static str),
    /// Required quorum value was invalid.
    InvalidRequiredQuorum {
        /// Required quorum value provided by caller.
        required: u16,
    },
    /// Resolver version was missing for this handshake.
    ResolverVersionMissing {
        /// Handshake identifier.
        handshake_id: String,
    },
    /// Trust store did not contain `(network, did)` entry.
    TrustStoreMiss {
        /// Subject DID not found in trust store.
        subject_did: String,
        /// Network queried in trust store.
        network: String,
    },
    /// Signature-policy checks failed.
    SignaturePolicyFailed {
        /// Handshake identifier.
        handshake_id: String,
    },
    /// Received quorum was below required quorum.
    QuorumShortfall {
        /// Required quorum threshold.
        required: u16,
        /// Received quorum count.
        received: u16,
    },
    /// Nonce replay or non-monotonic sequence detected.
    NonceReplayDetected {
        /// Handshake identifier.
        handshake_id: String,
    },
    /// Partition sequence replay/non-monotonic signal detected.
    PartitionSequenceReplayDetected {
        /// Handshake identifier.
        handshake_id: String,
    },
    /// Resolver downgrade signal detected.
    DowngradeDetected {
        /// Handshake identifier.
        handshake_id: String,
    },
}

impl FederatedDidHandshakeError {
    /// Returns stable reason code for telemetry and policy lanes.
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
/// Evaluator that applies federated DID handshake policy against a trust store.
pub struct FederatedDidHandshakeEvaluator<T: FederatedDidTrustStore> {
    trust_store: T,
}

impl<T: FederatedDidTrustStore> FederatedDidHandshakeEvaluator<T> {
    /// Creates an evaluator using the provided trust store implementation.
    pub fn new(trust_store: T) -> Self {
        Self { trust_store }
    }

    /// Evaluates federated DID handshake input and returns acceptance decision.
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
/// DID document canonicalization and validation errors.
pub enum DidDocumentError {
    /// Public key input was empty.
    EmptyPublicKey,
    /// Agent metadata type was empty.
    EmptyAgentType,
    /// Model family metadata was empty.
    EmptyModelFamily,
    /// Capability list was empty.
    MissingCapabilities,
    /// At least one capability entry was empty.
    InvalidCapability,
    /// Service endpoint failed canonical validation.
    InvalidServiceEndpoint(String),
    /// Verification method algorithm set was invalid.
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

/// Canonicalizes and validates a DID service endpoint.
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

/// Validates verification method algorithm set used in DID documents.
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

/// Builds canonical KAMN DID document from validated inputs.
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
        validate_did_verification_method_algorithms, AgentDid, AgentDidError,
        AgentDidKeyBindingError, AgentDidMetadata, DidDocumentError, KamnDid, KamnDidError,
    };
    const SOURCE: &str = include_str!("did.rs");

    fn ensure_public_key_hex_binding_source() -> &'static str {
        let function_start = SOURCE
            .find("pub fn ensure_public_key_hex_binding(")
            .expect("function must exist");
        let function_end = SOURCE[function_start..]
            .find(
                "\n    /// Builds an agent DID with deterministic key-binding fingerprint suffix.",
            )
            .map(|offset| function_start + offset)
            .expect("function boundary must exist");
        &SOURCE[function_start..function_end]
    }

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
    fn parse_kamn_did_accepts_owner_and_agent_dids() {
        let owner = KamnDid::parse("kamn:did:owner:sender-1")
            .expect("owner did should parse as generic kamn did");
        assert_eq!(owner.as_str(), "kamn:did:owner:sender-1");

        let agent = KamnDid::parse("kamn:did:agent:agent-1")
            .expect("agent did should parse as generic kamn did");
        assert_eq!(agent.as_str(), "kamn:did:agent:agent-1");
    }

    #[test]
    fn parse_kamn_did_rejects_invalid_prefix_and_shape() {
        assert_eq!(
            KamnDid::parse("did:example:alice"),
            Err(KamnDidError::InvalidPrefix("did:example:alice".to_owned()))
        );
        assert_eq!(
            KamnDid::parse("kamn:did:"),
            Err(KamnDidError::InvalidShape("kamn:did:".to_owned()))
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

    const TEST_PUBLIC_KEY_HEX: &str =
        "025f6ceceac37540cf6ef5f09d4f62c05f0b8f57fe6d8ae32a8f13f4a2eb6e940d";
    const TEST_PUBLIC_KEY_HEX_ALT: &str =
        "02dbf4fcb77ef6a9f2d0f5f0d7c7faaf02f53b724f4cfe6fe1d95ff5a6d4bf8132";

    #[test]
    fn unit_agent_did_with_public_key_hex_binding_embeds_fingerprint_suffix() {
        let did = AgentDid::with_public_key_hex_binding("agent-1", TEST_PUBLIC_KEY_HEX)
            .expect("bound did should render");
        assert!(did.as_str().starts_with("kamn:did:agent:agent-1--keyh-"));
        let fingerprint = did
            .key_binding_fingerprint()
            .expect("bound did should expose fingerprint");
        assert_eq!(fingerprint.len(), 32);
    }

    #[test]
    fn regression_agent_did_key_binding_verification_rejects_missing_binding() {
        // Regression: #6109
        let did = AgentDid::parse("kamn:did:agent:agent-1").expect("did should parse");
        assert_eq!(
            did.ensure_public_key_hex_binding(TEST_PUBLIC_KEY_HEX),
            Err(AgentDidKeyBindingError::MissingKeyBinding)
        );
    }

    #[test]
    fn regression_agent_did_key_binding_verification_rejects_mismatched_public_key() {
        // Regression: #6109
        let did = AgentDid::with_public_key_hex_binding("agent-2", TEST_PUBLIC_KEY_HEX)
            .expect("bound did should render");
        let error = did
            .ensure_public_key_hex_binding(TEST_PUBLIC_KEY_HEX_ALT)
            .expect_err("mismatched public key should fail binding verification");
        let AgentDidKeyBindingError::KeyBindingMismatch { expected, actual } = error else {
            panic!("expected key binding mismatch error");
        };
        assert!(
            expected.len() == 32 && actual.len() == 32,
            "mismatch error should preserve expected/actual fingerprint payloads"
        );
    }

    #[test]
    fn regression_agent_did_key_binding_verification_accepts_matching_public_key() {
        // Regression: #6109
        let did = AgentDid::with_public_key_hex_binding("agent-3", TEST_PUBLIC_KEY_HEX)
            .expect("bound did should render");
        did.ensure_public_key_hex_binding(TEST_PUBLIC_KEY_HEX)
            .expect("matching public key should satisfy did binding");
    }

    #[test]
    fn regression_agent_did_key_binding_verification_accepts_parsed_bound_did() {
        let rendered = AgentDid::with_public_key_hex_binding("agent-5", TEST_PUBLIC_KEY_HEX)
            .expect("bound did should render")
            .to_string();
        let parsed = AgentDid::parse(rendered.as_str()).expect("rendered bound did should parse");
        parsed
            .ensure_public_key_hex_binding(TEST_PUBLIC_KEY_HEX)
            .expect("parsed bound did should preserve key-binding verification");
    }

    #[test]
    fn regression_agent_did_key_binding_verification_rejects_malformed_public_key_hex() {
        let did = AgentDid::with_public_key_hex_binding("agent-4", TEST_PUBLIC_KEY_HEX)
            .expect("bound did should render");
        assert_eq!(
            did.ensure_public_key_hex_binding("zz-not-hex"),
            Err(AgentDidKeyBindingError::InvalidPublicKeyHex)
        );
    }

    #[test]
    fn regression_requires_constant_time_agent_did_key_binding_compare() {
        let function_source = ensure_public_key_hex_binding_source();
        let direct_pattern = ["if actual", "!=", " expected {"].concat();
        assert!(
            function_source.contains("crate::constant_time_eq::constant_time_eq_bytes("),
            "agent did key-binding verification must use constant-time compare"
        );
        assert!(
            !function_source.contains(direct_pattern.as_str()),
            "agent did key-binding verification must not use direct equality"
        );
    }
}
