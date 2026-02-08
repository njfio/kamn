use crate::{AgentDid, DidDocument};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
struct DidRegistryRecord {
    document: DidDocument,
    revoked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DidRegistry {
    records: HashMap<AgentDid, DidRegistryRecord>,
}

impl DidRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        did: AgentDid,
        document: DidDocument,
    ) -> Result<(), DidRegistryError> {
        Self::validate_document_did(&did, &document)?;
        let did_id = did.as_str().to_owned();
        match self.records.get(&did) {
            Some(record) if !record.revoked => Err(DidRegistryError::AlreadyRegistered(did_id)),
            Some(_) => Err(DidRegistryError::Revoked(did_id)),
            None => {
                self.records.insert(
                    did,
                    DidRegistryRecord {
                        document,
                        revoked: false,
                    },
                );
                Ok(())
            }
        }
    }

    pub fn resolve(&self, did: &AgentDid) -> Result<&DidDocument, DidRegistryError> {
        match self.records.get(did) {
            Some(record) if !record.revoked => Ok(&record.document),
            Some(_) => Err(DidRegistryError::Revoked(did.as_str().to_owned())),
            None => Err(DidRegistryError::NotFound(did.as_str().to_owned())),
        }
    }

    pub fn update(&mut self, did: AgentDid, document: DidDocument) -> Result<(), DidRegistryError> {
        Self::validate_document_did(&did, &document)?;
        match self.records.get_mut(&did) {
            Some(record) if record.revoked => {
                Err(DidRegistryError::Revoked(did.as_str().to_owned()))
            }
            Some(record) => {
                record.document = document;
                Ok(())
            }
            None => Err(DidRegistryError::NotFound(did.as_str().to_owned())),
        }
    }

    pub fn revoke(&mut self, did: &AgentDid) -> Result<(), DidRegistryError> {
        match self.records.get_mut(did) {
            Some(record) if record.revoked => {
                Err(DidRegistryError::Revoked(did.as_str().to_owned()))
            }
            Some(record) => {
                record.revoked = true;
                Ok(())
            }
            None => Err(DidRegistryError::NotFound(did.as_str().to_owned())),
        }
    }

    fn validate_document_did(
        did: &AgentDid,
        document: &DidDocument,
    ) -> Result<(), DidRegistryError> {
        if document.id != did.as_str() {
            return Err(DidRegistryError::DocumentDidMismatch {
                expected: did.as_str().to_owned(),
                actual: document.id.clone(),
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DidRegistryError {
    AlreadyRegistered(String),
    NotFound(String),
    Revoked(String),
    DocumentDidMismatch { expected: String, actual: String },
}

impl fmt::Display for DidRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyRegistered(value) => write!(f, "did is already registered: {value}"),
            Self::NotFound(value) => write!(f, "did not found: {value}"),
            Self::Revoked(value) => write!(f, "did is revoked: {value}"),
            Self::DocumentDidMismatch { expected, actual } => {
                write!(
                    f,
                    "did document id mismatch, expected {expected}, got {actual}"
                )
            }
        }
    }
}

impl std::error::Error for DidRegistryError {}

#[cfg(test)]
mod tests {
    use super::{DidRegistry, DidRegistryError};
    use crate::{canonical_did_document, AgentDid, AgentDidMetadata};

    fn metadata() -> AgentDidMetadata {
        AgentDidMetadata {
            agent_type: "autonomous".to_owned(),
            model_family: "claude-4".to_owned(),
            capabilities: vec!["text".to_owned()],
            operator: None,
        }
    }

    fn document_for(did: &AgentDid) -> crate::DidDocument {
        canonical_did_document(did, "z6Mpubkey", metadata()).expect("document should build")
    }

    #[test]
    fn rejects_document_mismatch() {
        let mut registry = DidRegistry::new();
        let did = AgentDid::parse("kamn:did:agent:agent-1").expect("did should parse");
        let other = AgentDid::parse("kamn:did:agent:agent-2").expect("did should parse");

        assert_eq!(
            registry.register(did.clone(), document_for(&other)),
            Err(DidRegistryError::DocumentDidMismatch {
                expected: did.as_str().to_owned(),
                actual: other.as_str().to_owned(),
            })
        );
    }

    #[test]
    fn update_rejects_revoked_did() {
        let mut registry = DidRegistry::new();
        let did = AgentDid::parse("kamn:did:agent:agent-3").expect("did should parse");
        registry
            .register(did.clone(), document_for(&did))
            .expect("register should succeed");
        registry.revoke(&did).expect("revoke should succeed");

        assert_eq!(
            registry.update(did.clone(), document_for(&did)),
            Err(DidRegistryError::Revoked(did.as_str().to_owned()))
        );
    }
}
