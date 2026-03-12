use super::{
    hex_decode, validate_key_ref_match, DirectMessageCiphertext, DirectMessageCryptoError,
    DIRECT_MESSAGE_CIPHER_ALGORITHM, DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM,
};
use std::collections::BTreeSet;

pub(crate) fn validate_encrypt_request(
    used_nonces: &mut BTreeSet<u64>,
    plaintext: &str,
    nonce: u64,
) -> Result<(), DirectMessageCryptoError> {
    if plaintext.is_empty() {
        return Err(DirectMessageCryptoError::EmptyPayload);
    }
    if nonce == 0 {
        return Err(DirectMessageCryptoError::InvalidNonce(nonce));
    }
    if !used_nonces.insert(nonce) {
        return Err(DirectMessageCryptoError::NonceReuse(nonce));
    }
    Ok(())
}

pub(crate) fn validate_ciphertext_context(
    sender_key_ref: &str,
    recipient_key_ref: &str,
    sealed: &DirectMessageCiphertext,
) -> Result<(), DirectMessageCryptoError> {
    if sealed.key_agreement_algorithm != DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM
        || sealed.cipher_algorithm != DIRECT_MESSAGE_CIPHER_ALGORITHM
    {
        return Err(DirectMessageCryptoError::AlgorithmMismatch);
    }
    validate_key_ref_match("sender", sender_key_ref, sealed.sender_key_ref.as_str())?;
    validate_key_ref_match(
        "recipient",
        recipient_key_ref,
        sealed.recipient_key_ref.as_str(),
    )?;
    if sealed.nonce == 0 {
        return Err(DirectMessageCryptoError::InvalidNonce(sealed.nonce));
    }
    Ok(())
}

pub(crate) fn decode_combined_ciphertext(
    sealed: &DirectMessageCiphertext,
) -> Result<Vec<u8>, DirectMessageCryptoError> {
    let ciphertext = hex_decode(&sealed.ciphertext)?;
    let auth_tag =
        hex_decode(&sealed.auth_tag).map_err(|_| DirectMessageCryptoError::IntegrityCheckFailed)?;
    let mut combined = ciphertext;
    combined.extend_from_slice(&auth_tag);
    Ok(combined)
}
