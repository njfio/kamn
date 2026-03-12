use super::{missing_key_material_message, SignerBackendError};

impl std::fmt::Display for SignerBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", error_message(self))
    }
}

fn error_message(error: &SignerBackendError) -> String {
    match error {
        SignerBackendError::EmptyField(field) => format!("{field} must not be empty"),
        SignerBackendError::FallbackDeniedByRolePolicy { key_role, key_id } => {
            format!("secure fallback denied for key role {key_role} ({key_id})")
        }
        SignerBackendError::InvalidNonce => "nonce must be positive".to_owned(),
        SignerBackendError::MissingSigningKeyMaterial {
            key_id,
            key_specific_env,
        } => missing_key_material_message(key_id, key_specific_env),
        SignerBackendError::InvalidSigningKeyMaterial { key_id } => {
            format!("invalid signer key material for key reference {key_id}")
        }
        SignerBackendError::KeyRoleMismatch {
            key_role,
            sender_role,
            sender,
            key_id,
        } => key_role_mismatch_message(key_role, sender_role, sender, key_id),
        SignerBackendError::MalformedSecureKeyReference { key_id } => {
            format!("malformed secure key reference: {key_id}")
        }
        SignerBackendError::ProviderHandshakeRejected {
            backend,
            failure_class,
        } => format!("signer provider handshake rejected for backend {backend}: {failure_class}"),
        SignerBackendError::ProviderUnavailable { backend } => {
            format!("signer backend unavailable: {backend}")
        }
        SignerBackendError::ProviderClientBackendMismatch {
            expected_backend,
            provided_backend,
            key_id,
        } => backend_mismatch_message(
            "provider client backend mismatch",
            expected_backend,
            provided_backend,
            key_id,
        ),
        SignerBackendError::SecureProviderBackendMismatch {
            expected_backend,
            provided_backend,
            key_id,
        } => backend_mismatch_message(
            "secure provider backend mismatch",
            expected_backend,
            provided_backend,
            key_id,
        ),
        SignerBackendError::SignatureMismatch {
            backend,
            expected,
            found,
        } => format!("signature mismatch for backend {backend}; expected {expected}, found {found}"),
        SignerBackendError::UnknownBackend(backend) => format!("unknown signer backend: {backend}"),
        SignerBackendError::UnsupportedSecureProvider {
            backend,
            provider,
            key_id,
        } => format!(
            "unsupported secure signer provider for backend {backend}: {provider} ({key_id})"
        ),
        SignerBackendError::UnsupportedSignerKeyRole { role, key_id } => {
            format!("unsupported signer key role {role} for key reference {key_id}")
        }
        SignerBackendError::UnsupportedKeyReference { backend, key_id } => {
            format!("unsupported key reference for backend {backend}: {key_id}")
        }
    }
}

fn key_role_mismatch_message(
    key_role: &str,
    sender_role: &str,
    sender: &str,
    key_id: &str,
) -> String {
    format!(
        "key role mismatch for {key_id}; key role {key_role}, sender role {sender_role}, sender {sender}"
    )
}

fn backend_mismatch_message(
    prefix: &str,
    expected_backend: &str,
    provided_backend: &str,
    key_id: &str,
) -> String {
    format!(
        "{prefix} for key {key_id}; expected {expected_backend}, found {provided_backend}"
    )
}
