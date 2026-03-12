use super::super::env::{SECURE_AWS_KMS_BACKEND_NAME, SECURE_MOCK_BACKEND_NAME};
use super::super::errors::SignerBackendError;

/// Supported secure signing providers for canonical secure key references.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecureSignerProvider {
    /// In-memory deterministic provider used for local/dev validation flows.
    Mock,
    /// AWS KMS-compatible emulator provider.
    AwsKmsEmulator,
}

impl SecureSignerProvider {
    /// Resolve provider from a secure key reference.
    pub fn from_key_id(key_id: &str) -> Result<Self, SignerBackendError> {
        Ok(super::CanonicalSecureKeyReference::parse(key_id)?.provider)
    }

    /// Return canonical backend name for this secure provider.
    pub fn backend_name(self) -> &'static str {
        match self {
            Self::Mock => SECURE_MOCK_BACKEND_NAME,
            Self::AwsKmsEmulator => SECURE_AWS_KMS_BACKEND_NAME,
        }
    }

    pub(super) fn from_label(label: &str, key_id: &str) -> Result<Self, SignerBackendError> {
        match label {
            "mock" => Ok(Self::Mock),
            "aws-kms" => Ok(Self::AwsKmsEmulator),
            _ => Err(SignerBackendError::UnsupportedSecureProvider {
                backend: SECURE_MOCK_BACKEND_NAME.to_owned(),
                provider: label.to_owned(),
                key_id: key_id.to_owned(),
            }),
        }
    }
}

/// Handshake outcome classes for secure signer providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignerProviderHandshakeStatus {
    /// Provider is reachable and policy-authorized.
    Available,
    /// Provider transport is unavailable.
    Unavailable,
    /// Provider is reachable but blocked by policy.
    PolicyBlocked,
}

/// Handshake status matrix keyed by secure provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignerProviderHandshakeMatrix {
    mock_status: SignerProviderHandshakeStatus,
    aws_kms_status: SignerProviderHandshakeStatus,
}

impl SignerProviderHandshakeMatrix {
    /// Construct a matrix where both providers share the same availability class.
    pub fn with_uniform_availability(available: bool) -> Self {
        let status = if available {
            SignerProviderHandshakeStatus::Available
        } else {
            SignerProviderHandshakeStatus::Unavailable
        };
        Self::with_statuses(status, status)
    }

    /// Construct a matrix with explicit status values for each provider.
    pub fn with_statuses(
        mock_status: SignerProviderHandshakeStatus,
        aws_kms_status: SignerProviderHandshakeStatus,
    ) -> Self {
        Self {
            mock_status,
            aws_kms_status,
        }
    }

    pub(crate) fn status_for_provider(
        &self,
        provider: SecureSignerProvider,
    ) -> SignerProviderHandshakeStatus {
        match provider {
            SecureSignerProvider::Mock => self.mock_status,
            SecureSignerProvider::AwsKmsEmulator => self.aws_kms_status,
        }
    }
}
