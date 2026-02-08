use crate::signature_profile::baseline_signature_for_fields;
use crate::transaction::BaselineTransaction;

const LOCAL_BACKEND_NAME: &str = "local-software";
const SECURE_MOCK_BACKEND_NAME: &str = "secure-mock";
const SECURE_AWS_KMS_BACKEND_NAME: &str = "secure-aws-kms-emulator";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SigningRequest {
    pub key_id: String,
    pub sender: String,
    pub nonce: u64,
    pub payload: String,
    pub state_hash: String,
}

impl SigningRequest {
    pub fn new(
        key_id: &str,
        sender: &str,
        nonce: u64,
        payload: &str,
        state_hash: &str,
    ) -> Result<Self, SignerBackendError> {
        if key_id.trim().is_empty() {
            return Err(SignerBackendError::EmptyField("key_id"));
        }
        if sender.trim().is_empty() {
            return Err(SignerBackendError::EmptyField("sender"));
        }
        if nonce == 0 {
            return Err(SignerBackendError::InvalidNonce);
        }
        if payload.trim().is_empty() {
            return Err(SignerBackendError::EmptyField("payload"));
        }
        if state_hash.trim().is_empty() {
            return Err(SignerBackendError::EmptyField("state_hash"));
        }

        Ok(Self {
            key_id: key_id.to_owned(),
            sender: sender.to_owned(),
            nonce,
            payload: payload.to_owned(),
            state_hash: state_hash.to_owned(),
        })
    }

    pub fn for_transaction(
        key_id: &str,
        tx: &BaselineTransaction,
    ) -> Result<Self, SignerBackendError> {
        if tx.id.trim().is_empty() {
            return Err(SignerBackendError::EmptyField("transaction_id"));
        }
        Self::new(key_id, &tx.sender, tx.nonce, &tx.payload, &tx.state_hash)
    }

