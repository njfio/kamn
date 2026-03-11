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
        ensure_register_submission_key(self, did, idempotency_key)?;
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
            finality_record(idempotency_key, sequence, status, receipt),
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
        ensure_lifecycle_submission_key(self, &mutation_key, did, idempotency_key)?;
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
            finality_record(idempotency_key, sequence, status, receipt),
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

fn ensure_register_submission_key(
    registry: &DidRegistry,
    did: &AgentDid,
    idempotency_key: &str,
) -> Result<(), DidRegistryError> {
    match registry.submission_keys_by_did.get(did) {
        Some(expected_key) if expected_key == idempotency_key => Ok(()),
        _ => unknown_submission_key(did, idempotency_key),
    }
}

fn ensure_lifecycle_submission_key(
    registry: &DidRegistry,
    mutation_key: &DidMutationSubmissionKey,
    did: &AgentDid,
    idempotency_key: &str,
) -> Result<(), DidRegistryError> {
    match registry
        .lifecycle_submission_keys_by_did_nonce
        .get(mutation_key)
    {
        Some(expected_key) if expected_key == idempotency_key => Ok(()),
        _ => unknown_submission_key(did, idempotency_key),
    }
}

fn unknown_submission_key<T>(did: &AgentDid, idempotency_key: &str) -> Result<T, DidRegistryError> {
    Err(DidRegistryError::UnknownSubmissionIdempotencyKey {
        did: did.as_str().to_owned(),
        idempotency_key: idempotency_key.to_owned(),
    })
}

fn finality_record(
    idempotency_key: &str,
    sequence: u64,
    status: DidSubmissionFinalityStatus,
    receipt: &str,
) -> DidSubmissionFinalityRecord {
    DidSubmissionFinalityRecord {
        idempotency_key: idempotency_key.to_owned(),
        sequence,
        status,
        receipt: receipt.to_owned(),
    }
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
        validate_finality_sequence(current, did, sequence)?;
        if sequence == current.sequence {
            return validate_same_sequence_update(
                current,
                did,
                idempotency_key,
                sequence,
                status,
                receipt,
            );
        }
        validate_idempotency_key_match(current, did, idempotency_key, sequence)?;
    }
    Ok(())
}

fn validate_finality_sequence(
    current: &DidSubmissionFinalityRecord,
    did: &AgentDid,
    sequence: u64,
) -> Result<(), DidRegistryError> {
    if sequence < current.sequence {
        return Err(DidRegistryError::StaleFinalityUpdate {
            did: did.as_str().to_owned(),
            current_sequence: current.sequence,
            attempted_sequence: sequence,
        });
    }
    Ok(())
}

fn validate_same_sequence_update(
    current: &DidSubmissionFinalityRecord,
    did: &AgentDid,
    idempotency_key: &str,
    sequence: u64,
    status: DidSubmissionFinalityStatus,
    receipt: &str,
) -> Result<(), DidRegistryError> {
    if current.idempotency_key == idempotency_key
        && current.status == status
        && current.receipt == receipt
    {
        return Ok(());
    }
    conflicting_finality_update(did, sequence)
}

fn validate_idempotency_key_match(
    current: &DidSubmissionFinalityRecord,
    did: &AgentDid,
    idempotency_key: &str,
    sequence: u64,
) -> Result<(), DidRegistryError> {
    if current.idempotency_key != idempotency_key {
        return conflicting_finality_update(did, sequence);
    }
    Ok(())
}

fn conflicting_finality_update(did: &AgentDid, sequence: u64) -> Result<(), DidRegistryError> {
    Err(DidRegistryError::ConflictingFinalityUpdate {
        did: did.as_str().to_owned(),
        sequence,
    })
}
