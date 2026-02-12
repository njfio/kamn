//! Canonical message envelope schema, validation rules, and payload normalization.

use crate::AgentDid;
use std::collections::BTreeMap;
use std::fmt;

/// Required envelope type identifier for canonical KAMN messages.
pub const CANONICAL_MESSAGE_ENVELOPE_TYPE: &str = "kamn:message:v1";
/// Required encryption algorithm for canonical envelope headers.
pub const CANONICAL_ENCRYPTION_ALGORITHM: &str = "X25519-XChaCha20-Poly1305";
/// Required proof purpose for canonical envelope signatures.
pub const CANONICAL_PROOF_PURPOSE: &str = "authentication";

const ALLOWED_MESSAGE_TYPES: [&str; 11] = [
    "Request",
    "Response",
    "Proposal",
    "Acceptance",
    "Rejection",
    "Delegation",
    "StatusUpdate",
    "PaymentOffer",
    "PaymentConfirm",
    "Heartbeat",
    "Revocation",
];

/// Envelope metadata fields used for routing, replay protection, and threading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvelopeMetadata {
    /// Stable message identifier.
    pub id: String,
    /// Envelope schema type discriminator.
    pub type_name: String,
    /// Sender DID.
    pub from: String,
    /// Recipient DID list.
    pub to: Vec<String>,
    /// Creation timestamp string.
    pub created: String,
    /// Expiration timestamp string.
    pub expires: String,
    /// Optional thread identifier.
    pub thread_id: Option<String>,
    /// Optional parent message identifier.
    pub parent_id: Option<String>,
    /// Monotonic sender nonce.
    pub nonce: u64,
}

/// Envelope encryption descriptor for recipient key distribution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvelopeEncryption {
    /// Encryption algorithm name.
    pub algorithm: String,
    /// Recipient public key references.
    pub recipient_keys: Vec<String>,
}

/// Header metadata describing message semantics and encryption policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvelopeHeader {
    /// Domain-specific message type.
    pub message_type: String,
    /// Delivery priority label.
    pub priority: String,
    /// Payload content type.
    pub content_type: String,
    /// Encryption parameters.
    pub encryption: EnvelopeEncryption,
}

/// Attachment pointer included in the envelope body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentRef {
    /// Attachment identifier.
    pub id: String,
    /// Attachment media type.
    pub media_type: String,
    /// Attachment retrieval URI.
    pub uri: String,
}

/// Signature proof block for envelope authenticity checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvelopeProof {
    /// Proof type name.
    pub type_name: String,
    /// Proof creation timestamp string.
    pub created: String,
    /// Verification method DID URL.
    pub verification_method: String,
    /// Proof purpose value.
    pub proof_purpose: String,
    /// Signature payload value.
    pub proof_value: String,
}

/// Canonical message envelope contract with metadata, payload, and proof sections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalMessageEnvelope {
    /// Envelope metadata section.
    pub envelope: EnvelopeMetadata,
    /// Envelope header section.
    pub header: EnvelopeHeader,
    /// Canonical body key/value payload.
    pub body: BTreeMap<String, String>,
    /// Attachment references for out-of-band content.
    pub attachments: Vec<AttachmentRef>,
    /// Signature proof section.
    pub proof: EnvelopeProof,
}

