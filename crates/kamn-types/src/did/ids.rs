use std::fmt;

use super::errors::{AgentDidError, AgentDidKeyBindingError, KamnDidError};
use super::key_binding::{
    constant_time_eq_bytes, fingerprint_for_public_key_hex, AGENT_DID_KEY_BINDING_HEX_LEN,
    AGENT_DID_KEY_BINDING_MARKER,
};

const KAMN_DID_PREFIX: &str = "kamn:did:";
const AGENT_DID_PREFIX: &str = "kamn:did:agent:";

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
        let actual = fingerprint_for_public_key_hex(public_key_hex)?;
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
        let fingerprint = fingerprint_for_public_key_hex(public_key_hex)?;
        let rendered = format!(
            "{AGENT_DID_PREFIX}{normalized_method_specific_id}{AGENT_DID_KEY_BINDING_MARKER}{fingerprint}"
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
