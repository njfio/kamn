use super::errors::SignerBackendError;

mod canonical_reference;
mod key_roles;
mod models;
mod provider_status;

pub use canonical_reference::CanonicalSecureKeyReference;
pub use key_roles::SignerKeyRole;
pub use models::BackendSignature;
pub use provider_status::{
    SecureSignerProvider, SignerProviderHandshakeMatrix, SignerProviderHandshakeStatus,
};

pub(super) fn parse_key_role(
    provider_key_id: &str,
    key_id: &str,
) -> Result<SignerKeyRole, SignerBackendError> {
    key_roles::parse_provider_key_role(provider_key_id, key_id)
}
