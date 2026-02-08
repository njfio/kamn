use std::collections::{HashMap, HashSet};

pub const DEFAULT_MAX_CLAIM_VALIDITY_WINDOW_SECS: u64 = 24 * 60 * 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionRecord {
    pub id: String,
    pub from_did: String,
    pub payload_hash: String,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionClaim {
    pub instruction_id: String,
    pub from_did: String,
    pub payload_hash: String,
    pub signature: String,
    pub expires_at_unix: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationContext {
    pub now_unix: u64,
    pub instructions: HashMap<String, InstructionRecord>,
    pub authorized_senders: HashSet<String>,
    pub max_claim_validity_window_secs: u64,
}

impl VerificationContext {
    pub fn new(now_unix: u64) -> Self {
        Self {
            now_unix,
            instructions: HashMap::new(),
            authorized_senders: HashSet::new(),
            max_claim_validity_window_secs: DEFAULT_MAX_CLAIM_VALIDITY_WINDOW_SECS,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationFailure {
    MissingInstruction(String),
    SenderMismatch {
        expected: String,
        actual: String,
    },
    PayloadMismatch,
    SignatureMismatch,
    UnauthorizedSender(String),
    Expired {
        expires_at: u64,
        now: u64,
    },
    OverlongValidityWindow {
        max_window_secs: u64,
        requested_window_secs: u64,
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

        if record.from_did != claim.from_did {
            return VerificationOutcome::Rejected(VerificationFailure::SenderMismatch {
                expected: record.from_did.clone(),
                actual: claim.from_did.clone(),
            });
        }
        if record.payload_hash != claim.payload_hash {
            return VerificationOutcome::Rejected(VerificationFailure::PayloadMismatch);
        }
        if record.signature != claim.signature {
            return VerificationOutcome::Rejected(VerificationFailure::SignatureMismatch);
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
            })
            .with_authorized_sender("kamn:did:agent:alpha");
        let claim = InstructionClaim {
            instruction_id: "ins_1".to_owned(),
            from_did: "kamn:did:agent:alpha".to_owned(),
            payload_hash: "hash_b".to_owned(),
            signature: "sig".to_owned(),
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
            })
            .with_authorized_sender("kamn:did:agent:alpha")
            .with_max_claim_validity_window_secs(60);
        let claim = InstructionClaim {
            instruction_id: "ins_1".to_owned(),
            from_did: "kamn:did:agent:alpha".to_owned(),
            payload_hash: "hash_a".to_owned(),
            signature: "sig".to_owned(),
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
}
