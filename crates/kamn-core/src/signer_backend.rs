use crate::transaction::BaselineTransaction;

const LOCAL_BACKEND_NAME: &str = "local-software";
const SECURE_BACKEND_NAME: &str = "secure-mock";

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
        format!(
            "sig:{}:{}:{}:{}",
            self.sender,
            self.nonce,
            self.state_hash,
            self.payload.len()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendSignature {
    pub backend: String,
    pub signature: String,
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
}

impl SignerBackend for SecureSignerBackend {
    fn backend_name(&self) -> &'static str {
        SECURE_BACKEND_NAME
    }

    fn sign(&self, request: &SigningRequest) -> Result<String, SignerBackendError> {
        if !self.available {
            return Err(SignerBackendError::ProviderUnavailable {
                backend: self.backend_name().to_owned(),
            });
        }
        if !request.key_id.starts_with("secure:") {
            return Err(SignerBackendError::UnsupportedKeyReference {
                backend: self.backend_name().to_owned(),
                key_id: request.key_id.clone(),
            });
        }

        Ok(request.expected_signature())
    }

    fn verify(&self, request: &SigningRequest, signature: &str) -> Result<(), SignerBackendError> {
        if !self.available {
            return Err(SignerBackendError::ProviderUnavailable {
                backend: self.backend_name().to_owned(),
            });
        }
        if !request.key_id.starts_with("secure:") {
            return Err(SignerBackendError::UnsupportedKeyReference {
                backend: self.backend_name().to_owned(),
                key_id: request.key_id.clone(),
            });
        }

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
        match self.secure.sign(request) {
            Ok(signature) => Ok(BackendSignature {
                backend: self.secure.backend_name().to_owned(),
                signature,
            }),
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
            SECURE_BACKEND_NAME => self.secure.verify(request, signature),
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
    ProviderUnavailable {
        backend: String,
    },
    SignatureMismatch {
        backend: String,
        expected: String,
        found: String,
    },
    UnknownBackend(String),
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
            Self::ProviderUnavailable { backend } => {
                write!(f, "signer backend unavailable: {backend}")
            }
            Self::SignatureMismatch {
                backend,
                expected,
                found,
            } => write!(
                f,
                "signature mismatch for backend {backend}; expected {expected}, found {found}"
            ),
            Self::UnknownBackend(backend) => write!(f, "unknown signer backend: {backend}"),
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
    use super::{SignerBackendError, SigningRequest};

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
}
