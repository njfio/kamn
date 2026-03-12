use super::{parse_key_role, SecureSignerProvider, SignerKeyRole};
use crate::signer_backend::errors::SignerBackendError;

/// Canonical parsed representation of a secure key reference string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalSecureKeyReference {
    /// Parsed secure-provider discriminator.
    pub provider: SecureSignerProvider,
    /// Parsed signer key role used for policy checks.
    pub key_role: SignerKeyRole,
    /// Provider-scoped key identifier payload.
    pub provider_key_id: String,
}

impl CanonicalSecureKeyReference {
    /// Parse secure key references across legacy and explicit provider formats.
    pub fn parse(key_id: &str) -> Result<Self, SignerBackendError> {
        if !key_id.starts_with("secure:") {
            return Err(SignerBackendError::UnsupportedKeyReference {
                backend: "secure-mock".to_owned(),
                key_id: key_id.to_owned(),
            });
        }
        let suffix = &key_id["secure:".len()..];
        if suffix.trim().is_empty() {
            return Err(SignerBackendError::MalformedSecureKeyReference {
                key_id: key_id.to_owned(),
            });
        }
        if let Some((provider_label, provider_key_id)) = suffix.split_once(':') {
            return parse_explicit_key_id(key_id, provider_label, provider_key_id);
        }
        Ok(Self {
            provider: SecureSignerProvider::Mock,
            key_role: SignerKeyRole::Operator,
            provider_key_id: suffix.to_owned(),
        })
    }
}

fn parse_explicit_key_id(
    key_id: &str,
    provider_label: &str,
    provider_key_id: &str,
) -> Result<CanonicalSecureKeyReference, SignerBackendError> {
    if provider_label.trim().is_empty() || provider_key_id.trim().is_empty() {
        return Err(SignerBackendError::MalformedSecureKeyReference {
            key_id: key_id.to_owned(),
        });
    }
    let normalized_provider_label = provider_label.trim().to_ascii_lowercase();
    let provider = SecureSignerProvider::from_label(&normalized_provider_label, key_id)?;
    let key_role = parse_key_role(provider_key_id, key_id)?;
    Ok(CanonicalSecureKeyReference {
        provider,
        key_role,
        provider_key_id: provider_key_id.to_owned(),
    })
}
