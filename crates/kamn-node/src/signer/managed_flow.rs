use super::managed_backend::{
    resolve_kolme_live_managed_signer_required_marker,
    resolve_required_kolme_live_managed_signer_command,
    resolve_required_managed_signer_public_key_hex, ManagedExternalKeySourceAdapter,
};
use super::models::{
    KolmeLiveManagedKeySourceAdapter, KolmeLiveManagedKeySourceAdapterOutput,
    KolmeLiveManagedKeySourceProvenanceMarker, KolmeLiveSignerSecretProvider,
    KolmeLiveSignerSelection,
};
use super::secret_provider::{
    ensure_kolme_live_managed_external_private_key_env_unset, EnvKolmeLiveSignerSecretProvider,
};
use crate::signer::{
    evaluate_kolme_live_signer_preflight_readiness,
    read_required_kolme_live_key_reference_from_env, resolve_kolme_live_nonce,
};
use crate::wire_payload::render_kolme_live_native_direct_message;
use kamn_core::ConfigError;
use kamn_core::{
    KolmeApiBroadcastRequest, KolmeRuntimeCommitHttpTransport, KolmeRuntimeCommitRequest,
    SignerProviderHandshakeMatrix,
};

pub(crate) fn enforce_kolme_live_managed_key_source_provenance_marker_parity(
    signer_selection: &KolmeLiveSignerSelection,
    marker: &KolmeLiveManagedKeySourceProvenanceMarker,
) -> Result<(), ConfigError> {
    if marker.profile != signer_selection.profile {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "managed key-source provenance marker profile {} does not match resolved signer profile {} (managed_signer_provenance_marker_profile_mismatch)",
            marker.profile, signer_selection.profile
        )));
    }
    if marker.key_source != signer_selection.key_source {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "managed key-source provenance marker key source {} does not match resolved signer key source {} (managed_signer_provenance_marker_key_source_mismatch)",
            marker.key_source, signer_selection.key_source
        )));
    }
    if marker.key_reference_env != signer_selection.key_reference_env {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "managed key-source provenance marker key-reference env {} does not match resolved signer key-reference env {} (managed_signer_provenance_marker_key_reference_env_mismatch)",
            marker.key_reference_env, signer_selection.key_reference_env
        )));
    }
    if marker.signer_public_key_hex.trim().is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(
            "managed key-source provenance marker signer_public_key_hex must not be empty (managed_signer_provenance_marker_public_key_missing)"
                .to_owned(),
        ));
    }
    Ok(())
}

pub(crate) fn build_kolme_live_direct_signed_wire_payload_with_managed_key_source_adapter<
    A: KolmeLiveManagedKeySourceAdapter,
>(
    base_url: &str,
    transport: &mut KolmeRuntimeCommitHttpTransport,
    request: &KolmeRuntimeCommitRequest,
    signer_selection: &KolmeLiveSignerSelection,
    managed_key_source_adapter: &A,
) -> Result<String, ConfigError> {
    let provider = EnvKolmeLiveSignerSecretProvider;
    provider.ensure_no_fallback_private_key_path()?;
    ensure_kolme_live_managed_external_private_key_env_unset(signer_selection)?;
    let _managed_signer_required_marker = resolve_kolme_live_managed_signer_required_marker()?;
    let _managed_signer_command = resolve_required_kolme_live_managed_signer_command()?;
    let signer_public_key_hex = resolve_required_managed_signer_public_key_hex(signer_selection)?;
    let key_reference = read_required_kolme_live_key_reference_from_env(signer_selection)?;
    let nonce = resolve_kolme_live_nonce(base_url, transport, signer_public_key_hex.as_str())?;
    let canonical_message =
        render_kolme_live_native_direct_message(request, signer_public_key_hex.as_str(), nonce)?;
    let managed_output: KolmeLiveManagedKeySourceAdapterOutput = managed_key_source_adapter
        .sign_message(
            signer_selection,
            key_reference.as_str(),
            request,
            nonce,
            canonical_message.as_str(),
            signer_public_key_hex.as_str(),
        )?;
    enforce_kolme_live_managed_key_source_provenance_marker_parity(
        signer_selection,
        &managed_output.provenance_marker,
    )?;
    let request = KolmeApiBroadcastRequest::new(
        canonical_message.as_str(),
        managed_output.signature_hex.as_str(),
        managed_output.recovery_id,
    )
    .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
    Ok(request.to_json_payload())
}

pub(crate) fn default_managed_key_source_adapter() -> ManagedExternalKeySourceAdapter {
    ManagedExternalKeySourceAdapter::with_provider_handshake_matrix(
        SignerProviderHandshakeMatrix::with_uniform_availability(true),
    )
}

pub(crate) fn validate_preflight(
    signer_selection: &KolmeLiveSignerSelection,
) -> Result<(), ConfigError> {
    let _signer_preflight = evaluate_kolme_live_signer_preflight_readiness(signer_selection)?;
    Ok(())
}
