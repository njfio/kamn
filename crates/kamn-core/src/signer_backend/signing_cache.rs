use super::env::resolve_signer_private_key_hex;
use super::errors::SignerBackendError;
use super::request::SigningRequest;
use crate::constant_time_eq::constant_time_eq_bytes;
use crate::signature_profile::{
    decode_service_auth_private_key_hex, service_auth_sign_with_signing_key, ServiceAuthSigningKey,
};
use k256::elliptic_curve::zeroize::Zeroize;
use std::sync::Mutex;

#[derive(Debug, Default)]
pub(crate) struct SignerBackendSigningCache {
    cached_key: Mutex<Option<CachedSigningKey>>,
}

impl SignerBackendSigningCache {
    pub(crate) fn expected_signature(
        &self,
        request: &SigningRequest,
    ) -> Result<String, SignerBackendError> {
        let private_key_hex = resolve_signer_private_key_hex(&request.key_id)?;
        let private_key_bytes = decode_signing_key_bytes(&request.key_id, &private_key_hex)?;
        self.sign_with_private_key_bytes(request, private_key_bytes)
    }

    fn sign_with_private_key_bytes(
        &self,
        request: &SigningRequest,
        mut private_key_bytes: [u8; 32],
    ) -> Result<String, SignerBackendError> {
        let signature = self.sign_with_private_key_bytes_ref(request, &private_key_bytes);
        private_key_bytes.zeroize();
        signature
    }

    fn sign_with_private_key_bytes_ref(
        &self,
        request: &SigningRequest,
        private_key_bytes: &[u8; 32],
    ) -> Result<String, SignerBackendError> {
        let mut cached_key = self
            .cached_key
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        refresh_cached_signing_key(&mut cached_key, &request.key_id, private_key_bytes)?;
        cached_key
            .as_ref()
            .ok_or_else(|| invalid_signing_key_material(&request.key_id))?
            .sign(request)
    }
}

#[derive(Debug)]
struct CachedSigningKey {
    private_key_bytes: [u8; 32],
    signing_key: ServiceAuthSigningKey,
}

impl CachedSigningKey {
    fn from_private_key_bytes(
        key_id: &str,
        private_key_bytes: &[u8; 32],
    ) -> Result<Self, SignerBackendError> {
        let signing_key = ServiceAuthSigningKey::from_private_key_bytes(private_key_bytes)
            .map_err(|_| invalid_signing_key_material(key_id))?;
        Ok(Self {
            private_key_bytes: *private_key_bytes,
            signing_key,
        })
    }

    fn matches_private_key_bytes(&self, private_key_bytes: &[u8; 32]) -> bool {
        constant_time_eq_bytes(
            self.private_key_bytes.as_slice(),
            private_key_bytes.as_slice(),
        )
    }

    fn sign(&self, request: &SigningRequest) -> Result<String, SignerBackendError> {
        service_auth_sign_with_signing_key(
            &request.sender,
            request.nonce,
            &request.state_hash,
            &request.payload,
            &self.signing_key,
        )
        .map_err(|_| invalid_signing_key_material(&request.key_id))
    }
}

impl Drop for CachedSigningKey {
    fn drop(&mut self) {
        self.private_key_bytes.zeroize();
    }
}

fn decode_signing_key_bytes(
    key_id: &str,
    private_key_hex: &str,
) -> Result<[u8; 32], SignerBackendError> {
    decode_service_auth_private_key_hex(private_key_hex)
        .map_err(|_| invalid_signing_key_material(key_id))
}

fn refresh_cached_signing_key(
    cached_key: &mut Option<CachedSigningKey>,
    key_id: &str,
    private_key_bytes: &[u8; 32],
) -> Result<(), SignerBackendError> {
    if cache_matches(cached_key, private_key_bytes) {
        return Ok(());
    }
    *cached_key = Some(CachedSigningKey::from_private_key_bytes(
        key_id,
        private_key_bytes,
    )?);
    Ok(())
}

fn cache_matches(cached_key: &Option<CachedSigningKey>, private_key_bytes: &[u8; 32]) -> bool {
    cached_key
        .as_ref()
        .is_some_and(|cached_key| cached_key.matches_private_key_bytes(private_key_bytes))
}

fn invalid_signing_key_material(key_id: &str) -> SignerBackendError {
    SignerBackendError::InvalidSigningKeyMaterial {
        key_id: key_id.to_owned(),
    }
}
