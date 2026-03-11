use sha2::{Digest, Sha256};

use super::AgentDidKeyBindingError;

pub(super) const AGENT_DID_KEY_BINDING_MARKER: &str = "--keyh-";
pub(super) const AGENT_DID_KEY_BINDING_HEX_LEN: usize = 32;

pub(super) fn constant_time_eq_bytes(lhs: &[u8], rhs: &[u8]) -> bool {
    if lhs.len() != rhs.len() {
        return false;
    }

    let mut diff = 0u8;
    for (&left, &right) in lhs.iter().zip(rhs.iter()) {
        diff |= left ^ right;
    }
    diff == 0
}

fn decode_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn decode_hex_bytes(value: &str) -> Option<Vec<u8>> {
    let bytes = value.as_bytes();
    if bytes.is_empty() || !bytes.len().is_multiple_of(2) {
        return None;
    }

    let mut output = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        let high = decode_hex_nibble(chunk[0])?;
        let low = decode_hex_nibble(chunk[1])?;
        output.push((high << 4) | low);
    }
    Some(output)
}

fn encode_hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

pub(super) fn fingerprint_for_public_key_hex(
    public_key_hex: &str,
) -> Result<String, AgentDidKeyBindingError> {
    let bytes = decode_hex_bytes(public_key_hex.trim())
        .ok_or(AgentDidKeyBindingError::InvalidPublicKeyHex)?;
    let digest = Sha256::digest(bytes.as_slice());
    let fingerprint_bytes = &digest[..(AGENT_DID_KEY_BINDING_HEX_LEN / 2)];
    Ok(encode_hex_lower(fingerprint_bytes))
}