impl CanonicalMessageEnvelope {
    /// Validates all envelope contract rules.
    pub fn validate(&self) -> Result<(), MessageEnvelopeError> {
        require_non_empty("envelope.id", &self.envelope.id)?;
        if self.envelope.type_name != CANONICAL_MESSAGE_ENVELOPE_TYPE {
            return Err(MessageEnvelopeError::InvalidEnvelopeType(
                self.envelope.type_name.clone(),
            ));
        }

        if let Err(error) = AgentDid::parse(&self.envelope.from) {
            return Err(MessageEnvelopeError::InvalidSenderDid(error.to_string()));
        }

        if self.envelope.to.is_empty() {
            return Err(MessageEnvelopeError::EmptyRecipients);
        }
        for recipient in &self.envelope.to {
            if let Err(error) = AgentDid::parse(recipient) {
                return Err(MessageEnvelopeError::InvalidRecipientDid(error.to_string()));
            }
        }

        require_non_empty("envelope.created", &self.envelope.created)?;
        require_non_empty("envelope.expires", &self.envelope.expires)?;
        if self.envelope.expires <= self.envelope.created {
            return Err(MessageEnvelopeError::InvalidExpiryWindow {
                created: self.envelope.created.clone(),
                expires: self.envelope.expires.clone(),
            });
        }
        if self.envelope.nonce == 0 {
            return Err(MessageEnvelopeError::InvalidNonce(self.envelope.nonce));
        }

        require_non_empty("header.message_type", &self.header.message_type)?;
        if !ALLOWED_MESSAGE_TYPES.contains(&self.header.message_type.as_str()) {
            return Err(MessageEnvelopeError::InvalidMessageType(
                self.header.message_type.clone(),
            ));
        }
        require_non_empty("header.priority", &self.header.priority)?;
        require_non_empty("header.content_type", &self.header.content_type)?;
        if self.header.encryption.algorithm != CANONICAL_ENCRYPTION_ALGORITHM {
            return Err(MessageEnvelopeError::InvalidEncryptionAlgorithm(
                self.header.encryption.algorithm.clone(),
            ));
        }
        if self.header.encryption.recipient_keys.is_empty() {
            return Err(MessageEnvelopeError::EmptyRecipientKeys);
        }
        if self
            .header
            .encryption
            .recipient_keys
            .iter()
            .any(|value| value.trim().is_empty())
        {
            return Err(MessageEnvelopeError::EmptyField(
                "header.encryption.recipient_keys[]",
            ));
        }

        if self.body.is_empty() {
            return Err(MessageEnvelopeError::EmptyBody);
        }
        if self
            .body
            .iter()
            .any(|(key, value)| key.trim().is_empty() || value.trim().is_empty())
        {
            return Err(MessageEnvelopeError::InvalidBodyEntry(
                "body entries must have non-empty key/value".to_owned(),
            ));
        }

        for attachment in &self.attachments {
            if attachment.id.trim().is_empty() {
                return Err(MessageEnvelopeError::InvalidAttachmentField {
                    attachment_id: attachment.id.clone(),
                    field: "id",
                });
            }
            if attachment.media_type.trim().is_empty() {
                return Err(MessageEnvelopeError::InvalidAttachmentField {
                    attachment_id: attachment.id.clone(),
                    field: "media_type",
                });
            }
            if attachment.uri.trim().is_empty() {
                return Err(MessageEnvelopeError::InvalidAttachmentField {
                    attachment_id: attachment.id.clone(),
                    field: "uri",
                });
            }
        }

        require_non_empty("proof.type_name", &self.proof.type_name)?;
        require_non_empty("proof.created", &self.proof.created)?;
        require_non_empty("proof.verification_method", &self.proof.verification_method)?;
        if self.proof.proof_purpose != CANONICAL_PROOF_PURPOSE {
            return Err(MessageEnvelopeError::InvalidProofPurpose(
                self.proof.proof_purpose.clone(),
            ));
        }
        require_non_empty("proof.proof_value", &self.proof.proof_value)?;

        let expected_prefix = format!("{}#", self.envelope.from);
        if !self.proof.verification_method.starts_with(&expected_prefix) {
            return Err(MessageEnvelopeError::ProofVerificationMethodMismatch {
                expected_prefix,
                actual: self.proof.verification_method.clone(),
            });
        }

        Ok(())
    }