    fn expected_signature(&self) -> String {
        baseline_signature_for_fields(&self.sender, self.nonce, &self.state_hash, &self.payload)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendSignature {
    pub backend: String,
    pub signature: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecureSignerProvider {
    Mock,
    AwsKmsEmulator,
}

impl SecureSignerProvider {
    pub fn from_key_id(key_id: &str) -> Result<Self, SignerBackendError> {
        Ok(SecureKeyReference::parse(key_id)?.provider)
    }

    pub fn backend_name(self) -> &'static str {
        match self {
            Self::Mock => SECURE_MOCK_BACKEND_NAME,
            Self::AwsKmsEmulator => SECURE_AWS_KMS_BACKEND_NAME,
        }
    }

    fn from_label(label: &str, key_id: &str) -> Result<Self, SignerBackendError> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct SecureKeyReference {
    provider: SecureSignerProvider,
    _provider_key_id: String,
}

impl SecureKeyReference {
    fn parse(key_id: &str) -> Result<Self, SignerBackendError> {
        if !key_id.starts_with("secure:") {
            return Err(SignerBackendError::UnsupportedKeyReference {
                backend: SECURE_MOCK_BACKEND_NAME.to_owned(),
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
            if provider_label.trim().is_empty() || provider_key_id.trim().is_empty() {
                return Err(SignerBackendError::MalformedSecureKeyReference {
                    key_id: key_id.to_owned(),
                });
            }
            let normalized_provider_label = provider_label.trim().to_ascii_lowercase();
            let provider = SecureSignerProvider::from_label(&normalized_provider_label, key_id)?;
            return Ok(Self {
                provider,
                _provider_key_id: provider_key_id.to_owned(),
            });
        }

        Ok(Self {
            provider: SecureSignerProvider::Mock,
            _provider_key_id: suffix.to_owned(),
        })
    }
}

pub trait SignerBackend {
    fn backend_name(&self) -> &'static str;
    fn sign(&self, request: &SigningRequest) -> Result<String, SignerBackendError>;
    fn verify(&self, request: &SigningRequest, signature: &str) -> Result<(), SignerBackendError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalSignerBackend;

impl SignerBackend for LocalSignerBackend {
    fn backend_name(&self) -> &'static str {
        LOCAL_BACKEND_NAME
    }

    fn sign(&self, request: &SigningRequest) -> Result<String, SignerBackendError> {
        Ok(request.expected_signature())
    }

    fn verify(&self, request: &SigningRequest, signature: &str) -> Result<(), SignerBackendError> {
        let expected = request.expected_signature();
        if expected != signature {
            return Err(SignerBackendError::SignatureMismatch {
                backend: self.backend_name().to_owned(),
                expected,
                found: signature.to_owned(),
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecureSignerBackend {
    available: bool,
}

impl SecureSignerBackend {
    pub fn new(available: bool) -> Self {
        Self { available }
    }

    fn sign_with_backend(
        &self,
        request: &SigningRequest,
    ) -> Result<BackendSignature, SignerBackendError> {
        let secure_key = SecureKeyReference::parse(&request.key_id)?;
        if !self.available {
            return Err(SignerBackendError::ProviderUnavailable {
                backend: secure_key.provider.backend_name().to_owned(),
            });
        }

        Ok(BackendSignature {
            backend: secure_key.provider.backend_name().to_owned(),
            signature: request.expected_signature(),
        })
    }

    fn verify_with_backend_name(
        &self,
        backend: &str,
        request: &SigningRequest,
        signature: &str,
    ) -> Result<(), SignerBackendError> {
        let secure_key = SecureKeyReference::parse(&request.key_id)?;
        let expected_backend = secure_key.provider.backend_name();
        if backend != expected_backend {
            return Err(SignerBackendError::SecureProviderBackendMismatch {
                expected_backend: expected_backend.to_owned(),
                provided_backend: backend.to_owned(),
                key_id: request.key_id.clone(),
            });
        }

        self.verify(request, signature)
    }
}

impl SignerBackend for SecureSignerBackend {
    fn backend_name(&self) -> &'static str {
        SECURE_MOCK_BACKEND_NAME
    }

    fn sign(&self, request: &SigningRequest) -> Result<String, SignerBackendError> {
        Ok(self.sign_with_backend(request)?.signature)
    }

    fn verify(&self, request: &SigningRequest, signature: &str) -> Result<(), SignerBackendError> {
        let secure_key = SecureKeyReference::parse(&request.key_id)?;
        if !self.available {
            return Err(SignerBackendError::ProviderUnavailable {
                backend: secure_key.provider.backend_name().to_owned(),
            });
        }

        let expected = request.expected_signature();
        if expected != signature {
            return Err(SignerBackendError::SignatureMismatch {
                backend: secure_key.provider.backend_name().to_owned(),
                expected,
                found: signature.to_owned(),
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignerBackendRouter {
    local: LocalSignerBackend,
    secure: SecureSignerBackend,
}

impl SignerBackendRouter {
    pub fn with_secure_availability(secure_available: bool) -> Self {
        Self {
            local: LocalSignerBackend,
            secure: SecureSignerBackend::new(secure_available),
        }
    }

    pub fn sign_with_secure_fallback(
        &self,
        request: &SigningRequest,
    ) -> Result<BackendSignature, SignerBackendError> {
        match self.secure.sign_with_backend(request) {
            Ok(signature) => Ok(signature),
            Err(SignerBackendError::ProviderUnavailable { .. }) => {
                let signature = self.local.sign(request)?;
                Ok(BackendSignature {
                    backend: self.local.backend_name().to_owned(),
                    signature,
                })
            }
            Err(error) => Err(error),
        }
    }

    pub fn verify_with_backend(
        &self,
        backend: &str,
        request: &SigningRequest,
        signature: &str,
    ) -> Result<(), SignerBackendError> {
        match backend {
            LOCAL_BACKEND_NAME => self.local.verify(request, signature),
            SECURE_MOCK_BACKEND_NAME | SECURE_AWS_KMS_BACKEND_NAME => self
                .secure
                .verify_with_backend_name(backend, request, signature),
            _ => Err(SignerBackendError::UnknownBackend(backend.to_owned())),
        }
    }
}

impl Default for SignerBackendRouter {
    fn default() -> Self {
        Self::with_secure_availability(true)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignerBackendError {
    EmptyField(&'static str),
    InvalidNonce,
    MalformedSecureKeyReference {
        key_id: String,
    },
    ProviderUnavailable {
        backend: String,
    },
    SecureProviderBackendMismatch {
        expected_backend: String,
        provided_backend: String,
        key_id: String,
    },
    SignatureMismatch {
        backend: String,
        expected: String,
        found: String,
    },
    UnknownBackend(String),
    UnsupportedSecureProvider {
        backend: String,
        provider: String,
        key_id: String,
    },
    UnsupportedKeyReference {
        backend: String,
        key_id: String,
    },
}

impl std::fmt::Display for SignerBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::InvalidNonce => write!(f, "nonce must be positive"),
            Self::MalformedSecureKeyReference { key_id } => {
                write!(f, "malformed secure key reference: {key_id}")
            }
            Self::ProviderUnavailable { backend } => {
                write!(f, "signer backend unavailable: {backend}")
            }
            Self::SecureProviderBackendMismatch {
                expected_backend,
                provided_backend,
                key_id,
            } => write!(
                f,
                "secure provider backend mismatch for key {key_id}; expected {expected_backend}, found {provided_backend}"
            ),
            Self::SignatureMismatch {
                backend,
                expected,
                found,
            } => write!(
                f,
                "signature mismatch for backend {backend}; expected {expected}, found {found}"
            ),
            Self::UnknownBackend(backend) => write!(f, "unknown signer backend: {backend}"),
            Self::UnsupportedSecureProvider {
                backend,
                provider,
                key_id,
            } => write!(
                f,
                "unsupported secure signer provider for backend {backend}: {provider} ({key_id})"
            ),
            Self::UnsupportedKeyReference { backend, key_id } => {
                write!(
                    f,
                    "unsupported key reference for backend {backend}: {key_id}"
                )
            }
        }
    }
}

impl std::error::Error for SignerBackendError {}

#[cfg(test)]
mod tests {
    use super::{SecureSignerProvider, SignerBackendError, SigningRequest};

    #[test]
    fn signing_request_rejects_invalid_fields() {
        assert_eq!(
            SigningRequest::new("", "agent-a", 1, "payload", "state:genesis"),
            Err(SignerBackendError::EmptyField("key_id"))
        );
        assert_eq!(
            SigningRequest::new("secure:key-1", "agent-a", 0, "payload", "state:genesis"),
            Err(SignerBackendError::InvalidNonce)
        );
    }

    #[test]
    fn secure_provider_parser_accepts_legacy_and_explicit_key_formats() {
        assert_eq!(
            SecureSignerProvider::from_key_id("secure:key-legacy-1"),
            Ok(SecureSignerProvider::Mock)
        );
        assert_eq!(
            SecureSignerProvider::from_key_id("secure:mock:key-legacy-2"),
            Ok(SecureSignerProvider::Mock)
        );
        assert_eq!(
            SecureSignerProvider::from_key_id("secure:aws-kms:key-prod-1"),
            Ok(SecureSignerProvider::AwsKmsEmulator)
        );
    }

    #[test]
    fn secure_provider_parser_rejects_unknown_and_malformed_key_references() {
        assert_eq!(
            SecureSignerProvider::from_key_id("secure:gcp-kms:key-prod-1"),
            Err(SignerBackendError::UnsupportedSecureProvider {
                backend: "secure-mock".to_owned(),
                provider: "gcp-kms".to_owned(),
                key_id: "secure:gcp-kms:key-prod-1".to_owned(),
            })
        );
        assert_eq!(
            SecureSignerProvider::from_key_id("secure:"),
            Err(SignerBackendError::MalformedSecureKeyReference {
                key_id: "secure:".to_owned(),
            })
        );
    }
}
