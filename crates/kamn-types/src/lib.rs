#![warn(missing_docs)]
//! Shared canonical type surface for cross-crate KAMN domain identifiers.

/// DID-focused shared type and helper boundary for cross-crate consumers.
pub mod did;

pub use did::{
    parse_agent_did_canonical, parse_kamn_did_canonical, AgentDid, AgentDidError,
    AgentDidKeyBindingError, AgentDidMetadata, DidDocument, DidService, DidVerificationMethod,
    KamnDid, KamnDidError, SharedDidParseError,
};
