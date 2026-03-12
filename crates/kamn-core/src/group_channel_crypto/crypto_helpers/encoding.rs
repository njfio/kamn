use super::super::GroupChannelCryptoError;

pub(crate) fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|value| format!("{value:02x}")).collect()
}

pub(crate) fn hex_decode(value: &str) -> Result<Vec<u8>, GroupChannelCryptoError> {
    if !value.len().is_multiple_of(2) {
        return Err(GroupChannelCryptoError::InvalidCiphertextEncoding);
    }

    let mut bytes = Vec::with_capacity(value.len() / 2);
    for chunk in value.as_bytes().chunks_exact(2) {
        let encoded = std::str::from_utf8(chunk)
            .map_err(|_| GroupChannelCryptoError::InvalidCiphertextEncoding)?;
        let byte = u8::from_str_radix(encoded, 16)
            .map_err(|_| GroupChannelCryptoError::InvalidCiphertextEncoding)?;
        bytes.push(byte);
    }
    Ok(bytes)
}
