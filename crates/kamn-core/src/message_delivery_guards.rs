use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryGuardInput {
    pub message_id: String,
    pub sender: String,
    pub recipient: String,
    pub nonce: u64,
    pub created: String,
    pub expires: String,
    pub received_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeliveryFailureCode {
    NonceOutOfSequence { expected: u64, found: u64 },
    Replay,
    Expired,
    InvalidWindow,
}

impl DeliveryFailureCode {
    fn slug(&self) -> &'static str {
        match self {
            Self::NonceOutOfSequence { .. } => "nonce_out_of_sequence",
            Self::Replay => "replay",
            Self::Expired => "expired",
            Self::InvalidWindow => "invalid_window",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedDeliveryNotice {
    pub message_id: String,
    pub sender: String,
    pub recipient: String,
    pub code: DeliveryFailureCode,
    pub detail: String,
    pub timestamp: String,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeliveryValidationResult {
    Accepted,
    Rejected(FailedDeliveryNotice),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MessageDeliveryGuards {
    next_nonce_by_sender: BTreeMap<String, u64>,
    seen_message_ids: BTreeSet<String>,
}

impl MessageDeliveryGuards {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn expected_nonce(&self, sender: &str) -> u64 {
        self.next_nonce_by_sender.get(sender).copied().unwrap_or(1)
    }

    pub fn validate(&mut self, input: DeliveryGuardInput) -> DeliveryValidationResult {
        if input.expires <= input.created {
            return DeliveryValidationResult::Rejected(self.failed_notice(
                &input,
                DeliveryFailureCode::InvalidWindow,
                "expires must be strictly greater than created".to_owned(),
            ));
        }
        if input.received_at > input.expires {
            return DeliveryValidationResult::Rejected(self.failed_notice(
                &input,
                DeliveryFailureCode::Expired,
                "message expired before delivery".to_owned(),
            ));
        }
        if self.seen_message_ids.contains(&input.message_id) {
            return DeliveryValidationResult::Rejected(self.failed_notice(
                &input,
                DeliveryFailureCode::Replay,
                "message replay detected".to_owned(),
            ));
        }

        let expected = self.expected_nonce(&input.sender);
        if input.nonce != expected {
            return DeliveryValidationResult::Rejected(self.failed_notice(
                &input,
                DeliveryFailureCode::NonceOutOfSequence {
                    expected,
                    found: input.nonce,
                },
                format!(
                    "nonce out of sequence for sender {}, expected {}, found {}",
                    input.sender, expected, input.nonce
                ),
            ));
        }

        self.seen_message_ids.insert(input.message_id.clone());
        self.next_nonce_by_sender
            .insert(input.sender.clone(), input.nonce + 1);
        DeliveryValidationResult::Accepted
    }

    fn failed_notice(
        &self,
        input: &DeliveryGuardInput,
        code: DeliveryFailureCode,
        detail: String,
    ) -> FailedDeliveryNotice {
        let code_slug = code.slug();
        FailedDeliveryNotice {
            message_id: input.message_id.clone(),
            sender: input.sender.clone(),
            recipient: input.recipient.clone(),
            code,
            detail,
            timestamp: input.received_at.clone(),
            signature: format!(
                "notice:{}:{}:{}:{}:{}",
                input.message_id, code_slug, input.recipient, input.received_at, input.nonce
            ),
        }
    }
}

impl fmt::Display for DeliveryFailureCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonceOutOfSequence { expected, found } => {
                write!(
                    f,
                    "nonce out of sequence (expected {expected}, found {found})"
                )
            }
            Self::Replay => write!(f, "message replay detected"),
            Self::Expired => write!(f, "message expired"),
            Self::InvalidWindow => write!(f, "invalid created/expires window"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DeliveryFailureCode, DeliveryGuardInput, DeliveryValidationResult, MessageDeliveryGuards,
    };

    fn input(message_id: &str, nonce: u64, received_at: &str) -> DeliveryGuardInput {
        DeliveryGuardInput {
            message_id: message_id.to_owned(),
            sender: "kamn:did:agent:sender-1".to_owned(),
            recipient: "kamn:did:agent:recipient-1".to_owned(),
            nonce,
            created: "2026-02-07T20:15:30.123Z".to_owned(),
            expires: "2026-02-07T20:45:30.123Z".to_owned(),
            received_at: received_at.to_owned(),
        }
    }

    #[test]
    fn invalid_window_is_rejected() {
        let mut guards = MessageDeliveryGuards::new();
        let mut candidate = input("urn:uuid:msg-1", 1, "2026-02-07T20:20:30.123Z");
        candidate.expires = candidate.created.clone();

        match guards.validate(candidate) {
            DeliveryValidationResult::Rejected(notice) => {
                assert_eq!(notice.code, DeliveryFailureCode::InvalidWindow);
            }
            DeliveryValidationResult::Accepted => panic!("expected invalid window rejection"),
        }
    }

    #[test]
    fn replay_rejected_after_accept() {
        let mut guards = MessageDeliveryGuards::new();
        assert_eq!(
            guards.validate(input("urn:uuid:msg-2", 1, "2026-02-07T20:20:30.123Z")),
            DeliveryValidationResult::Accepted
        );

        match guards.validate(input("urn:uuid:msg-2", 2, "2026-02-07T20:21:30.123Z")) {
            DeliveryValidationResult::Rejected(notice) => {
                assert_eq!(notice.code, DeliveryFailureCode::Replay);
            }
            DeliveryValidationResult::Accepted => panic!("expected replay rejection"),
        }
    }
}
