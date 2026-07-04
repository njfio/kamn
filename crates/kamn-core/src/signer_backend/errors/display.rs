use super::{missing_key_material_message, SignerBackendError};

impl std::fmt::Display for SignerBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", error_message(self))
    }
}

fn error_message(error: &SignerBackendError) -> String {
    basic_message(error)
        .or_else(|| key_policy_message(error))
        .or_else(|| provider_message(error))
        .or_else(|| backend_message(error))
        .unwrap_or_else(|| unmapped_error_message(error))
}

fn basic_message(error: &SignerBackendError) -> Option<String> {
    match error {
        SignerBackendError::EmptyField(field) => Some(format!("{field} must not be empty")),
        SignerBackendError::InvalidNonce => Some("nonce must be positive".to_owned()),
        SignerBackendError::MissingSigningKeyMaterial {
            key_id,
            key_specific_env,
        } => Some(missing_key_material_message(key_id, key_specific_env)),
        SignerBackendError::InvalidSigningKeyMaterial { key_id } => {
            Some(invalid_signing_key_material_message(key_id))
        }
        _ => None,
    }
}

fn key_policy_message(error: &SignerBackendError) -> Option<String> {
    match error {
        SignerBackendError::FallbackDeniedByRolePolicy { key_role, key_id } => {
            Some(fallback_denied_message(key_role, key_id))
        }
        SignerBackendError::KeyRoleMismatch {
            key_role,
            sender_role,
            sender,
            key_id,
        } => Some(key_role_mismatch_message(
            key_role,
            sender_role,
            sender,
            key_id,
        )),
        SignerBackendError::MalformedSecureKeyReference { key_id } => {
            Some(malformed_key_message(key_id))
        }
        SignerBackendError::UnsupportedSignerKeyRole { role, key_id } => {
            Some(unsupported_key_role_message(role, key_id))
        }
        _ => None,
    }
}

fn provider_message(error: &SignerBackendError) -> Option<String> {
    match error {
        SignerBackendError::ProviderHandshakeRejected {
            backend,
            failure_class,
        } => Some(provider_handshake_message(backend, failure_class)),
        SignerBackendError::ProviderUnavailable { backend } => {
            Some(provider_unavailable_message(backend))
        }
        SignerBackendError::UnsupportedSecureProvider {
            backend,
            provider,
            key_id,
        } => Some(unsupported_provider_message(backend, provider, key_id)),
        _ => None,
    }
}

fn backend_message(error: &SignerBackendError) -> Option<String> {
    match error {
        SignerBackendError::ProviderClientBackendMismatch {
            expected_backend,
            provided_backend,
            key_id,
        } => Some(backend_mismatch_message(
            "provider client backend mismatch",
            expected_backend,
            provided_backend,
            key_id,
        )),
        SignerBackendError::SecureProviderBackendMismatch {
            expected_backend,
            provided_backend,
            key_id,
        } => Some(backend_mismatch_message(
            "secure provider backend mismatch",
            expected_backend,
            provided_backend,
            key_id,
        )),
        SignerBackendError::SignatureMismatch {
            backend,
            expected,
            found,
        } => Some(signature_mismatch_message(backend, expected, found)),
        SignerBackendError::UnknownBackend(backend) => Some(unknown_backend_message(backend)),
        SignerBackendError::UnsupportedKeyReference { backend, key_id } => {
            Some(unsupported_key_reference_message(backend, key_id))
        }
        _ => None,
    }
}

fn invalid_signing_key_material_message(key_id: &str) -> String {
    format!("invalid signer key material for key reference {key_id}")
}

fn unmapped_error_message(error: &SignerBackendError) -> String {
    format!("signer backend error: {error:?}")
}

fn fallback_denied_message(key_role: &str, key_id: &str) -> String {
    format!("secure fallback denied for key role {key_role} ({key_id})")
}

fn malformed_key_message(key_id: &str) -> String {
    format!("malformed secure key reference: {key_id}")
}

fn provider_handshake_message(backend: &str, failure_class: &str) -> String {
    format!("signer provider handshake rejected for backend {backend}: {failure_class}")
}

fn provider_unavailable_message(backend: &str) -> String {
    format!("signer backend unavailable: {backend}")
}

fn signature_mismatch_message(backend: &str, expected: &str, found: &str) -> String {
    format!("signature mismatch for backend {backend}; expected {expected}, found {found}")
}

fn unknown_backend_message(backend: &str) -> String {
    format!("unknown signer backend: {backend}")
}

fn unsupported_provider_message(backend: &str, provider: &str, key_id: &str) -> String {
    format!("unsupported secure signer provider for backend {backend}: {provider} ({key_id})")
}

fn unsupported_key_role_message(role: &str, key_id: &str) -> String {
    format!("unsupported signer key role {role} for key reference {key_id}")
}

fn unsupported_key_reference_message(backend: &str, key_id: &str) -> String {
    format!("unsupported key reference for backend {backend}: {key_id}")
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
    format!("{prefix} for key {key_id}; expected {expected_backend}, found {provided_backend}")
}
