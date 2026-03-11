//! DID-focused shared canonical value surface for cross-crate consumers.

use sha2::{Digest, Sha256};
use std::fmt;

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
        if !constant_time_eq_bytes(actual.as_bytes(), expected.as_bytes()) {
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


fn constant_time_eq_bytes(lhs: &[u8], rhs: &[u8]) -> bool {
    if lhs.len() != rhs.len() {
        return false;
    }

    let mut diff = 0u8;
    for (&left, &right) in lhs.iter().zip(rhs.iter()) {
        diff |= left ^ right;
    }
    diff == 0
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
/// Wrapper error for canonical DID parse helpers.
pub enum SharedDidParseError {
    /// Input was empty after canonical trim.
    EmptyInput,
    /// Underlying agent DID parse failure.
    Agent(AgentDidError),
    /// Underlying generic KAMN DID parse failure.
    Kamn(KamnDidError),
}

impl fmt::Display for SharedDidParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "did input must not be empty"),
            Self::Agent(error) => write!(f, "agent did parse failed: {error}"),
            Self::Kamn(error) => write!(f, "kamn did parse failed: {error}"),
        }
    }
}

impl std::error::Error for SharedDidParseError {}

fn normalize_non_empty(value: &str) -> Result<&str, SharedDidParseError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(SharedDidParseError::EmptyInput);
    }
    Ok(normalized)
}

/// Parses agent DID inputs with canonical trim semantics.
pub fn parse_agent_did_canonical(value: &str) -> Result<AgentDid, SharedDidParseError> {
    let normalized = normalize_non_empty(value)?;
    AgentDid::parse(normalized).map_err(SharedDidParseError::Agent)
}

/// Parses generic KAMN DID inputs with canonical trim semantics.
pub fn parse_kamn_did_canonical(value: &str) -> Result<KamnDid, SharedDidParseError> {
    let normalized = normalize_non_empty(value)?;
    KamnDid::parse(normalized).map_err(SharedDidParseError::Kamn)
}

#[cfg(test)]
mod tests {
    use super::{
        parse_agent_did_canonical, parse_kamn_did_canonical, AgentDid, AgentDidError, KamnDid,
        SharedDidParseError,
    };

    #[test]
    fn shared_agent_did_parse_accepts_valid_did() {
        let parsed = AgentDid::parse("kamn:did:agent:shared-alpha");
        assert_eq!(
            parsed.expect("expected valid shared did parse").as_str(),
            "kamn:did:agent:shared-alpha"
        );
    }

    #[test]
    fn shared_agent_did_parse_rejects_invalid_prefix() {
        let parsed = AgentDid::parse("did:example:agent");
        assert!(matches!(parsed, Err(AgentDidError::InvalidPrefix(_))));
    }

    #[test]
    fn shared_agent_did_parse_rejects_missing_method_specific_id() {
        let parsed = AgentDid::parse("kamn:did:agent:");
        assert!(matches!(
            parsed,
            Err(AgentDidError::MissingMethodSpecificId)
        ));
    }

    #[test]
    fn shared_kamn_did_parse_accepts_non_agent_kamn_did() {
        let parsed = KamnDid::parse("kamn:did:operator:node-1").expect("kamn did should parse");
        assert_eq!(parsed.as_str(), "kamn:did:operator:node-1");
    }

    #[test]
    fn canonical_parse_helpers_reject_empty_inputs() {
        assert_eq!(
            parse_agent_did_canonical(" "),
            Err(SharedDidParseError::EmptyInput)
        );
        assert_eq!(
            parse_kamn_did_canonical(""),
            Err(SharedDidParseError::EmptyInput)
        );
    }
}
