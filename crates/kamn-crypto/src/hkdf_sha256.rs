use hkdf::Hkdf;
use sha2::Sha256;
use std::fmt;

/// Marker asserting HKDF derivation is backed by RustCrypto hkdf crate.
pub const HKDF_SHA256_BACKEND_MARKER: &str = "rustcrypto.hkdf.sha256.v1";
/// Marker asserting HMAC backend semantics are provided by RustCrypto primitives.
pub const HMAC_SHA256_BACKEND_MARKER: &str = "rustcrypto.hmac.sha256.v1";

/// Error emitted when HKDF expansion to a fixed output length fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HkdfSha256Error;

impl fmt::Display for HkdfSha256Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "hkdf sha256 expansion failed")
    }
}

impl std::error::Error for HkdfSha256Error {}

/// Derives a 32-byte key using HKDF-SHA256 for the given `salt`, `ikm`, and
/// `info` labels.
pub fn derive_key_32(salt: &[u8], ikm: &[u8], info: &[u8]) -> Result<[u8; 32], HkdfSha256Error> {
    let hkdf = Hkdf::<Sha256>::new(Some(salt), ikm);
    let mut key = [0u8; 32];
    hkdf.expand(info, &mut key).map_err(|_| HkdfSha256Error)?;
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::{derive_key_32, HKDF_SHA256_BACKEND_MARKER, HMAC_SHA256_BACKEND_MARKER};

    #[test]
    fn derive_key_32_is_deterministic_for_same_inputs() {
        let salt = b"salt";
        let ikm = b"ikm";
        let info = b"info";

        let first = derive_key_32(salt, ikm, info).expect("hkdf should derive");
        let second = derive_key_32(salt, ikm, info).expect("hkdf should derive");
        assert_eq!(first, second);
    }

    #[test]
    fn backend_markers_are_stable() {
        assert_eq!(HKDF_SHA256_BACKEND_MARKER, "rustcrypto.hkdf.sha256.v1");
        assert_eq!(HMAC_SHA256_BACKEND_MARKER, "rustcrypto.hmac.sha256.v1");
    }
}