    /// Renders a deterministic canonical payload string for signing and verification.
    pub fn canonical_payload(&self) -> String {
        let mut recipients = self.envelope.to.clone();
        recipients.sort();

        let mut recipient_keys = self.header.encryption.recipient_keys.clone();
        recipient_keys.sort();

        let mut attachments = self.attachments.clone();
        attachments.sort_by(|a, b| a.id.cmp(&b.id));

        let mut output = String::new();
        output.push_str("envelope|");
        output.push_str(&self.envelope.id);
        output.push('|');
        output.push_str(&self.envelope.type_name);
        output.push('|');
        output.push_str(&self.envelope.from);
        output.push('|');
        output.push_str(&recipients.join(","));
        output.push('|');
        output.push_str(&self.envelope.created);
        output.push('|');
        output.push_str(&self.envelope.expires);
        output.push('|');
        output.push_str(self.envelope.thread_id.as_deref().unwrap_or(""));
        output.push('|');
        output.push_str(self.envelope.parent_id.as_deref().unwrap_or(""));
        output.push('|');
        output.push_str(&self.envelope.nonce.to_string());

        output.push_str("|header|");
        output.push_str(&self.header.message_type);
        output.push('|');
        output.push_str(&self.header.priority);
        output.push('|');
        output.push_str(&self.header.content_type);
        output.push('|');
        output.push_str(&self.header.encryption.algorithm);
        output.push('|');
        output.push_str(&recipient_keys.join(","));

        output.push_str("|body|");
        for (key, value) in &self.body {
            output.push_str(key);
            output.push('=');
            output.push_str(value);
            output.push(';');
        }

        output.push_str("|attachments|");
        for attachment in attachments {
            output.push_str(&attachment.id);
            output.push(':');
            output.push_str(&attachment.media_type);
            output.push(':');
            output.push_str(&attachment.uri);
            output.push(';');
        }

        output.push_str("|proof|");
        output.push_str(&self.proof.type_name);
        output.push('|');
        output.push_str(&self.proof.created);
        output.push('|');
        output.push_str(&self.proof.verification_method);
        output.push('|');
        output.push_str(&self.proof.proof_purpose);
        output.push('|');
        output.push_str(&self.proof.proof_value);
        output
    }
}

/// Validation and normalization error taxonomy for canonical message envelopes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageEnvelopeError {
    /// A required field is empty.
    EmptyField(&'static str),
    /// Envelope type does not match the canonical type.
    InvalidEnvelopeType(String),
    /// Sender DID is invalid.
    InvalidSenderDid(String),
    /// Recipient list is empty.
    EmptyRecipients,
    /// A recipient DID is invalid.
    InvalidRecipientDid(String),
    /// Envelope expiry is not strictly after creation time.
    InvalidExpiryWindow {
        /// Creation timestamp string.
        created: String,
        /// Expiration timestamp string.
        expires: String,
    },
    /// Envelope nonce is invalid.
    InvalidNonce(u64),
    /// Header message type is not supported.
    InvalidMessageType(String),
    /// Header encryption algorithm does not match the canonical algorithm.
    InvalidEncryptionAlgorithm(String),
    /// Header recipient key list is empty.
    EmptyRecipientKeys,
    /// Body map is empty.
    EmptyBody,
    /// Body entry contains invalid key/value data.
    InvalidBodyEntry(String),
    /// Attachment has an invalid required field.
    InvalidAttachmentField {
        /// Attachment identifier.
        attachment_id: String,
        /// Field name that failed validation.
        field: &'static str,
    },
    /// Proof purpose does not match the canonical purpose.
    InvalidProofPurpose(String),
    /// Proof verification method does not match sender DID prefix.
    ProofVerificationMethodMismatch {
        /// Expected verification method DID URL prefix.
        expected_prefix: String,
        /// Observed verification method DID URL.
        actual: String,
    },
}

impl fmt::Display for MessageEnvelopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::InvalidEnvelopeType(value) => write!(f, "invalid envelope type: {value}"),
            Self::InvalidSenderDid(value) => write!(f, "invalid sender did: {value}"),
            Self::EmptyRecipients => write!(f, "envelope.to must contain at least one recipient"),
            Self::InvalidRecipientDid(value) => write!(f, "invalid recipient did: {value}"),
            Self::InvalidExpiryWindow { created, expires } => write!(
                f,
                "invalid expiry window, created {created}, expires {expires}"
            ),
            Self::InvalidNonce(value) => write!(f, "nonce must be positive: {value}"),
            Self::InvalidMessageType(value) => write!(f, "invalid header message type: {value}"),
            Self::InvalidEncryptionAlgorithm(value) => {
                write!(f, "invalid encryption algorithm: {value}")
            }
            Self::EmptyRecipientKeys => {
                write!(f, "header.encryption.recipient_keys must not be empty")
            }
            Self::EmptyBody => write!(f, "body must contain at least one entry"),
            Self::InvalidBodyEntry(value) => write!(f, "invalid body entry: {value}"),
            Self::InvalidAttachmentField {
                attachment_id,
                field,
            } => {
                write!(f, "attachment field {field} is empty for attachment {attachment_id}")
            }
            Self::InvalidProofPurpose(value) => write!(f, "invalid proof purpose: {value}"),
            Self::ProofVerificationMethodMismatch {
                expected_prefix,
                actual,
            } => write!(
                f,
                "proof verification method mismatch, expected prefix {expected_prefix}, got {actual}"
            ),
        }
    }
}

