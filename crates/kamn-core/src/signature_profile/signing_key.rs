use k256::ecdsa::{RecoveryId, Signature, SigningKey};
use std::fmt;

use super::{decode_hex_nibble, encode_hex_lower, ServiceAuthSignatureError};
use crate::signature_profile::encoding::wipe_bytes;

pub(crate) struct ServiceAuthSigningKey {
    signing_key: SigningKey,
}

impl ServiceAuthSigningKey {
    pub(crate) fn from_private_key_hex(
        private_key_hex: &str,
    ) -> Result<Self, ServiceAuthSignatureError> {
        let mut private_key_bytes = decode_service_auth_private_key_hex(private_key_hex)?;
        let result = Self::from_private_key_bytes(&private_key_bytes);
        wipe_bytes(private_key_bytes.as_mut_slice());
        result
    }

    pub(crate) fn from_private_key_bytes(
        private_key_bytes: &[u8; 32],
    ) -> Result<Self, ServiceAuthSignatureError> {
        let signing_key = SigningKey::from_slice(private_key_bytes.as_slice())
            .map_err(|_| ServiceAuthSignatureError::InvalidPrivateKeyHex)?;
        Ok(Self { signing_key })
    }

    pub(crate) fn sign_message(
        &self,
        message: &str,
    ) -> Result<(Signature, RecoveryId), ServiceAuthSignatureError> {
        self.signing_key
            .sign_recoverable(message.as_bytes())
            .map_err(|_| ServiceAuthSignatureError::SigningFailure)
    }

    pub(crate) fn public_key_hex(&self) -> String {
        encode_hex_lower(
            self.signing_key
                .verifying_key()
                .to_encoded_point(true)
                .as_bytes(),
        )
    }
}

impl fmt::Debug for ServiceAuthSigningKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ServiceAuthSigningKey(<redacted>)")
    }
}

pub(crate) fn decode_service_auth_private_key_hex(
    private_key_hex: &str,
) -> Result<[u8; 32], ServiceAuthSignatureError> {
    let trimmed = private_key_hex.trim();
    if trimmed.len() != 64 {
        return Err(ServiceAuthSignatureError::InvalidPrivateKeyHex);
    }
    decode_fixed_private_key_bytes(trimmed)
}

fn decode_fixed_private_key_bytes(
    private_key_hex: &str,
) -> Result<[u8; 32], ServiceAuthSignatureError> {
    let mut private_key_bytes = [0_u8; 32];
    for (index, pair) in private_key_hex.as_bytes().chunks_exact(2).enumerate() {
        private_key_bytes[index] = decode_private_key_byte(pair)?;
    }
    Ok(private_key_bytes)
}

fn decode_private_key_byte(pair: &[u8]) -> Result<u8, ServiceAuthSignatureError> {
    let high = decode_hex_nibble(pair[0]).ok_or(ServiceAuthSignatureError::InvalidPrivateKeyHex)?;
    let low = decode_hex_nibble(pair[1]).ok_or(ServiceAuthSignatureError::InvalidPrivateKeyHex)?;
    Ok((high << 4) | low)
}
