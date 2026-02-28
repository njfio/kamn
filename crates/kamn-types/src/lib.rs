#![warn(missing_docs)]
//! Shared canonical type surface for cross-crate KAMN domain identifiers.

pub use kamn_core::AgentDidKeyBindingError;
pub use kamn_core::{
    AgentDid, AgentDidError, AgentDidMetadata, DidDocument, DidService, DidVerificationMethod,
    KamnDid, KamnDidError,
};

#[cfg(test)]
mod tests {
    use super::{AgentDid, AgentDidError, KamnDid};

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
}
