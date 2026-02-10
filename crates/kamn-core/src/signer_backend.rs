use crate::signature_profile::{
    baseline_signature_for_fields, signature_matches_supported_profile_for_fields,
};
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
        Ok(CanonicalSecureKeyReference::parse(key_id)?.provider)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignerProviderHandshakeStatus {
    Available,
    Unavailable,
    PolicyBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignerProviderHandshakeMatrix {
    mock_status: SignerProviderHandshakeStatus,
    aws_kms_status: SignerProviderHandshakeStatus,
}

impl SignerProviderHandshakeMatrix {
    pub fn with_uniform_availability(available: bool) -> Self {
        let status = if available {
            SignerProviderHandshakeStatus::Available
        } else {
            SignerProviderHandshakeStatus::Unavailable
        };
        Self::with_statuses(status, status)
    }

    pub fn with_statuses(
        mock_status: SignerProviderHandshakeStatus,
        aws_kms_status: SignerProviderHandshakeStatus,
    ) -> Self {
        Self {
            mock_status,
            aws_kms_status,
        }
    }

    fn status_for_provider(&self, provider: SecureSignerProvider) -> SignerProviderHandshakeStatus {
        match provider {
            SecureSignerProvider::Mock => self.mock_status,
            SecureSignerProvider::AwsKmsEmulator => self.aws_kms_status,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignerKeyRole {
    Operator,
    Admin,
    Treasury,
    Auditor,
}

impl SignerKeyRole {
    pub fn from_key_id(key_id: &str) -> Result<Self, SignerBackendError> {
        Ok(CanonicalSecureKeyReference::parse(key_id)?.key_role)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Operator => "operator",
            Self::Admin => "admin",
            Self::Treasury => "treasury",
            Self::Auditor => "auditor",
        }
    }

    fn allows_secure_fallback(self) -> bool {
        matches!(self, Self::Operator)
    }

    fn from_sender(sender: &str) -> Self {
        let normalized_sender = sender.trim().to_ascii_lowercase();
        if normalized_sender.starts_with("admin-") || normalized_sender.starts_with("admin:") {
            return Self::Admin;
        }
        if normalized_sender.starts_with("treasury-") || normalized_sender.starts_with("treasury:")
        {
            return Self::Treasury;
        }
        if normalized_sender.starts_with("auditor-")
            || normalized_sender.starts_with("auditor:")
            || normalized_sender.starts_with("audit-")
            || normalized_sender.starts_with("audit:")
        {
            return Self::Auditor;
        }

        Self::Operator
    }

    fn from_provider_key_id(
        provider_key_id: &str,
        key_id: &str,
    ) -> Result<Self, SignerBackendError> {
        let Some(role_suffix) = provider_key_id.strip_prefix("role-") else {
            return Ok(Self::Operator);
        };

        let Some((role_label, role_key_id)) = role_suffix.split_once('/') else {
            return Err(SignerBackendError::MalformedSecureKeyReference {
                key_id: key_id.to_owned(),
            });
        };
        if role_label.trim().is_empty() || role_key_id.trim().is_empty() {
            return Err(SignerBackendError::MalformedSecureKeyReference {
                key_id: key_id.to_owned(),
            });
        }

        let normalized_role_label = role_label.trim().to_ascii_lowercase();
        match normalized_role_label.as_str() {
            "operator" => Ok(Self::Operator),
            "admin" => Ok(Self::Admin),
            "treasury" => Ok(Self::Treasury),
            "auditor" => Ok(Self::Auditor),
            _ => Err(SignerBackendError::UnsupportedSignerKeyRole {
                role: normalized_role_label,
                key_id: key_id.to_owned(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalSecureKeyReference {
    pub provider: SecureSignerProvider,
    pub key_role: SignerKeyRole,
    pub provider_key_id: String,
}

impl CanonicalSecureKeyReference {
    pub fn parse(key_id: &str) -> Result<Self, SignerBackendError> {
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
            let key_role = SignerKeyRole::from_provider_key_id(provider_key_id, key_id)?;
            return Ok(Self {
                provider,
                key_role,
                provider_key_id: provider_key_id.to_owned(),
            });
        }

        Ok(Self {
            provider: SecureSignerProvider::Mock,
            key_role: SignerKeyRole::Operator,
            provider_key_id: suffix.to_owned(),
        })
    }
}

pub trait SignerBackend {
    fn backend_name(&self) -> &'static str;
    fn sign(&self, request: &SigningRequest) -> Result<String, SignerBackendError>;
    fn verify(&self, request: &SigningRequest, signature: &str) -> Result<(), SignerBackendError>;
}

pub trait SecureSignerProviderClient {
    fn sign_with_provider(
        &self,
        request: &SigningRequest,
        key_reference: &CanonicalSecureKeyReference,
    ) -> Result<BackendSignature, SignerBackendError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeterministicSecureSignerProviderClient;

impl SecureSignerProviderClient for DeterministicSecureSignerProviderClient {
    fn sign_with_provider(
        &self,
        request: &SigningRequest,
        key_reference: &CanonicalSecureKeyReference,
    ) -> Result<BackendSignature, SignerBackendError> {
        Ok(BackendSignature {
            backend: key_reference.provider.backend_name().to_owned(),
            signature: request.expected_signature(),
        })
    }
}

pub type SecureSignerProviderClientSignFn = fn(
    request: &SigningRequest,
    key_reference: &CanonicalSecureKeyReference,
) -> Result<BackendSignature, SignerBackendError>;

fn deterministic_secure_provider_client_sign(
    request: &SigningRequest,
    key_reference: &CanonicalSecureKeyReference,
) -> Result<BackendSignature, SignerBackendError> {
    DeterministicSecureSignerProviderClient.sign_with_provider(request, key_reference)
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
        if !signature_matches_supported_profile_for_fields(
            signature,
            &request.sender,
            request.nonce,
            &request.state_hash,
            &request.payload,
        ) {
            return Err(SignerBackendError::SignatureMismatch {
                backend: self.backend_name().to_owned(),
                expected,
                found: signature.to_owned(),
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SecureSignerBackend {
    provider_handshake_matrix: SignerProviderHandshakeMatrix,
    provider_client_sign: SecureSignerProviderClientSignFn,
}

impl SecureSignerBackend {
    pub fn new(available: bool) -> Self {
        Self::with_provider_handshake_matrix(
            SignerProviderHandshakeMatrix::with_uniform_availability(available),
        )
    }

    pub fn with_provider_handshake_matrix(
        provider_handshake_matrix: SignerProviderHandshakeMatrix,
    ) -> Self {
        Self::with_provider_client(
            provider_handshake_matrix,
            deterministic_secure_provider_client_sign,
        )
    }

    pub fn with_provider_client(
        provider_handshake_matrix: SignerProviderHandshakeMatrix,
        provider_client_sign: SecureSignerProviderClientSignFn,
    ) -> Self {
        Self {
            provider_handshake_matrix,
            provider_client_sign,
        }
    }

    fn enforce_key_role_segregation(
        &self,
        request: &SigningRequest,
        secure_key: &CanonicalSecureKeyReference,
    ) -> Result<(), SignerBackendError> {
        let sender_role = SignerKeyRole::from_sender(&request.sender);
        if sender_role != secure_key.key_role {
            return Err(SignerBackendError::KeyRoleMismatch {
                key_role: secure_key.key_role.label().to_owned(),
                sender_role: sender_role.label().to_owned(),
                sender: request.sender.clone(),
                key_id: request.key_id.clone(),
            });
        }

        Ok(())
    }

    fn enforce_provider_handshake(
        &self,
        provider: SecureSignerProvider,
    ) -> Result<(), SignerBackendError> {
        let backend = provider.backend_name().to_owned();
        match self.provider_handshake_matrix.status_for_provider(provider) {
            SignerProviderHandshakeStatus::Available => Ok(()),
            SignerProviderHandshakeStatus::Unavailable => {
                Err(SignerBackendError::ProviderUnavailable { backend })
            }
            SignerProviderHandshakeStatus::PolicyBlocked => {
                Err(SignerBackendError::ProviderHandshakeRejected {
                    backend,
                    failure_class: "policy-blocked".to_owned(),
                })
            }
        }
    }

    fn sign_with_backend(
        &self,
        request: &SigningRequest,
    ) -> Result<BackendSignature, SignerBackendError> {
        let secure_key = CanonicalSecureKeyReference::parse(&request.key_id)?;
        self.enforce_key_role_segregation(request, &secure_key)?;
        self.enforce_provider_handshake(secure_key.provider)?;

        let signed = (self.provider_client_sign)(request, &secure_key)?;
        let expected_backend = secure_key.provider.backend_name().to_owned();
        if signed.backend != expected_backend {
            return Err(SignerBackendError::ProviderClientBackendMismatch {
                expected_backend,
                provided_backend: signed.backend,
                key_id: request.key_id.clone(),
            });
        }

        Ok(signed)
    }

    fn verify_with_backend_name(
        &self,
        backend: &str,
        request: &SigningRequest,
        signature: &str,
    ) -> Result<(), SignerBackendError> {
        let secure_key = CanonicalSecureKeyReference::parse(&request.key_id)?;
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
        let secure_key = CanonicalSecureKeyReference::parse(&request.key_id)?;
        self.enforce_key_role_segregation(request, &secure_key)?;
        self.enforce_provider_handshake(secure_key.provider)?;

        let expected = request.expected_signature();
        if !signature_matches_supported_profile_for_fields(
            signature,
            &request.sender,
            request.nonce,
            &request.state_hash,
            &request.payload,
        ) {
            return Err(SignerBackendError::SignatureMismatch {
                backend: secure_key.provider.backend_name().to_owned(),
                expected,
                found: signature.to_owned(),
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SignerBackendRouter {
    local: LocalSignerBackend,
    secure: SecureSignerBackend,
}

impl SignerBackendRouter {
    pub fn with_secure_availability(secure_available: bool) -> Self {
        Self::with_provider_handshake_matrix(
            SignerProviderHandshakeMatrix::with_uniform_availability(secure_available),
        )
    }

    pub fn with_provider_handshake_matrix(
        provider_handshake_matrix: SignerProviderHandshakeMatrix,
    ) -> Self {
        Self::with_provider_client(
            provider_handshake_matrix,
            deterministic_secure_provider_client_sign,
        )
    }

    pub fn with_provider_client(
        provider_handshake_matrix: SignerProviderHandshakeMatrix,
        provider_client_sign: SecureSignerProviderClientSignFn,
    ) -> Self {
        Self {
            local: LocalSignerBackend,
            secure: SecureSignerBackend::with_provider_client(
                provider_handshake_matrix,
                provider_client_sign,
            ),
        }
    }

    pub fn sign_with_secure_fallback(
        &self,
        request: &SigningRequest,
    ) -> Result<BackendSignature, SignerBackendError> {
        match self.secure.sign_with_backend(request) {
            Ok(signature) => Ok(signature),
            Err(SignerBackendError::ProviderUnavailable { .. }) => {
                let key_role = SignerKeyRole::from_key_id(&request.key_id)?;
                if !key_role.allows_secure_fallback() {
                    return Err(SignerBackendError::FallbackDeniedByRolePolicy {
                        key_role: key_role.label().to_owned(),
                        key_id: request.key_id.clone(),
                    });
                }
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
    FallbackDeniedByRolePolicy {
        key_role: String,
        key_id: String,
    },
    InvalidNonce,
    KeyRoleMismatch {
        key_role: String,
        sender_role: String,
        sender: String,
        key_id: String,
    },
    MalformedSecureKeyReference {
        key_id: String,
    },
    ProviderHandshakeRejected {
        backend: String,
        failure_class: String,
    },
    ProviderUnavailable {
        backend: String,
    },
    ProviderClientBackendMismatch {
        expected_backend: String,
        provided_backend: String,
        key_id: String,
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
    UnsupportedSignerKeyRole {
        role: String,
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
            Self::FallbackDeniedByRolePolicy { key_role, key_id } => write!(
                f,
                "secure fallback denied for key role {key_role} ({key_id})"
            ),
            Self::InvalidNonce => write!(f, "nonce must be positive"),
            Self::KeyRoleMismatch {
                key_role,
                sender_role,
                sender,
                key_id,
            } => write!(
                f,
                "key role mismatch for {key_id}; key role {key_role}, sender role {sender_role}, sender {sender}"
            ),
            Self::MalformedSecureKeyReference { key_id } => {
                write!(f, "malformed secure key reference: {key_id}")
            }
            Self::ProviderHandshakeRejected {
                backend,
                failure_class,
            } => write!(
                f,
                "signer provider handshake rejected for backend {backend}: {failure_class}"
            ),
            Self::ProviderUnavailable { backend } => {
                write!(f, "signer backend unavailable: {backend}")
            }
            Self::ProviderClientBackendMismatch {
                expected_backend,
                provided_backend,
                key_id,
            } => write!(
                f,
                "provider client backend mismatch for key {key_id}; expected {expected_backend}, found {provided_backend}"
            ),
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
            Self::UnsupportedSignerKeyRole { role, key_id } => {
                write!(f, "unsupported signer key role {role} for key reference {key_id}")
            }
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
    use super::{
        deterministic_secure_provider_client_sign, CanonicalSecureKeyReference,
        SecureSignerBackend, SecureSignerProvider, SignerBackend, SignerBackendError,
        SignerBackendRouter, SignerKeyRole, SignerProviderHandshakeMatrix,
        SignerProviderHandshakeStatus, SigningRequest,
    };

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

    #[test]
    fn signer_key_role_parser_supports_role_prefixes_and_legacy_defaults() {
        assert_eq!(
            SignerKeyRole::from_key_id("secure:key-legacy-1"),
            Ok(SignerKeyRole::Operator)
        );
        assert_eq!(
            SignerKeyRole::from_key_id("secure:aws-kms:role-admin/key-prod-1"),
            Ok(SignerKeyRole::Admin)
        );
        assert_eq!(
            SignerKeyRole::from_key_id("secure:aws-kms:role-treasury/key-prod-2"),
            Ok(SignerKeyRole::Treasury)
        );
    }

    #[test]
    fn signer_key_role_parser_rejects_unsupported_role_labels() {
        assert_eq!(
            SignerKeyRole::from_key_id("secure:aws-kms:role-root/key-prod-3"),
            Err(SignerBackendError::UnsupportedSignerKeyRole {
                role: "root".to_owned(),
                key_id: "secure:aws-kms:role-root/key-prod-3".to_owned(),
            })
        );
    }

    #[test]
    fn canonical_secure_key_reference_parser_preserves_provider_key_scope() {
        let parsed = CanonicalSecureKeyReference::parse("secure:AWS-KMS:role-TREASURY/key-prod-9")
            .expect("canonical parser should accept provider + role scoped keys");
        assert_eq!(parsed.provider, SecureSignerProvider::AwsKmsEmulator);
        assert_eq!(parsed.key_role, SignerKeyRole::Treasury);
        assert_eq!(parsed.provider_key_id, "role-TREASURY/key-prod-9");
    }

    #[test]
    fn handshake_matrix_maps_provider_statuses() {
        let matrix = SignerProviderHandshakeMatrix::with_statuses(
            SignerProviderHandshakeStatus::Available,
            SignerProviderHandshakeStatus::PolicyBlocked,
        );
        assert_eq!(
            matrix.status_for_provider(SecureSignerProvider::Mock),
            SignerProviderHandshakeStatus::Available
        );
        assert_eq!(
            matrix.status_for_provider(SecureSignerProvider::AwsKmsEmulator),
            SignerProviderHandshakeStatus::PolicyBlocked
        );
    }

    #[test]
    fn router_decision_matrix_distinguishes_unavailable_vs_policy_blocked_handshakes() {
        let request = SigningRequest::new(
            "secure:aws-kms:key-ops-1",
            "agent-a",
            1,
            "payload-1",
            "state:genesis",
        )
        .expect("request should be valid");

        let unavailable_router = SignerBackendRouter::with_provider_handshake_matrix(
            SignerProviderHandshakeMatrix::with_statuses(
                SignerProviderHandshakeStatus::Available,
                SignerProviderHandshakeStatus::Unavailable,
            ),
        );
        let signed = unavailable_router
            .sign_with_secure_fallback(&request)
            .expect("unavailable provider should allow operator fallback");
        assert_eq!(signed.backend, "local-software");

        let policy_blocked_router = SignerBackendRouter::with_provider_handshake_matrix(
            SignerProviderHandshakeMatrix::with_statuses(
                SignerProviderHandshakeStatus::Available,
                SignerProviderHandshakeStatus::PolicyBlocked,
            ),
        );
        assert_eq!(
            policy_blocked_router.sign_with_secure_fallback(&request),
            Err(SignerBackendError::ProviderHandshakeRejected {
                backend: "secure-aws-kms-emulator".to_owned(),
                failure_class: "policy-blocked".to_owned(),
            })
        );
    }

    #[test]
    fn secure_backend_rejects_policy_blocked_provider_handshake() {
        let backend = SecureSignerBackend::with_provider_handshake_matrix(
            SignerProviderHandshakeMatrix::with_statuses(
                SignerProviderHandshakeStatus::Available,
                SignerProviderHandshakeStatus::PolicyBlocked,
            ),
        );
        let request = SigningRequest::new(
            "secure:aws-kms:key-prod-1",
            "agent-a",
            1,
            "payload-1",
            "state:genesis",
        )
        .expect("request should be valid");

        assert_eq!(
            backend.sign(&request),
            Err(SignerBackendError::ProviderHandshakeRejected {
                backend: "secure-aws-kms-emulator".to_owned(),
                failure_class: "policy-blocked".to_owned(),
            })
        );
    }

    #[test]
    fn provider_client_maps_backend_from_canonical_reference() {
        let request = SigningRequest::new(
            "secure:aws-kms:key-prod-1",
            "agent-a",
            1,
            "payload-1",
            "state:genesis",
        )
        .expect("request should be valid");
        let key_reference = CanonicalSecureKeyReference::parse(&request.key_id)
            .expect("canonical parser should parse secure provider key");

        let signed = deterministic_secure_provider_client_sign(&request, &key_reference)
            .expect("deterministic provider client should sign");
        assert_eq!(signed.backend, "secure-aws-kms-emulator");
    }

    #[test]
    fn secure_backend_rejects_provider_client_backend_mismatch() {
        fn mismatched_provider_client(
            request: &SigningRequest,
            _key_reference: &CanonicalSecureKeyReference,
        ) -> Result<super::BackendSignature, SignerBackendError> {
            Ok(super::BackendSignature {
                backend: "secure-mock".to_owned(),
                signature: request.expected_signature(),
            })
        }

        let backend = SecureSignerBackend::with_provider_client(
            SignerProviderHandshakeMatrix::with_uniform_availability(true),
            mismatched_provider_client,
        );
        let request = SigningRequest::new(
            "secure:aws-kms:key-prod-1",
            "agent-a",
            1,
            "payload-1",
            "state:genesis",
        )
        .expect("request should be valid");

        assert_eq!(
            backend.sign(&request),
            Err(SignerBackendError::ProviderClientBackendMismatch {
                expected_backend: "secure-aws-kms-emulator".to_owned(),
                provided_backend: "secure-mock".to_owned(),
                key_id: "secure:aws-kms:key-prod-1".to_owned(),
            })
        );
    }
}
