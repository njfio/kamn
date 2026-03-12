use super::super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthenticatedPeerFrameError {
    /// Invalid frame id.
    InvalidFrameId,
    /// Invalid sender did.
    InvalidSenderDid {
        /// Input field carrying invalid DID.
        field: &'static str,
        /// Stable deterministic reason marker.
        reason_code: &'static str,
        /// Canonical parser detail.
        detail: String,
    },
    /// Invalid recipient did.
    InvalidRecipientDid {
        /// Input field carrying invalid DID.
        field: &'static str,
        /// Stable deterministic reason marker.
        reason_code: &'static str,
        /// Canonical parser detail.
        detail: String,
    },
    /// Invalid local peer did.
    InvalidLocalPeerDid {
        /// Input field carrying invalid DID.
        field: &'static str,
        /// Stable deterministic reason marker.
        reason_code: &'static str,
        /// Canonical parser detail.
        detail: String,
    },
    /// Empty allowed senders.
    EmptyAllowedSenders,
    /// Invalid nonce.
    InvalidNonce,
    /// Empty payload.
    EmptyPayload,
    /// Empty signature.
    EmptySignature,
    /// Invalid wire field delimiter.
    InvalidWireFieldDelimiter {
        /// Field.
        field: &'static str,
    },
    /// Invalid wire format.
    InvalidWireFormat(String),
    /// Signature mismatch.
    SignatureMismatch {
        /// Expected.
        expected: String,
        /// Found.
        found: String,
    },
    /// Unauthorized sender.
    UnauthorizedSender(String),
    /// Wrong recipient.
    WrongRecipient {
        /// Expected.
        expected: String,
        /// Found.
        found: String,
    },
    /// Replay nonce.
    ReplayNonce {
        /// Sender did.
        sender_did: String,
        /// Last nonce.
        last_nonce: u64,
        /// Found.
        found: u64,
    },
    /// Missing runtime peer signing key material.
    MissingSigningKeyMaterial,
    /// Runtime peer signing key material is malformed.
    InvalidSigningKeyMaterial,
}

impl Display for AuthenticatedPeerFrameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&peer_frame_error_message(self))
    }
}

impl Error for AuthenticatedPeerFrameError {}

fn peer_frame_error_message(error: &AuthenticatedPeerFrameError) -> String {
    if let Some(message) = simple_peer_frame_error_message(error) {
        return message.to_owned();
    }
    if let Some(message) = invalid_did_error_message(error) {
        return message;
    }
    if let Some(message) = contextual_peer_frame_error_message(error) {
        return message;
    }
    signing_key_error_message(error).to_owned()
}

fn simple_peer_frame_error_message(error: &AuthenticatedPeerFrameError) -> Option<&'static str> {
    match error {
        AuthenticatedPeerFrameError::InvalidFrameId => Some("peer frame id cannot be empty"),
        AuthenticatedPeerFrameError::EmptyAllowedSenders => {
            Some("allowed sender DID set cannot be empty")
        }
        AuthenticatedPeerFrameError::InvalidNonce => Some("peer frame nonce must be positive"),
        AuthenticatedPeerFrameError::EmptyPayload => Some("peer frame payload cannot be empty"),
        AuthenticatedPeerFrameError::EmptySignature => Some("peer frame signature cannot be empty"),
        _ => None,
    }
}

fn invalid_did_error_message(error: &AuthenticatedPeerFrameError) -> Option<String> {
    match error {
        AuthenticatedPeerFrameError::InvalidSenderDid {
            field,
            reason_code,
            detail,
        }
        | AuthenticatedPeerFrameError::InvalidRecipientDid {
            field,
            reason_code,
            detail,
        }
        | AuthenticatedPeerFrameError::InvalidLocalPeerDid {
            field,
            reason_code,
            detail,
        } => Some(format!("invalid did field {field}: {reason_code} ({detail})")),
        _ => None,
    }
}

fn contextual_peer_frame_error_message(error: &AuthenticatedPeerFrameError) -> Option<String> {
    match error {
        AuthenticatedPeerFrameError::InvalidWireFieldDelimiter { field } => {
            Some(format!("peer frame field contains unsupported wire delimiters: {field}"))
        }
        AuthenticatedPeerFrameError::InvalidWireFormat(payload) => {
            Some(format!("peer frame wire payload is invalid: {payload}"))
        }
        AuthenticatedPeerFrameError::SignatureMismatch { expected, found } => Some(
            signature_mismatch_message(expected, found),
        ),
        AuthenticatedPeerFrameError::UnauthorizedSender(value) => {
            Some(format!("peer frame sender is unauthorized: {value}"))
        }
        AuthenticatedPeerFrameError::WrongRecipient { expected, found } => {
            Some(wrong_recipient_message(expected, found))
        }
        AuthenticatedPeerFrameError::ReplayNonce {
            sender_did,
            last_nonce,
            found,
        } => Some(replay_nonce_message(sender_did, *last_nonce, *found)),
        _ => None,
    }
}

fn signature_mismatch_message(expected: &str, found: &str) -> String {
    format!("peer frame signature mismatch: expected {expected}, found {found}")
}

fn wrong_recipient_message(expected: &str, found: &str) -> String {
    format!("peer frame recipient mismatch: expected {expected}, found {found}")
}

fn replay_nonce_message(sender_did: &str, last_nonce: u64, found: u64) -> String {
    format!("peer frame nonce replay for {sender_did}: last {last_nonce}, found {found}")
}

fn signing_key_error_message(error: &AuthenticatedPeerFrameError) -> &'static str {
    match error {
        AuthenticatedPeerFrameError::MissingSigningKeyMaterial => {
            "runtime peer frame signing key material is missing; configure KAMN_RUNTIME_PEER_SIGNING_PRIVATE_KEY_HEX or KAMN_RUNTIME_PEER_SIGNING_PUBLIC_KEY_HEX"
        }
        AuthenticatedPeerFrameError::InvalidSigningKeyMaterial => {
            "runtime peer frame signing key material is invalid"
        }
        _ => unreachable!("non-signing error routed to signing_key_error_message"),
    }
}
