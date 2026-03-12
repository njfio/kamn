use super::GroupChannelCryptoError;
use std::fmt;

impl fmt::Display for GroupChannelCryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNonce(value) => nonce_message(f, "nonce must be positive", *value),
            Self::NonceReuse(value) => nonce_message(f, "nonce reuse detected", *value),
            Self::InvalidDid {
                field,
                reason_code,
                detail,
            } => invalid_did_message(f, field, reason_code, detail),
            Self::SenderKeyNotFound(value) => sender_key_not_found_message(f, value),
            Self::UnknownSenderKeyGeneration {
                sender_did,
                key_generation,
            } => unknown_generation_message(f, sender_did, *key_generation),
            Self::RecipientNotAuthorized {
                recipient_did,
                sender_did,
                key_generation,
            } => recipient_not_authorized_message(f, recipient_did, sender_did, *key_generation),
            Self::ChannelMismatch { expected, actual } => {
                channel_mismatch_message(f, expected, actual)
            }
            other => write!(f, "{}", simple_message(other)),
        }
    }
}

fn simple_message(error: &GroupChannelCryptoError) -> &'static str {
    if let Some(message) = policy_or_input_message(error) {
        return message;
    }
    if let Some(message) = sender_ref_or_transport_message(error) {
        return message;
    }
    unreachable!("caller filters dynamic variants")
}

fn policy_or_input_message(error: &GroupChannelCryptoError) -> Option<&'static str> {
    match error {
        GroupChannelCryptoError::InsecureCryptoDisabled
        | GroupChannelCryptoError::MissingKeyAgreementMasterSeed
        | GroupChannelCryptoError::InvalidKeyAgreementMasterSeed => {
            Some(crypto_policy_message(error))
        }
        GroupChannelCryptoError::EmptyChannelId
        | GroupChannelCryptoError::EmptyRecipients
        | GroupChannelCryptoError::EmptyPayload => Some(input_message(error)),
        _ => None,
    }
}

fn sender_ref_or_transport_message(error: &GroupChannelCryptoError) -> Option<&'static str> {
    match error {
        GroupChannelCryptoError::InvalidSenderKeyRef => {
            Some("sender key reference must include #sender-key-")
        }
        GroupChannelCryptoError::AlgorithmMismatch
        | GroupChannelCryptoError::SignatureMismatch
        | GroupChannelCryptoError::EncryptionFailed
        | GroupChannelCryptoError::KeyDerivationFailed
        | GroupChannelCryptoError::IntegrityCheckFailed
        | GroupChannelCryptoError::InvalidCiphertextEncoding => Some(transport_message(error)),
        _ => None,
    }
}

fn crypto_policy_message(error: &GroupChannelCryptoError) -> &'static str {
    match error {
        GroupChannelCryptoError::InsecureCryptoDisabled => {
            "legacy deterministic group-message crypto has been removed"
        }
        GroupChannelCryptoError::MissingKeyAgreementMasterSeed => {
            "missing required key-agreement seed KAMN_KEY_AGREEMENT_MASTER_SEED_HEX"
        }
        GroupChannelCryptoError::InvalidKeyAgreementMasterSeed => {
            "invalid key-agreement seed KAMN_KEY_AGREEMENT_MASTER_SEED_HEX; expected 32-byte hex"
        }
        _ => unreachable!("caller filters crypto-policy variants"),
    }
}

fn input_message(error: &GroupChannelCryptoError) -> &'static str {
    match error {
        GroupChannelCryptoError::EmptyChannelId => "channel_id must not be empty",
        GroupChannelCryptoError::EmptyRecipients => "recipient allowlist must not be empty",
        GroupChannelCryptoError::EmptyPayload => "plaintext payload must not be empty",
        _ => unreachable!("caller filters input variants"),
    }
}

fn transport_message(error: &GroupChannelCryptoError) -> &'static str {
    match error {
        GroupChannelCryptoError::AlgorithmMismatch => "group message algorithm mismatch",
        GroupChannelCryptoError::SignatureMismatch => "group message signature verification failed",
        GroupChannelCryptoError::EncryptionFailed => "group message encryption failed",
        GroupChannelCryptoError::KeyDerivationFailed => "group message key derivation failed",
        GroupChannelCryptoError::IntegrityCheckFailed => "group message integrity check failed",
        GroupChannelCryptoError::InvalidCiphertextEncoding => "invalid ciphertext encoding",
        _ => unreachable!("caller filters transport variants"),
    }
}

fn nonce_message(f: &mut fmt::Formatter<'_>, prefix: &str, value: u64) -> fmt::Result {
    write!(f, "{prefix}: {value}")
}

fn invalid_did_message(
    f: &mut fmt::Formatter<'_>,
    field: &'static str,
    reason_code: &'static str,
    detail: &str,
) -> fmt::Result {
    write!(f, "invalid did field {field}: {reason_code} ({detail})")
}

fn sender_key_not_found_message(f: &mut fmt::Formatter<'_>, value: &str) -> fmt::Result {
    write!(f, "sender key not found: {value}")
}

fn unknown_generation_message(
    f: &mut fmt::Formatter<'_>,
    sender_did: &str,
    key_generation: u64,
) -> fmt::Result {
    write!(
        f,
        "unknown sender key generation {key_generation} for {sender_did}"
    )
}

fn recipient_not_authorized_message(
    f: &mut fmt::Formatter<'_>,
    recipient_did: &str,
    sender_did: &str,
    key_generation: u64,
) -> fmt::Result {
    write!(
        f,
        "recipient {recipient_did} is not authorized for {sender_did} generation {key_generation}"
    )
}

fn channel_mismatch_message(
    f: &mut fmt::Formatter<'_>,
    expected: &str,
    actual: &str,
) -> fmt::Result {
    write!(
        f,
        "group message channel mismatch, expected {expected}, got {actual}"
    )
}
