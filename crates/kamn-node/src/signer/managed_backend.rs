use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};

mod adapter;
mod command;
mod execution;
mod key_material;
mod response;
#[cfg(test)]
mod tests;

#[cfg(windows)]
const MANAGED_SIGNER_CHILD_ENV_ALLOWLIST: &[&str] = &["PATH", "SYSTEMROOT", "WINDIR"];
#[cfg(not(windows))]
const MANAGED_SIGNER_CHILD_ENV_ALLOWLIST: &[&str] = &["PATH"];

#[derive(Debug, Clone, PartialEq, Eq)]
struct ManagedExternalBackendSignature {
    signature_hex: String,
    recovery_id: u8,
    signer_public_key_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ManagedSignerCommandSpec {
    executable: String,
    args: Vec<String>,
}

pub(crate) use adapter::ManagedExternalKeySourceAdapter;
#[cfg(test)]
pub(crate) use adapter::sign_kolme_live_managed_external_message;
pub(crate) use command::resolve_kolme_live_managed_signer_required_marker;
pub(super) use command::resolve_required_kolme_live_managed_signer_command;
pub(super) use key_material::resolve_required_managed_signer_public_key_hex;
