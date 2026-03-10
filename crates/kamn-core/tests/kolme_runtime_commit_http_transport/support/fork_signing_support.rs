use super::*;
pub(crate) fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&path).expect("temporary directory should be created");
    path
}

pub(crate) fn decode_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(10 + (byte - b'a')),
        b'A'..=b'F' => Some(10 + (byte - b'A')),
        _ => None,
    }
}

pub(crate) fn decode_hex_bytes(input: &str) -> Result<Vec<u8>, String> {
    let trimmed = input.trim();
    if !trimmed.len().is_multiple_of(2) {
        return Err("hex string must contain an even number of characters".to_owned());
    }

    let mut bytes = Vec::with_capacity(trimmed.len() / 2);
    for pair in trimmed.as_bytes().chunks_exact(2) {
        let high = decode_hex_nibble(pair[0])
            .ok_or_else(|| format!("invalid hex character: {}", pair[0] as char))?;
        let low = decode_hex_nibble(pair[1])
            .ok_or_else(|| format!("invalid hex character: {}", pair[1] as char))?;
        bytes.push((high << 4) | low);
    }

    Ok(bytes)
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

pub(crate) fn kolme_fork_live_smoke_signing_key() -> SigningKey {
    let private_key = decode_hex_bytes(KOLME_FORK_LIVE_SMOKE_SECRET_KEY_HEX)
        .expect("kolme fork live smoke private key hex must decode");
    SigningKey::from_slice(private_key.as_slice())
        .expect("kolme fork live smoke private key bytes must be valid secp256k1 key")
}

pub(crate) fn kolme_fork_live_smoke_pubkey_hex() -> String {
    let signing_key = kolme_fork_live_smoke_signing_key();
    encode_hex_lower(
        signing_key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    )
}

pub(crate) fn kolme_fork_sign_message(message: &str) -> (String, u8) {
    let signing_key = kolme_fork_live_smoke_signing_key();
    let (signature, recovery_id) = signing_key
        .sign_recoverable(message.as_bytes())
        .expect("kolme fork live smoke signature generation must succeed");
    let signature_bytes = signature.to_bytes();
    (
        encode_hex_lower(signature_bytes.as_ref()),
        recovery_id.to_byte(),
    )
}

