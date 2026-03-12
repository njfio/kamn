use super::managed_flow::{
    build_kolme_live_direct_signed_wire_payload_with_managed_key_source_adapter,
    default_managed_key_source_adapter, validate_preflight,
};
use super::models::{KolmeLiveSignerSecretProvider, KolmeLiveSignerSelection};
use super::secret_provider::{
    ensure_kolme_live_strict_signer_secret_source_precedence_and_zeroize,
    EnvKolmeLiveSignerSecretProvider,
};
use kamn_core::ConfigError;
use super::managed_backend::ManagedExternalKeySourceAdapter;
use crate::signer::{
    read_required_kolme_live_key_reference_from_env, resolve_kolme_live_nonce,
    resolve_kolme_live_signer_selection,
};
use crate::KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL;
use crate::wire_payload::render_kolme_live_native_direct_message;
use super::signer_adapter::KolmeForkSecp256k1SignerAdapter;
use kamn_core::{KolmeRuntimeCommitHttpTransport, KolmeRuntimeCommitRequest};
use kamn_core::KolmeApiBroadcastRequest;

pub(crate) fn read_kolme_live_signer_private_key_hex_with_provider<P: KolmeLiveSignerSecretProvider>(
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
    provider: &P,
) -> Result<(String, KolmeLiveSignerSelection), ConfigError> {
    let selection =
        resolve_kolme_live_signer_selection(strict_signer_profile, strict_signer_key_source)?;
    provider.ensure_no_fallback_private_key_path()?;
    if selection.key_source == KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL {
        let key_reference = read_required_kolme_live_key_reference_from_env(&selection)?;
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer key reference {key_reference} from {} cannot use private-key signer adapter path; route through managed signer backend execution (managed_signer_private_key_adapter_unsupported)",
            selection.key_reference_env
        )));
    }
    let mut private_key_hex = provider.read_private_key_hex(&selection)?;
    ensure_kolme_live_strict_signer_secret_source_precedence_and_zeroize(
        strict_signer_profile,
        &selection,
        &mut private_key_hex,
    )?;
    Ok((private_key_hex, selection))
}

pub(crate) fn read_kolme_live_signer_private_key_hex(
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
) -> Result<(String, KolmeLiveSignerSelection), ConfigError> {
    let provider = EnvKolmeLiveSignerSecretProvider;
    read_kolme_live_signer_private_key_hex_with_provider(
        strict_signer_profile,
        strict_signer_key_source,
        &provider,
    )
}

pub(crate) fn build_kolme_live_signer_adapter(
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
) -> Result<(KolmeForkSecp256k1SignerAdapter, KolmeLiveSignerSelection), ConfigError> {
    let (mut private_key_hex, selection) =
        read_kolme_live_signer_private_key_hex(strict_signer_profile, strict_signer_key_source)?;
    let signer_adapter = KolmeForkSecp256k1SignerAdapter::from_private_key_hex_in_place(
        &mut private_key_hex,
        selection.private_key_env,
    )?;
    Ok((signer_adapter, selection))
}

pub(crate) fn build_kolme_live_direct_signed_wire_payload(
    base_url: &str,
    transport: &mut KolmeRuntimeCommitHttpTransport,
    request: &KolmeRuntimeCommitRequest,
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
) -> Result<(String, KolmeLiveSignerSelection), ConfigError> {
    let signer_selection =
        resolve_kolme_live_signer_selection(strict_signer_profile, strict_signer_key_source)?;
    validate_preflight(&signer_selection)?;
    let payload = if signer_selection.key_source == KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL {
        let managed_key_source_adapter: ManagedExternalKeySourceAdapter =
            default_managed_key_source_adapter();
        build_kolme_live_direct_signed_wire_payload_with_managed_key_source_adapter(
            base_url,
            transport,
            request,
            &signer_selection,
            &managed_key_source_adapter,
        )?
    } else {
        let (signer_adapter, signer_selection_from_adapter) =
            build_kolme_live_signer_adapter(strict_signer_profile, strict_signer_key_source)?;
        let pubkey = signer_adapter.public_key_compressed_hex();
        let nonce = resolve_kolme_live_nonce(base_url, transport, pubkey.as_str())?;
        let canonical_message =
            render_kolme_live_native_direct_message(request, pubkey.as_str(), nonce)?;
        let (signature_hex, recovery_id) = signer_adapter.sign_message(canonical_message.as_str())?;
        debug_assert_eq!(signer_selection.profile, signer_selection_from_adapter.profile);
        debug_assert_eq!(signer_selection.key_source, signer_selection_from_adapter.key_source);
        debug_assert_eq!(
            signer_selection.private_key_env,
            signer_selection_from_adapter.private_key_env
        );
        let request = KolmeApiBroadcastRequest::new(
            canonical_message.as_str(),
            signature_hex.as_str(),
            recovery_id,
        )
        .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
        request.to_json_payload()
    };
    Ok((payload, signer_selection))
}
