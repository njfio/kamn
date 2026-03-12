use super::super::support::{
    SELF_CERTIFYING_AGENT_DID_KEY_PREFIX, header_value, normalized_public_key_hexes_match,
};
use super::*;

pub(crate) fn resolve_signer_public_key_for_request<'a>(
    headers: &'a BTreeMap<String, String>,
    fallback_public_key_hex: Option<&'a str>,
    allow_legacy_sender_binding: bool,
) -> Result<&'a str, RequestAuthFailure> {
    if let Some(public_key_hex) = header_value(headers, REQUEST_AUTH_SIGNER_PUBLIC_KEY_HEADER) {
        let normalized = public_key_hex.trim();
        if normalized.is_empty() {
            return Err(RequestAuthFailure::Unauthorized(
                ServiceApiReasonedError::new(
                    REASON_CODE_AUTH_SIGNATURE_VERIFICATION_FAILED,
                    format!(
                        "invalid signer public key header: {REQUEST_AUTH_SIGNER_PUBLIC_KEY_HEADER}"
                    ),
                ),
            ));
        }
        return Ok(public_key_hex);
    }
    if allow_legacy_sender_binding {
        if let Some(public_key_hex) = fallback_public_key_hex {
            return Ok(public_key_hex);
        }
    }
    Err(RequestAuthFailure::Unauthorized(
        ServiceApiReasonedError::new(
            REASON_CODE_AUTH_SIGNATURE_VERIFICATION_FAILED,
            format!("missing required header: {REQUEST_AUTH_SIGNER_PUBLIC_KEY_HEADER}"),
        ),
    ))
}

pub(crate) fn sender_did_matches_signer_public_key(
    sender_did: &str,
    signer_public_key_hex: &str,
    allow_legacy_sender_binding: bool,
) -> bool {
    if agent_did_matches_public_key(sender_did, signer_public_key_hex) {
        return true;
    }
    if self_certifying_did_matches_public_key(sender_did, signer_public_key_hex) {
        return true;
    }
    allow_legacy_sender_binding
}

fn agent_did_matches_public_key(sender_did: &str, signer_public_key_hex: &str) -> bool {
    let Ok(parsed_did) = AgentDid::parse(sender_did) else {
        return false;
    };
    parsed_did
        .ensure_public_key_hex_binding(signer_public_key_hex)
        .is_ok()
}

fn self_certifying_did_matches_public_key(sender_did: &str, signer_public_key_hex: &str) -> bool {
    let Some(bound_public_key_hex) = sender_did.strip_prefix(SELF_CERTIFYING_AGENT_DID_KEY_PREFIX)
    else {
        return false;
    };
    normalized_public_key_hexes_match(bound_public_key_hex.trim(), signer_public_key_hex.trim())
}
