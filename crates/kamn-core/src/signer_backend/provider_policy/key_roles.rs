use super::super::errors::SignerBackendError;

/// Security roles extracted from signer key references and sender naming policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignerKeyRole {
    /// Default operational role that can use secure fallback policy.
    Operator,
    /// Administrative role for privileged operations.
    Admin,
    /// Treasury role for settlement-related actions.
    Treasury,
    /// Audit role for read/evidence workflows.
    Auditor,
}

impl SignerKeyRole {
    /// Resolve signer key role from a secure key reference.
    pub fn from_key_id(key_id: &str) -> Result<Self, SignerBackendError> {
        Ok(super::CanonicalSecureKeyReference::parse(key_id)?.key_role)
    }

    /// Return canonical lowercase label for role-policy and diagnostics output.
    pub fn label(self) -> &'static str {
        match self {
            Self::Operator => "operator",
            Self::Admin => "admin",
            Self::Treasury => "treasury",
            Self::Auditor => "auditor",
        }
    }

    pub(crate) fn allows_secure_fallback(self) -> bool {
        matches!(self, Self::Operator)
    }

    pub(crate) fn from_sender(sender: &str) -> Self {
        let normalized_sender = sender.trim().to_ascii_lowercase();
        if normalized_sender.starts_with("admin-") || normalized_sender.starts_with("admin:") {
            return Self::Admin;
        }
        if normalized_sender.starts_with("treasury-")
            || normalized_sender.starts_with("treasury:")
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
}

pub(super) fn parse_provider_key_role(
    provider_key_id: &str,
    key_id: &str,
) -> Result<SignerKeyRole, SignerBackendError> {
    let Some(role_suffix) = provider_key_id.strip_prefix("role-") else {
        return Ok(SignerKeyRole::Operator);
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
    match role_label.trim().to_ascii_lowercase().as_str() {
        "operator" => Ok(SignerKeyRole::Operator),
        "admin" => Ok(SignerKeyRole::Admin),
        "treasury" => Ok(SignerKeyRole::Treasury),
        "auditor" => Ok(SignerKeyRole::Auditor),
        role => Err(SignerBackendError::UnsupportedSignerKeyRole {
            role: role.to_owned(),
            key_id: key_id.to_owned(),
        }),
    }
}
