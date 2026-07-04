use k256::elliptic_curve::zeroize::Zeroize;

use super::{
    ServiceAuthSignatureError, SignatureProfileMetadata, BASELINE_SIGNATURE_ALGORITHM,
    BASELINE_SIGNATURE_PROFILE_ID,
};

/// Runs the parse signature profile metadata contract helper.
pub fn parse_signature_profile_metadata(signature: &str) -> Option<SignatureProfileMetadata> {
    let suffix = signature.strip_prefix("sig:")?;
    let mut segments = suffix.splitn(3, ':');
    let algorithm = segments.next()?.trim();
    let profile_id = segments.next()?.trim();
    let payload_segments = segments.next()?.trim();
    if algorithm.is_empty() || profile_id.is_empty() || payload_segments.is_empty() {
        return None;
    }
    Some(SignatureProfileMetadata {
        algorithm: algorithm.to_owned(),
        profile_id: profile_id.to_owned(),
    })
}

pub(crate) fn decode_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(10 + (byte - b'a')),
        b'A'..=b'F' => Some(10 + (byte - b'A')),
        _ => None,
    }
}

pub(crate) fn decode_hex_bytes(value: &str) -> Result<Vec<u8>, ServiceAuthSignatureError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || !trimmed.len().is_multiple_of(2) {
        return Err(ServiceAuthSignatureError::InvalidSignatureHex);
    }
    let mut decoded = Vec::with_capacity(trimmed.len() / 2);
    for pair in trimmed.as_bytes().chunks_exact(2) {
        let high =
            decode_hex_nibble(pair[0]).ok_or(ServiceAuthSignatureError::InvalidSignatureHex)?;
        let low =
            decode_hex_nibble(pair[1]).ok_or(ServiceAuthSignatureError::InvalidSignatureHex)?;
        decoded.push((high << 4) | low);
    }
    Ok(decoded)
}

pub(crate) fn encode_hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

pub(crate) fn wipe_bytes(bytes: &mut [u8]) {
    bytes.zeroize();
}

fn canonical_service_auth_message(
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
) -> String {
    format!(
        "sender_len={}\nsender={sender}\nnonce={nonce}\nstate_hash_len={}\nstate_hash={state_hash}\npayload_len={}\npayload={payload}",
        sender.len(),
        state_hash.len(),
        payload.len()
    )
}

/// Runs the service auth signing payload for fields contract helper.
pub fn service_auth_signing_payload_for_fields(
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
) -> Result<String, ServiceAuthSignatureError> {
    if sender.trim().is_empty() {
        return Err(ServiceAuthSignatureError::EmptyField("sender"));
    }
    if nonce == 0 {
        return Err(ServiceAuthSignatureError::InvalidNonce);
    }
    if state_hash.trim().is_empty() {
        return Err(ServiceAuthSignatureError::EmptyField("state_hash"));
    }
    Ok(canonical_service_auth_message(
        sender, nonce, state_hash, payload,
    ))
}

pub(crate) fn baseline_signature_for_fields(
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
) -> String {
    format!(
        "sig:{}:{}:{}:{}:{}:{}",
        BASELINE_SIGNATURE_ALGORITHM,
        BASELINE_SIGNATURE_PROFILE_ID,
        sender,
        nonce,
        state_hash,
        payload.len()
    )
}
