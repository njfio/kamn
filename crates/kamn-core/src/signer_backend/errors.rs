use super::env::SIGNER_PRIVATE_KEY_ENV;
use crate::SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV;

mod display;

/// Errors emitted by signer backend request validation, routing, and provider policy checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignerBackendError {
    /// Required string field was empty or whitespace.
    EmptyField(&'static str),
    /// Fallback to local backend is denied for this signer key role.
    FallbackDeniedByRolePolicy {
        /// Parsed signer key role label.
        key_role: String,
        /// Key reference that triggered fallback-denied policy.
        key_id: String,
    },
    /// Nonce must be positive.
    InvalidNonce,
    /// Signer key material is required but missing from the environment.
    MissingSigningKeyMaterial {
        /// Key reference for which key material was requested.
        key_id: String,
        /// Key-specific environment variable name derived from key_id.
        key_specific_env: String,
    },
    /// Signer key material was present but malformed.
    InvalidSigningKeyMaterial {
        /// Key reference for which key material was invalid.
        key_id: String,
    },
    /// Sender-derived role does not match role encoded in secure key reference.
    KeyRoleMismatch {
        /// Role encoded by key reference.
        key_role: String,
        /// Role inferred from sender naming policy.
        sender_role: String,
        /// Sender identifier provided by request.
        sender: String,
        /// Key reference used by request.
        key_id: String,
    },
    /// Secure key reference failed canonical parse/validation.
    MalformedSecureKeyReference {
        /// Invalid key reference input.
        key_id: String,
    },
    /// Provider handshake was rejected by policy classification.
    ProviderHandshakeRejected {
        /// Backend that rejected handshake.
        backend: String,
        /// Failure class returned by policy gate.
        failure_class: String,
    },
    /// Provider backend is unavailable.
    ProviderUnavailable {
        /// Backend that is unavailable.
        backend: String,
    },
    /// Provider client returned a backend label that does not match parsed key provider.
    ProviderClientBackendMismatch {
        /// Backend expected from key reference/provider mapping.
        expected_backend: String,
        /// Backend returned by provider client callback.
        provided_backend: String,
        /// Key reference being processed.
        key_id: String,
    },
    /// Verification attempted with backend name that does not match secure key provider.
    SecureProviderBackendMismatch {
        /// Backend expected from key reference/provider mapping.
        expected_backend: String,
        /// Backend provided by caller.
        provided_backend: String,
        /// Key reference being processed.
        key_id: String,
    },
    /// Signature verification failed.
    SignatureMismatch {
        /// Backend used for verification.
        backend: String,
        /// Expected deterministic signature.
        expected: String,
        /// Signature provided by caller/provider.
        found: String,
    },
    /// Caller requested unknown backend identifier.
    UnknownBackend(String),
    /// Secure-provider label is unsupported.
    UnsupportedSecureProvider {
        /// Backend family where provider label was parsed.
        backend: String,
        /// Unsupported provider label.
        provider: String,
        /// Key reference being processed.
        key_id: String,
    },
    /// Signer role encoded in key reference is unsupported.
    UnsupportedSignerKeyRole {
        /// Unsupported role label.
        role: String,
        /// Key reference being processed.
        key_id: String,
    },
    /// Key reference prefix/backend is unsupported by signer backend contracts.
    UnsupportedKeyReference {
        /// Backend family expected by parser.
        backend: String,
        /// Key reference input.
        key_id: String,
    },
}

impl std::error::Error for SignerBackendError {}

pub(super) fn missing_key_material_message(key_id: &str, key_specific_env: &str) -> String {
    format!(
        "missing signer key material for {key_id}; set {key_specific_env} or {SIGNER_PRIVATE_KEY_ENV} or {SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV}"
    )
}
