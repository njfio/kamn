use crate::{AgentDid, DidDocument};
use std::collections::HashMap;

use super::models::{
    DidLifecycleMutationEvidence, DidMutationSubmissionKey, DidRegistrationChainAdapter,
    DidRegistryRecord, DidSubmissionFinalityRecord, DidSubmissionFinalityStatus,
    DidSubmissionRetryClass,
};
use super::validation::DidRegistryError;

mod finality;
mod registration;
pub(crate) mod support;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// DID registry state machine and submission/finality index.
pub struct DidRegistry {
    pub(crate) records: HashMap<AgentDid, DidRegistryRecord>,
    pub(crate) submission_keys_by_did: HashMap<AgentDid, String>,
    pub(crate) finality_by_did: HashMap<AgentDid, DidSubmissionFinalityRecord>,
    pub(crate) lifecycle_submission_keys_by_did_nonce: HashMap<DidMutationSubmissionKey, String>,
    pub(crate) lifecycle_finality_by_did_nonce:
        HashMap<DidMutationSubmissionKey, DidSubmissionFinalityRecord>,
    pub(crate) lifecycle_evidence_by_did_nonce:
        HashMap<DidMutationSubmissionKey, DidLifecycleMutationEvidence>,
    pub(crate) last_mutation_nonce_by_did: HashMap<AgentDid, u64>,
}

impl DidRegistry {
    /// Creates an empty DID registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new DID document in active state.
    pub fn register(
        &mut self,
        did: AgentDid,
        document: DidDocument,
    ) -> Result<(), DidRegistryError> {
        support::validate_document_did(&did, &document)?;
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

    /// Resolves active DID document by identifier.
    pub fn resolve(&self, did: &AgentDid) -> Result<&DidDocument, DidRegistryError> {
        match self.records.get(did) {
            Some(record) if !record.revoked => Ok(&record.document),
            Some(_) => Err(DidRegistryError::Revoked(did.as_str().to_owned())),
            None => Err(DidRegistryError::NotFound(did.as_str().to_owned())),
        }
    }

    /// Replaces document for an active DID.
    pub fn update(&mut self, did: AgentDid, document: DidDocument) -> Result<(), DidRegistryError> {
        support::validate_document_did(&did, &document)?;
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

    /// Revokes an active DID.
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
}
