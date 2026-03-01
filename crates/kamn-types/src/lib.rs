#![warn(missing_docs)]
//! Shared canonical type surface for cross-crate KAMN domain identifiers.

use std::fmt;

pub use kamn_core::AgentDidKeyBindingError;
pub use kamn_core::{
    AgentDid, AgentDidError, AgentDidMetadata, DidDocument, DidService, DidVerificationMethod,
    KamnDid, KamnDidError,
};

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

/// Parses agent DID inputs with canonical trim semantics.
pub fn parse_agent_did_canonical(value: &str) -> Result<AgentDid, SharedDidParseError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(SharedDidParseError::EmptyInput);
    }
    AgentDid::parse(normalized).map_err(SharedDidParseError::Agent)
}

/// Parses generic KAMN DID inputs with canonical trim semantics.
pub fn parse_kamn_did_canonical(value: &str) -> Result<KamnDid, SharedDidParseError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(SharedDidParseError::EmptyInput);
    }
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
