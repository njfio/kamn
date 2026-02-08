use crate::AgentDid;
use std::collections::{HashMap, HashSet};

pub const DEFAULT_MAX_CLAIM_VALIDITY_WINDOW_SECS: u64 = 24 * 60 * 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionRecord {
    pub id: String,
    pub from_did: String,
    pub payload_hash: String,
    pub signature: String,
    pub inclusion_proof_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionClaim {
    pub instruction_id: String,
    pub from_did: String,
    pub payload_hash: String,
    pub signature: String,
    pub inclusion_proof_ref: String,
    pub expires_at_unix: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationContext {
    pub now_unix: u64,
    pub instructions: HashMap<String, InstructionRecord>,
    pub authorized_senders: HashSet<String>,
    pub max_claim_validity_window_secs: u64,
    pub consumed_instruction_ids: HashSet<String>,
}

impl VerificationContext {
    pub fn new(now_unix: u64) -> Self {
        Self {
            now_unix,
            instructions: HashMap::new(),
            authorized_senders: HashSet::new(),
            max_claim_validity_window_secs: DEFAULT_MAX_CLAIM_VALIDITY_WINDOW_SECS,
            consumed_instruction_ids: HashSet::new(),
        }
    }

    pub fn with_instruction(mut self, record: InstructionRecord) -> Self {
        self.instructions.insert(record.id.clone(), record);
        self
    }

    pub fn with_authorized_sender(mut self, did: &str) -> Self {
        self.authorized_senders.insert(did.to_owned());
        self
    }

    pub fn with_max_claim_validity_window_secs(mut self, max_window_secs: u64) -> Self {
        self.max_claim_validity_window_secs = max_window_secs;
        self
    }

    pub fn with_consumed_instruction_id(mut self, instruction_id: &str) -> Self {
        self.consumed_instruction_ids
            .insert(instruction_id.to_owned());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationFailure {
    MissingInstruction(String),
    MissingInclusionProofReference,
    InvalidClaimSenderDid(String),
    InvalidRecordSenderDid(String),
    SenderMismatch {
        expected: String,
        actual: String,
    },
    PayloadMismatch,
    MissingClaimSignature,
    MissingRecordSignature,
    SignatureMismatch,
    InclusionProofMismatch {
        expected: String,
        actual: String,
    },
    UnauthorizedSender(String),
    Expired {
        expires_at: u64,
        now: u64,
    },
    OverlongValidityWindow {
        max_window_secs: u64,
        requested_window_secs: u64,
    },
    ReplayClaim {
        instruction_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationOutcome {
    Valid,
    Rejected(VerificationFailure),
}

pub struct InstructionVerifier;

impl InstructionVerifier {
    pub fn verify(claim: &InstructionClaim, context: &VerificationContext) -> VerificationOutcome {
        let record = match context.instructions.get(&claim.instruction_id) {
            Some(value) => value,
            None => {
                return VerificationOutcome::Rejected(VerificationFailure::MissingInstruction(
                    claim.instruction_id.clone(),
                ));
            }
        };
        if AgentDid::parse(&claim.from_did).is_err() {
            return VerificationOutcome::Rejected(VerificationFailure::InvalidClaimSenderDid(
                claim.from_did.clone(),
            ));
        }
        if AgentDid::parse(&record.from_did).is_err() {
            return VerificationOutcome::Rejected(VerificationFailure::InvalidRecordSenderDid(
                record.from_did.clone(),
            ));
        }

        if record.from_did != claim.from_did {
            return VerificationOutcome::Rejected(VerificationFailure::SenderMismatch {
                expected: record.from_did.clone(),
                actual: claim.from_did.clone(),
            });
        }
        if record.payload_hash != claim.payload_hash {
            return VerificationOutcome::Rejected(VerificationFailure::PayloadMismatch);
        }
        if claim.signature.trim().is_empty() {
            return VerificationOutcome::Rejected(VerificationFailure::MissingClaimSignature);
        }
        if record.signature.trim().is_empty() {
            return VerificationOutcome::Rejected(VerificationFailure::MissingRecordSignature);
        }
        if record.signature != claim.signature {
            return VerificationOutcome::Rejected(VerificationFailure::SignatureMismatch);
        }
        if record.inclusion_proof_ref.trim().is_empty()
            || claim.inclusion_proof_ref.trim().is_empty()
        {
            return VerificationOutcome::Rejected(
                VerificationFailure::MissingInclusionProofReference,
            );
        }
        if record.inclusion_proof_ref != claim.inclusion_proof_ref {
            return VerificationOutcome::Rejected(VerificationFailure::InclusionProofMismatch {
                expected: record.inclusion_proof_ref.clone(),
                actual: claim.inclusion_proof_ref.clone(),
            });
        }
        if !context.authorized_senders.contains(&claim.from_did) {
            return VerificationOutcome::Rejected(VerificationFailure::UnauthorizedSender(
                claim.from_did.clone(),
            ));
        }
        if claim.expires_at_unix <= context.now_unix {
            return VerificationOutcome::Rejected(VerificationFailure::Expired {
                expires_at: claim.expires_at_unix,
                now: context.now_unix,
            });
        }
        let requested_window_secs = claim.expires_at_unix.saturating_sub(context.now_unix);
        if requested_window_secs > context.max_claim_validity_window_secs {
            return VerificationOutcome::Rejected(VerificationFailure::OverlongValidityWindow {
                max_window_secs: context.max_claim_validity_window_secs,
                requested_window_secs,
            });
        }
        VerificationOutcome::Valid
    }

    pub fn verify_and_record(
        claim: &InstructionClaim,
        context: &mut VerificationContext,
    ) -> VerificationOutcome {
        if context
            .consumed_instruction_ids
            .contains(&claim.instruction_id)
        {
            return VerificationOutcome::Rejected(VerificationFailure::ReplayClaim {
                instruction_id: claim.instruction_id.clone(),
            });
        }

        let outcome = Self::verify(claim, context);
        if outcome == VerificationOutcome::Valid {
            context
                .consumed_instruction_ids
                .insert(claim.instruction_id.clone());
        }
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::{
        InstructionClaim, InstructionRecord, InstructionVerifier, VerificationContext,
        VerificationFailure, VerificationOutcome, DEFAULT_MAX_CLAIM_VALIDITY_WINDOW_SECS,
    };

    #[test]
    fn rejects_payload_hash_mismatch() {
        let context = VerificationContext::new(1)
            .with_instruction(InstructionRecord {
                id: "ins_1".to_owned(),
                from_did: "kamn:did:agent:alpha".to_owned(),
                payload_hash: "hash_a".to_owned(),
                signature: "sig".to_owned(),
                inclusion_proof_ref: "proof:chain:tx-1".to_owned(),
            })
            .with_authorized_sender("kamn:did:agent:alpha");
        let claim = InstructionClaim {
            instruction_id: "ins_1".to_owned(),
            from_did: "kamn:did:agent:alpha".to_owned(),
            payload_hash: "hash_b".to_owned(),
            signature: "sig".to_owned(),
            inclusion_proof_ref: "proof:chain:tx-1".to_owned(),
            expires_at_unix: 2,
        };

        assert_eq!(
            InstructionVerifier::verify(&claim, &context),
            VerificationOutcome::Rejected(VerificationFailure::PayloadMismatch)
        );
    }

    #[test]
    fn verification_context_uses_bounded_default_claim_window_policy() {
        let context = VerificationContext::new(10);
        assert_eq!(
            context.max_claim_validity_window_secs,
            DEFAULT_MAX_CLAIM_VALIDITY_WINDOW_SECS
        );
    }

    #[test]
    fn rejects_overlong_claim_validity_window() {
        let context = VerificationContext::new(100)
            .with_instruction(InstructionRecord {
                id: "ins_1".to_owned(),
                from_did: "kamn:did:agent:alpha".to_owned(),
                payload_hash: "hash_a".to_owned(),
                signature: "sig".to_owned(),
                inclusion_proof_ref: "proof:chain:tx-1".to_owned(),
            })
            .with_authorized_sender("kamn:did:agent:alpha")
            .with_max_claim_validity_window_secs(60);
        let claim = InstructionClaim {
            instruction_id: "ins_1".to_owned(),
            from_did: "kamn:did:agent:alpha".to_owned(),
            payload_hash: "hash_a".to_owned(),
            signature: "sig".to_owned(),
            inclusion_proof_ref: "proof:chain:tx-1".to_owned(),
            expires_at_unix: 200,
        };

        assert_eq!(
            InstructionVerifier::verify(&claim, &context),
            VerificationOutcome::Rejected(VerificationFailure::OverlongValidityWindow {
                max_window_secs: 60,
                requested_window_secs: 100,
            })
        );
    }

    #[test]
    fn verify_and_record_rejects_replayed_claim() {
        let mut context = VerificationContext::new(100)
            .with_instruction(InstructionRecord {
                id: "ins_1".to_owned(),
                from_did: "kamn:did:agent:alpha".to_owned(),
                payload_hash: "hash_a".to_owned(),
                signature: "sig".to_owned(),
                inclusion_proof_ref: "proof:chain:tx-1".to_owned(),
            })
            .with_authorized_sender("kamn:did:agent:alpha");
        let claim = InstructionClaim {
            instruction_id: "ins_1".to_owned(),
            from_did: "kamn:did:agent:alpha".to_owned(),
            payload_hash: "hash_a".to_owned(),
            signature: "sig".to_owned(),
            inclusion_proof_ref: "proof:chain:tx-1".to_owned(),
            expires_at_unix: 120,
        };

        assert_eq!(
            InstructionVerifier::verify_and_record(&claim, &mut context),
            VerificationOutcome::Valid
        );
        assert_eq!(
            InstructionVerifier::verify_and_record(&claim, &mut context),
            VerificationOutcome::Rejected(VerificationFailure::ReplayClaim {
                instruction_id: "ins_1".to_owned(),
            })
        );
    }

    #[test]
    fn rejects_missing_inclusion_proof_reference() {
        let context = VerificationContext::new(100)
            .with_instruction(InstructionRecord {
                id: "ins_1".to_owned(),
                from_did: "kamn:did:agent:alpha".to_owned(),
                payload_hash: "hash_a".to_owned(),
                signature: "sig".to_owned(),
                inclusion_proof_ref: "proof:chain:tx-1".to_owned(),
            })
            .with_authorized_sender("kamn:did:agent:alpha");
        let claim = InstructionClaim {
            instruction_id: "ins_1".to_owned(),
            from_did: "kamn:did:agent:alpha".to_owned(),
            payload_hash: "hash_a".to_owned(),
            signature: "sig".to_owned(),
            inclusion_proof_ref: String::new(),
            expires_at_unix: 120,
        };

        assert_eq!(
            InstructionVerifier::verify(&claim, &context),
            VerificationOutcome::Rejected(VerificationFailure::MissingInclusionProofReference)
        );
    }

    #[test]
    fn rejects_missing_claim_signature() {
        let context = VerificationContext::new(100)
            .with_instruction(InstructionRecord {
                id: "ins_1".to_owned(),
                from_did: "kamn:did:agent:alpha".to_owned(),
                payload_hash: "hash_a".to_owned(),
                signature: "sig".to_owned(),
                inclusion_proof_ref: "proof:chain:tx-1".to_owned(),
            })
            .with_authorized_sender("kamn:did:agent:alpha");
        let claim = InstructionClaim {
            instruction_id: "ins_1".to_owned(),
            from_did: "kamn:did:agent:alpha".to_owned(),
            payload_hash: "hash_a".to_owned(),
            signature: String::new(),
            inclusion_proof_ref: "proof:chain:tx-1".to_owned(),
            expires_at_unix: 120,
        };

        assert_eq!(
            InstructionVerifier::verify(&claim, &context),
            VerificationOutcome::Rejected(VerificationFailure::MissingClaimSignature)
        );
    }

    #[test]
    fn rejects_missing_record_signature() {
        let context = VerificationContext::new(100)
            .with_instruction(InstructionRecord {
                id: "ins_1".to_owned(),
                from_did: "kamn:did:agent:alpha".to_owned(),
                payload_hash: "hash_a".to_owned(),
                signature: String::new(),
                inclusion_proof_ref: "proof:chain:tx-1".to_owned(),
            })
            .with_authorized_sender("kamn:did:agent:alpha");
        let claim = InstructionClaim {
            instruction_id: "ins_1".to_owned(),
            from_did: "kamn:did:agent:alpha".to_owned(),
            payload_hash: "hash_a".to_owned(),
            signature: "sig".to_owned(),
            inclusion_proof_ref: "proof:chain:tx-1".to_owned(),
            expires_at_unix: 120,
        };

        assert_eq!(
            InstructionVerifier::verify(&claim, &context),
            VerificationOutcome::Rejected(VerificationFailure::MissingRecordSignature)
        );
    }

    #[test]
    fn rejects_inclusion_proof_reference_mismatch() {
        let context = VerificationContext::new(100)
            .with_instruction(InstructionRecord {
                id: "ins_1".to_owned(),
                from_did: "kamn:did:agent:alpha".to_owned(),
                payload_hash: "hash_a".to_owned(),
                signature: "sig".to_owned(),
                inclusion_proof_ref: "proof:chain:tx-1".to_owned(),
            })
            .with_authorized_sender("kamn:did:agent:alpha");
        let claim = InstructionClaim {
            instruction_id: "ins_1".to_owned(),
            from_did: "kamn:did:agent:alpha".to_owned(),
            payload_hash: "hash_a".to_owned(),
            signature: "sig".to_owned(),
            inclusion_proof_ref: "proof:chain:tx-2".to_owned(),
            expires_at_unix: 120,
        };

        assert_eq!(
            InstructionVerifier::verify(&claim, &context),
            VerificationOutcome::Rejected(VerificationFailure::InclusionProofMismatch {
                expected: "proof:chain:tx-1".to_owned(),
                actual: "proof:chain:tx-2".to_owned(),
            })
        );
    }

    #[test]
    fn rejects_invalid_claim_sender_did() {
        let context = VerificationContext::new(100)
            .with_instruction(InstructionRecord {
                id: "ins_1".to_owned(),
                from_did: "kamn:did:agent:alpha".to_owned(),
                payload_hash: "hash_a".to_owned(),
                signature: "sig".to_owned(),
                inclusion_proof_ref: "proof:chain:tx-1".to_owned(),
            })
            .with_authorized_sender("not-a-did");
        let claim = InstructionClaim {
            instruction_id: "ins_1".to_owned(),
            from_did: "not-a-did".to_owned(),
            payload_hash: "hash_a".to_owned(),
            signature: "sig".to_owned(),
            inclusion_proof_ref: "proof:chain:tx-1".to_owned(),
            expires_at_unix: 120,
        };

        assert_eq!(
            InstructionVerifier::verify(&claim, &context),
            VerificationOutcome::Rejected(VerificationFailure::InvalidClaimSenderDid(
                "not-a-did".to_owned()
            ))
        );
    }

    #[test]
    fn rejects_invalid_record_sender_did() {
        let context = VerificationContext::new(100)
            .with_instruction(InstructionRecord {
                id: "ins_1".to_owned(),
                from_did: "bad-record-did".to_owned(),
                payload_hash: "hash_a".to_owned(),
                signature: "sig".to_owned(),
                inclusion_proof_ref: "proof:chain:tx-1".to_owned(),
            })
            .with_authorized_sender("kamn:did:agent:alpha");
        let claim = InstructionClaim {
            instruction_id: "ins_1".to_owned(),
            from_did: "kamn:did:agent:alpha".to_owned(),
            payload_hash: "hash_a".to_owned(),
            signature: "sig".to_owned(),
            inclusion_proof_ref: "proof:chain:tx-1".to_owned(),
            expires_at_unix: 120,
        };

        assert_eq!(
            InstructionVerifier::verify(&claim, &context),
            VerificationOutcome::Rejected(VerificationFailure::InvalidRecordSenderDid(
                "bad-record-did".to_owned()
            ))
        );
    }
}
