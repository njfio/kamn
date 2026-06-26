use kamn_core::{
    ConfigError, KolmeRuntimeCommitRequest, SecureSignerBackend, SecureSignerProvider,
    SignerBackend, SignerBackendError, SignerProviderHandshakeMatrix, SigningRequest,
};

use super::super::{
    KolmeLiveManagedKeySourceAdapter, KolmeLiveManagedKeySourceAdapterOutput,
    KolmeLiveManagedKeySourceProvenanceMarker, KolmeLiveSignerSelection,
    KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL,
};
use super::command::resolve_required_kolme_live_managed_signer_command;
use super::execution::execute_kolme_live_managed_signer_backend_command;
use super::response::verify_kolme_live_managed_signer_backend_signature_provenance;

#[derive(Debug, Clone)]
pub(crate) struct ManagedExternalKeySourceAdapter {
    provider_handshake_matrix: SignerProviderHandshakeMatrix,
}

impl ManagedExternalKeySourceAdapter {
    pub(crate) fn with_provider_handshake_matrix(
        provider_handshake_matrix: SignerProviderHandshakeMatrix,
    ) -> Self {
        Self {
            provider_handshake_matrix,
        }
    }
}

fn map_kolme_live_secure_signer_backend_error(error: SignerBackendError) -> ConfigError {
    let (reason_code, message) = match &error {
        SignerBackendError::ProviderUnavailable { .. } => {
            ("managed_signer_provider_unavailable", error.to_string())
        }
        SignerBackendError::ProviderHandshakeRejected { .. } => (
            "managed_signer_provider_handshake_rejected",
            error.to_string(),
        ),
        SignerBackendError::FallbackDeniedByRolePolicy { .. } => (
            "managed_signer_fallback_denied_by_role_policy",
            error.to_string(),
        ),
        _ => ("managed_signer_backend_error", error.to_string()),
    };
    ConfigError::RuntimeKolmeLive(format!(
        "managed-external secure signer backend error: {message} ({reason_code})"
    ))
}

fn ensure_managed_external_key_source(
    signer_selection: &KolmeLiveSignerSelection,
) -> Result<(), ConfigError> {
    if signer_selection.key_source == KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL {
        return Ok(());
    }
    Err(ConfigError::RuntimeKolmeLive(format!(
        "managed key-source adapter requires --kolme-live-signer-key-source={} for signer profile {} (managed_signer_adapter_key_source_unsupported)",
        KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL,
        signer_selection.profile
    )))
}

fn build_provenance_marker(
    signer_selection: &KolmeLiveSignerSelection,
    signer_public_key_hex: &str,
) -> KolmeLiveManagedKeySourceProvenanceMarker {
    KolmeLiveManagedKeySourceProvenanceMarker {
        profile: signer_selection.profile,
        key_source: signer_selection.key_source,
        key_reference_env: signer_selection.key_reference_env,
        signer_public_key_hex: signer_public_key_hex.to_owned(),
    }
}

fn parse_secure_signer_provider(key_reference: &str) -> Result<SecureSignerProvider, ConfigError> {
    SecureSignerProvider::from_key_id(key_reference).map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer key reference parse failed before secure routing: {error} (managed_signer_key_reference_invalid)"
        ))
    })
}

fn build_signing_request(
    key_reference: &str,
    request: &KolmeRuntimeCommitRequest,
    nonce: u64,
    canonical_message: &str,
) -> Result<SigningRequest, ConfigError> {
    SigningRequest::new(
        key_reference,
        request.actor_did.as_str(),
        nonce,
        canonical_message,
        request.state_root.as_str(),
    )
    .map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer request validation failed: {error} (managed_signer_request_invalid)"
        ))
    })
}

fn run_secure_backend_handshake(
    provider_handshake_matrix: SignerProviderHandshakeMatrix,
    signing_request: &SigningRequest,
) -> Result<(), ConfigError> {
    let secure_backend =
        SecureSignerBackend::with_provider_handshake_matrix(provider_handshake_matrix);
    secure_backend
        .sign(signing_request)
        .map(|_| ())
        .map_err(map_kolme_live_secure_signer_backend_error)
}

fn execute_managed_backend_signature(
    key_reference: &str,
    signing_request: &SigningRequest,
    canonical_message: &str,
) -> Result<super::ManagedExternalBackendSignature, ConfigError> {
    let command = resolve_required_kolme_live_managed_signer_command()?;
    execute_kolme_live_managed_signer_backend_command(
        command.as_str(),
        key_reference,
        signing_request,
        canonical_message,
    )
}

fn finalize_backend_signature(
    canonical_message: &str,
    expected_signer_public_key_hex: &str,
    backend_signature: super::ManagedExternalBackendSignature,
) -> Result<(String, u8), ConfigError> {
    verify_kolme_live_managed_signer_backend_signature_provenance(
        canonical_message,
        expected_signer_public_key_hex,
        &backend_signature,
    )?;
    Ok((
        backend_signature.signature_hex,
        backend_signature.recovery_id,
    ))
}

impl KolmeLiveManagedKeySourceAdapter for ManagedExternalKeySourceAdapter {
    fn sign_message(
        &self,
        signer_selection: &KolmeLiveSignerSelection,
        key_reference: &str,
        request: &KolmeRuntimeCommitRequest,
        nonce: u64,
        canonical_message: &str,
        signer_public_key_hex: &str,
    ) -> Result<KolmeLiveManagedKeySourceAdapterOutput, ConfigError> {
        ensure_managed_external_key_source(signer_selection)?;
        let (signature_hex, recovery_id) = sign_kolme_live_managed_external_message(
            key_reference,
            request,
            nonce,
            canonical_message,
            self.provider_handshake_matrix.clone(),
            signer_public_key_hex,
        )?;
        Ok(KolmeLiveManagedKeySourceAdapterOutput {
            signature_hex,
            recovery_id,
            provenance_marker: build_provenance_marker(signer_selection, signer_public_key_hex),
        })
    }
}

pub(crate) fn sign_kolme_live_managed_external_message(
    key_reference: &str,
    request: &KolmeRuntimeCommitRequest,
    nonce: u64,
    canonical_message: &str,
    provider_handshake_matrix: SignerProviderHandshakeMatrix,
    expected_signer_public_key_hex: &str,
) -> Result<(String, u8), ConfigError> {
    let _provider = parse_secure_signer_provider(key_reference)?;
    let signing_request = build_signing_request(key_reference, request, nonce, canonical_message)?;
    run_secure_backend_handshake(provider_handshake_matrix, &signing_request)?;
    let backend_signature =
        execute_managed_backend_signature(key_reference, &signing_request, canonical_message)?;
    finalize_backend_signature(
        canonical_message,
        expected_signer_public_key_hex,
        backend_signature,
    )
}
