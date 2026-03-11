use super::*;

impl DidRegistry {
    /// Records finality update for prior register submission.
    pub fn record_register_finality(
        &mut self,
        did: &AgentDid,
        idempotency_key: &str,
        sequence: u64,
        status: DidSubmissionFinalityStatus,
        receipt: &str,
    ) -> Result<(), DidRegistryError> {
        let Some(expected_key) = self.submission_keys_by_did.get(did) else {
            return unknown_submission_key(did, idempotency_key);
        };
        if expected_key != idempotency_key {
            return unknown_submission_key(did, idempotency_key);
        }
        validate_finality_update(
            self.finality_by_did.get(did),
            did,
            idempotency_key,
            sequence,
            status,
            receipt,
        )?;
        self.finality_by_did.insert(
            did.clone(),
            DidSubmissionFinalityRecord {
                idempotency_key: idempotency_key.to_owned(),
                sequence,
                status,
                receipt: receipt.to_owned(),
            },
        );
        Ok(())
    }

    /// Returns most recent finality record for DID, if present.
    pub fn register_finality(&self, did: &AgentDid) -> Option<&DidSubmissionFinalityRecord> {
        self.finality_by_did.get(did)
    }

    /// Records finality update for prior lifecycle mutation submission.
    pub fn record_lifecycle_finality(
        &mut self,
        did: &AgentDid,
        nonce: u64,
        idempotency_key: &str,
        sequence: u64,
        status: DidSubmissionFinalityStatus,
        receipt: &str,
    ) -> Result<(), DidRegistryError> {
        let mutation_key = (did.clone(), nonce);
        let Some(expected_key) = self
            .lifecycle_submission_keys_by_did_nonce
            .get(&mutation_key)
        else {
            return unknown_submission_key(did, idempotency_key);
        };
        if expected_key != idempotency_key {
            return unknown_submission_key(did, idempotency_key);
        }
        validate_finality_update(
            self.lifecycle_finality_by_did_nonce.get(&mutation_key),
            did,
            idempotency_key,
            sequence,
            status,
            receipt,
        )?;
        self.lifecycle_finality_by_did_nonce.insert(
            mutation_key,
            DidSubmissionFinalityRecord {
                idempotency_key: idempotency_key.to_owned(),
                sequence,
                status,
                receipt: receipt.to_owned(),
            },
        );
        Ok(())
    }

    /// Returns most recent lifecycle finality record for DID nonce, if present.
    pub fn lifecycle_finality(
        &self,
        did: &AgentDid,
        nonce: u64,
    ) -> Option<&DidSubmissionFinalityRecord> {
        self.lifecycle_finality_by_did_nonce
            .get(&(did.clone(), nonce))
    }
}

fn unknown_submission_key<T>(did: &AgentDid, idempotency_key: &str) -> Result<T, DidRegistryError> {
    Err(DidRegistryError::UnknownSubmissionIdempotencyKey {
        did: did.as_str().to_owned(),
        idempotency_key: idempotency_key.to_owned(),
    })
}

fn validate_finality_update(
    current: Option<&DidSubmissionFinalityRecord>,
    did: &AgentDid,
    idempotency_key: &str,
    sequence: u64,
    status: DidSubmissionFinalityStatus,
    receipt: &str,
) -> Result<(), DidRegistryError> {
    if let Some(current) = current {
        if sequence < current.sequence {
            return Err(DidRegistryError::StaleFinalityUpdate {
                did: did.as_str().to_owned(),
                current_sequence: current.sequence,
                attempted_sequence: sequence,
            });
        }
        if sequence == current.sequence {
            if current.idempotency_key == idempotency_key
                && current.status == status
                && current.receipt == receipt
            {
                return Ok(());
            }
            return Err(DidRegistryError::ConflictingFinalityUpdate {
                did: did.as_str().to_owned(),
                sequence,
            });
        }
        if current.idempotency_key != idempotency_key {
            return Err(DidRegistryError::ConflictingFinalityUpdate {
                did: did.as_str().to_owned(),
                sequence,
            });
        }
    }
    Ok(())
}
