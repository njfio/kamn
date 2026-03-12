#[cfg(test)]
use std::time::{Duration, Instant};

#[cfg(test)]
use kamn_core::KolmeRuntimeCommitProviderError;
use kamn_core::ConfigError;

mod direct_payload;
mod managed_backend;
mod managed_flow;
mod models;
mod nonce;
mod secret_provider;
mod signer_adapter;
mod signer_policy;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) use managed_backend::sign_kolme_live_managed_external_message;
pub(crate) use managed_backend::resolve_kolme_live_managed_signer_required_marker;
pub(crate) use nonce::resolve_kolme_live_nonce;
#[cfg(test)]
use nonce::{classify_nonce_retry_category, deterministic_nonce_retry_backoff_millis};
#[cfg(test)]
pub(crate) use signer_adapter::{
    build_kolme_live_managed_signing_key, resolve_kolme_live_signer_private_key_env_name,
};
pub(crate) use signer_adapter::{decode_kolme_hex_bytes, encode_kolme_hex_lower};
pub(crate) use signer_policy::{
    evaluate_kolme_live_signer_preflight_readiness, normalize_kolme_live_signer_key_source,
    normalize_kolme_live_signer_profile_selector,
};
use signer_policy::{
    read_required_kolme_live_key_reference_from_env, resolve_kolme_live_signer_selection,
};

pub(crate) use models::{
    KolmeLiveManagedKeySourceAdapter, KolmeLiveManagedKeySourceAdapterOutput,
    KolmeLiveManagedKeySourceProvenanceMarker, KolmeLiveSignerPreflightReadiness,
    KolmeLiveSignerSelection,
};
pub(crate) use super::KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL;
pub(crate) use secret_provider::{
    ensure_kolme_live_managed_external_private_key_env_unset,
    ensure_kolme_live_strict_signer_secret_source_precedence,
};

pub(crate) type ManagedExternalKeySourceAdapter = managed_backend::ManagedExternalKeySourceAdapter;
pub(crate) type KolmeForkSecp256k1SignerAdapter = signer_adapter::KolmeForkSecp256k1SignerAdapter;

pub(crate) fn build_kolme_live_signer_adapter(
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
) -> Result<(KolmeForkSecp256k1SignerAdapter, KolmeLiveSignerSelection), ConfigError> {
    direct_payload::build_kolme_live_signer_adapter(
        strict_signer_profile,
        strict_signer_key_source,
    )
}

pub(crate) fn read_kolme_live_signer_private_key_hex(
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
) -> Result<(String, KolmeLiveSignerSelection), ConfigError> {
    direct_payload::read_kolme_live_signer_private_key_hex(
        strict_signer_profile,
        strict_signer_key_source,
    )
}

pub(crate) fn enforce_kolme_live_managed_key_source_provenance_marker_parity(
    signer_selection: &KolmeLiveSignerSelection,
    marker: &KolmeLiveManagedKeySourceProvenanceMarker,
) -> Result<(), ConfigError> {
    managed_flow::enforce_kolme_live_managed_key_source_provenance_marker_parity(
        signer_selection,
        marker,
    )
}

pub(crate) fn ensure_kolme_live_strict_signer_secret_source_precedence_and_zeroize(
    strict_signer_profile: Option<&str>,
    signer_selection: &KolmeLiveSignerSelection,
    private_key_hex: &mut String,
) -> Result<(), ConfigError> {
    secret_provider::ensure_kolme_live_strict_signer_secret_source_precedence_and_zeroize(
        strict_signer_profile,
        signer_selection,
        private_key_hex,
    )
}

pub(crate) fn build_kolme_live_direct_signed_wire_payload(
    base_url: &str,
    transport: &mut kamn_core::KolmeRuntimeCommitHttpTransport,
    request: &kamn_core::KolmeRuntimeCommitRequest,
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
) -> Result<(String, KolmeLiveSignerSelection), ConfigError> {
    direct_payload::build_kolme_live_direct_signed_wire_payload(
        base_url,
        transport,
        request,
        strict_signer_profile,
        strict_signer_key_source,
    )
}

fn validate_managed_signer_preflight(
    signer_selection: &KolmeLiveSignerSelection,
) -> Result<(), ConfigError> {
    ensure_kolme_live_managed_external_private_key_env_unset(signer_selection)?;
    let _managed_signer_required_marker = resolve_kolme_live_managed_signer_required_marker()?;
    let _managed_signer_command =
        managed_backend::resolve_required_kolme_live_managed_signer_command()?;
    let _managed_signer_public_key =
        managed_backend::resolve_required_managed_signer_public_key_hex(signer_selection)?;
    let _managed_key_reference = read_required_kolme_live_key_reference_from_env(signer_selection)?;
    Ok(())
}

pub(crate) fn enforce_kolme_live_signer_preflight(
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
) -> Result<KolmeLiveSignerPreflightReadiness, ConfigError> {
    let signer_selection =
        resolve_kolme_live_signer_selection(strict_signer_profile, strict_signer_key_source)?;
    let readiness = evaluate_kolme_live_signer_preflight_readiness(&signer_selection)?;
    let provider = secret_provider::EnvKolmeLiveSignerSecretProvider;
    use models::KolmeLiveSignerSecretProvider;
    provider.ensure_no_fallback_private_key_path()?;
    ensure_kolme_live_strict_signer_secret_source_precedence(
        strict_signer_profile,
        &signer_selection,
    )?;
    if signer_selection.key_source == super::KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL {
        validate_managed_signer_preflight(&signer_selection)?;
    }
    Ok(readiness)
}