impl std::error::Error for MessageEnvelopeError {}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), MessageEnvelopeError> {
    if value.trim().is_empty() {
        return Err(MessageEnvelopeError::EmptyField(field));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        AttachmentRef, CanonicalMessageEnvelope, EnvelopeEncryption, EnvelopeHeader,
        EnvelopeMetadata, EnvelopeProof, MessageEnvelopeError, CANONICAL_ENCRYPTION_ALGORITHM,
        CANONICAL_MESSAGE_ENVELOPE_TYPE, CANONICAL_PROOF_PURPOSE,
    };
    use std::collections::BTreeMap;

    fn valid_envelope() -> CanonicalMessageEnvelope {
        let mut body = BTreeMap::new();
        body.insert("task.type".to_owned(), "research".to_owned());
        body.insert(
            "task.description".to_owned(),
            "analyze market trends".to_owned(),
        );

        CanonicalMessageEnvelope {
            envelope: EnvelopeMetadata {
                id: "urn:uuid:msg-1".to_owned(),
                type_name: CANONICAL_MESSAGE_ENVELOPE_TYPE.to_owned(),
                from: "kamn:did:agent:sender-1".to_owned(),
                to: vec!["kamn:did:agent:recipient-1".to_owned()],
                created: "2026-02-07T20:15:30.123Z".to_owned(),
                expires: "2026-02-07T20:45:30.123Z".to_owned(),
                thread_id: Some("urn:uuid:thread-1".to_owned()),
                parent_id: None,
                nonce: 42,
            },
            header: EnvelopeHeader {
                message_type: "Request".to_owned(),
                priority: "Elevated".to_owned(),
                content_type: "application/json".to_owned(),
                encryption: EnvelopeEncryption {
                    algorithm: CANONICAL_ENCRYPTION_ALGORITHM.to_owned(),
                    recipient_keys: vec!["kamn:did:agent:recipient-1#key-agreement-1".to_owned()],
                },
            },
            body,
            attachments: vec![AttachmentRef {
                id: "attachment-1".to_owned(),
                media_type: "application/pdf".to_owned(),
                uri: "ipfs://Qm123".to_owned(),
            }],
            proof: EnvelopeProof {
                type_name: "Ed25519Signature2020".to_owned(),
                created: "2026-02-07T20:15:30.123Z".to_owned(),
                verification_method: "kamn:did:agent:sender-1#keys-1".to_owned(),
                proof_purpose: CANONICAL_PROOF_PURPOSE.to_owned(),
                proof_value: "z58proof".to_owned(),
            },
        }
    }

    #[test]
    fn validate_rejects_invalid_message_type() {
        let mut envelope = valid_envelope();
        envelope.header.message_type = "UnknownType".to_owned();

        assert_eq!(
            envelope.validate(),
            Err(MessageEnvelopeError::InvalidMessageType(
                "UnknownType".to_owned()
            ))
        );
    }

    #[test]
    fn canonical_payload_orders_attachment_ids() {
        let mut envelope = valid_envelope();
        envelope.attachments.push(AttachmentRef {
            id: "attachment-0".to_owned(),
            media_type: "text/plain".to_owned(),
            uri: "ipfs://Qm999".to_owned(),
        });

        let payload = envelope.canonical_payload();
        let first = payload
            .find("attachment-0")
            .expect("payload must include attachment-0");
        let second = payload
            .find("attachment-1")
            .expect("payload must include attachment-1");
        assert!(first < second);
    }
}
