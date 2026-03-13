//! DID identity parsing, canonical document construction, and federated trust-handshake contracts.

pub use kamn_types::did::{
    AgentDid, AgentDidError, AgentDidKeyBindingError, AgentDidMetadata, DidDocument, DidService,
    DidVerificationMethod, KamnDid, KamnDidError,
};

mod document;
mod federated;

#[cfg(test)]
mod tests;

pub use document::*;
pub use federated::*;
