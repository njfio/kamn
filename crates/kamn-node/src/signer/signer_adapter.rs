use k256::ecdsa::{RecoveryId, Signature, SigningKey, VerifyingKey};
use kamn_core::ConfigError;
use zeroize::Zeroize;

#[cfg(test)]
use super::signer_policy::resolve_kolme_live_signer_env_name_set;

#[derive(Debug, Clone)]
pub(crate) struct KolmeForkSecp256k1SignerAdapter {
    signing_key: SigningKey,
    pub(crate) private_key_env: &'static str,
}

fn decode_kolme_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(10 + (byte - b'a')),
        b'A'..=b'F' => Some(10 + (byte - b'A')),
        _ => None,
    }
}

pub(crate) fn decode_kolme_hex_bytes(value: &str, env_name: &str) -> Result<Vec<u8>, ConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{env_name} must not be empty"
        )));
    }
    if !trimmed.len().is_multiple_of(2) {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{env_name} must have an even number of hex characters"
        )));
    }

    let mut decoded = Vec::with_capacity(trimmed.len() / 2);
    for pair in trimmed.as_bytes().chunks_exact(2) {
        let high = match decode_kolme_hex_nibble(pair[0]) {
            Some(value) => value,
            None => {
                decoded.zeroize();
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "{env_name} contains invalid hex character '{}'",
                    pair[0] as char
                )));
            }
        };
        let low = match decode_kolme_hex_nibble(pair[1]) {
            Some(value) => value,
            None => {
                decoded.zeroize();
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "{env_name} contains invalid hex character '{}'",
                    pair[1] as char
                )));
            }
        };
        decoded.push((high << 4) | low);
    }

    Ok(decoded)
}

pub(crate) fn encode_kolme_hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

#[cfg(test)]
fn derive_kolme_live_managed_signing_key_material(key_reference: &str) -> [u8; 32] {
    let mut material = [0u8; 32];
    for (index, byte) in key_reference.as_bytes().iter().copied().enumerate() {
        let slot = index % material.len();
        let salt = ((index as u8).wrapping_mul(17)).wrapping_add(31);
        material[slot] = material[slot].wrapping_add(byte).wrapping_add(salt);
    }
    if material.iter().all(|value| *value == 0) {
        material[0] = 1;
    }
    material
}

#[cfg(test)]
pub(crate) fn build_kolme_live_managed_signing_key(
    key_reference: &str,
) -> Result<SigningKey, ConfigError> {
    let mut key_material = derive_kolme_live_managed_signing_key_material(key_reference);
    let signing_key = SigningKey::from_slice(&key_material).map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "failed to derive managed-external signer key material for {key_reference}: {error} (managed_signer_key_material_invalid)"
        ))
    });
    key_material.zeroize();
    signing_key
}

#[cfg(test)]
pub(crate) fn resolve_kolme_live_signer_private_key_env_name(
    strict_signer_profile: Option<&str>,
) -> Result<(&'static str, &'static str), ConfigError> {
    let (profile, private_key_env, _) =
        resolve_kolme_live_signer_env_name_set(strict_signer_profile)?;
    Ok((profile, private_key_env))
}

impl KolmeForkSecp256k1SignerAdapter {
    pub(crate) fn from_private_key_hex_in_place(
        private_key_hex: &mut String,
        private_key_env: &'static str,
    ) -> Result<Self, ConfigError> {
        let decode_result = decode_kolme_hex_bytes(private_key_hex.as_str(), private_key_env);
        let mut private_key_bytes = match decode_result {
            Ok(bytes) => bytes,
            Err(error) => {
                private_key_hex.zeroize();
                return Err(error);
            }
        };
        let signing_key_result =
            SigningKey::from_slice(private_key_bytes.as_slice()).map_err(|error| {
                ConfigError::RuntimeKolmeLive(format!(
                    "{private_key_env} is not a valid secp256k1 private key: {error}",
                ))
            });
        private_key_bytes.zeroize();
        private_key_hex.zeroize();
        let signing_key = signing_key_result?;
        Ok(Self {
            signing_key,
            private_key_env,
        })
    }

    #[cfg(test)]
    pub(crate) fn from_private_key_hex(
        private_key_hex: &str,
        private_key_env: &'static str,
    ) -> Result<Self, ConfigError> {
        let mut private_key_hex = private_key_hex.to_owned();
        Self::from_private_key_hex_in_place(&mut private_key_hex, private_key_env)
    }

    pub(crate) fn public_key_compressed_hex(&self) -> String {
        encode_kolme_hex_lower(
            self.signing_key
                .verifying_key()
                .to_encoded_point(true)
                .as_bytes(),
        )
    }

    pub(crate) fn verify_message(
        &self,
        message: &str,
        signature_hex: &str,
        recovery_id: u8,
    ) -> Result<(), ConfigError> {
        let signature_bytes =
            decode_kolme_hex_bytes(signature_hex, "runtime_commit_signature_hex")?;
        if signature_bytes.len() != 64 {
            return Err(ConfigError::RuntimeKolmeLive(
                "runtime commit signature hex must decode to exactly 64 bytes".to_owned(),
            ));
        }
        let signature = Signature::from_slice(signature_bytes.as_slice()).map_err(|error| {
            ConfigError::RuntimeKolmeLive(format!(
                "runtime commit signature bytes are invalid secp256k1 material: {error}",
            ))
        })?;
        let recovery = RecoveryId::from_byte(recovery_id).ok_or_else(|| {
            ConfigError::RuntimeKolmeLive(format!(
                "runtime commit recovery id must be within secp256k1 range [0,3], found {recovery_id}",
            ))
        })?;
        let recovered = VerifyingKey::recover_from_msg(message.as_bytes(), &signature, recovery)
            .map_err(|error| {
                ConfigError::RuntimeKolmeLive(format!(
                    "failed to recover secp256k1 public key from runtime commit signature: {error}",
                ))
            })?;
        if recovered != *self.signing_key.verifying_key() {
            return Err(ConfigError::RuntimeKolmeLive(format!(
                "runtime commit signature recovered public key does not match signer selection {}",
                self.private_key_env,
            )));
        }
        Ok(())
    }

    pub(crate) fn sign_message(&self, message: &str) -> Result<(String, u8), ConfigError> {
        let (signature, recovery_id) = self
            .signing_key
            .sign_recoverable(message.as_bytes())
            .map_err(|error| {
                ConfigError::RuntimeKolmeLive(format!(
                    "failed to sign live runtime commit payload: {error}",
                ))
            })?;
        let signature_hex = encode_kolme_hex_lower(signature.to_bytes().as_ref());
        let recovery_id = recovery_id.to_byte();
        self.verify_message(message, signature_hex.as_str(), recovery_id)?;
        Ok((signature_hex, recovery_id))
    }
}
