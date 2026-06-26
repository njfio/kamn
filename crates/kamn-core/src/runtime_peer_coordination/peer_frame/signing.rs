use super::super::*;
use super::errors::AuthenticatedPeerFrameError;
use super::frame::AuthenticatedPeerFrame;

pub(super) fn parse_agent_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
    role: PeerDidRole,
) -> Result<AgentDid, AuthenticatedPeerFrameError> {
    AgentDid::parse(value).map_err(|error| match role {
        PeerDidRole::Sender => AuthenticatedPeerFrameError::InvalidSenderDid {
            field,
            reason_code,
            detail: error.to_string(),
        },
        PeerDidRole::Recipient => AuthenticatedPeerFrameError::InvalidRecipientDid {
            field,
            reason_code,
            detail: error.to_string(),
        },
        PeerDidRole::LocalPeer => AuthenticatedPeerFrameError::InvalidLocalPeerDid {
            field,
            reason_code,
            detail: error.to_string(),
        },
    })
}

#[derive(Debug, Clone, Copy)]
pub(super) enum PeerDidRole {
    Sender,
    Recipient,
    LocalPeer,
}

pub(super) fn expected_peer_frame_signature(
    sender_peer_did: &str,
    recipient_peer_did: &str,
    nonce: u64,
    payload: &str,
) -> Result<String, AuthenticatedPeerFrameError> {
    let private_key = resolve_runtime_peer_frame_signing_private_key_hex()?;
    service_auth_sign_with_private_key_hex(
        sender_peer_did,
        nonce,
        recipient_peer_did,
        payload,
        private_key.as_str(),
    )
    .map_err(|_| AuthenticatedPeerFrameError::InvalidSigningKeyMaterial)
}

fn resolve_runtime_peer_frame_signing_private_key_hex(
) -> Result<String, AuthenticatedPeerFrameError> {
    if let Ok(value) = env::var(RUNTIME_PEER_FRAME_SIGNING_PRIVATE_KEY_ENV) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_owned());
        }
    }
    if cfg!(debug_assertions) {
        if let Some(debug_key) = debug_fallback_signer_private_key_hex() {
            return Ok(debug_key.to_owned());
        }
    }
    Err(AuthenticatedPeerFrameError::MissingSigningKeyMaterial)
}

fn resolve_runtime_peer_frame_signing_public_key_hex() -> Result<String, AuthenticatedPeerFrameError>
{
    if let Ok(value) = env::var(RUNTIME_PEER_FRAME_SIGNING_PUBLIC_KEY_ENV) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_owned());
        }
    }
    let private_key = resolve_runtime_peer_frame_signing_private_key_hex()?;
    service_auth_public_key_hex_from_private_key_hex(private_key.as_str())
        .map_err(|_| AuthenticatedPeerFrameError::InvalidSigningKeyMaterial)
}

pub(super) fn ensure_peer_frame_wire_field(
    value: &str,
    field: &'static str,
) -> Result<(), AuthenticatedPeerFrameError> {
    if value.contains('|') || value.contains('\n') || value.contains('\r') {
        return Err(AuthenticatedPeerFrameError::InvalidWireFieldDelimiter { field });
    }
    Ok(())
}

pub(super) fn verify_peer_frame_signature(
    frame: &AuthenticatedPeerFrame,
) -> Result<(), AuthenticatedPeerFrameError> {
    let expected_public_key = resolve_runtime_peer_frame_signing_public_key_hex()?;
    if service_auth_verify_with_public_key_hex(
        frame.signature(),
        frame.sender_peer_did(),
        frame.nonce(),
        frame.recipient_peer_did(),
        frame.payload(),
        expected_public_key.as_str(),
    )
    .is_err()
    {
        return Err(AuthenticatedPeerFrameError::SignatureMismatch {
            expected: format!(
                "sig:secp256k1:baseline-v2:<recovery-id>:<signature-hex-for-{}>",
                frame.sender_peer_did()
            ),
            found: frame.signature().to_owned(),
        });
    }
    Ok(())
}

pub(super) fn serialize_peer_frame_wire(
    frame: &AuthenticatedPeerFrame,
) -> Result<String, AuthenticatedPeerFrameError> {
    ensure_peer_frame_wire_field(frame.frame_id(), "frame_id")?;
    ensure_peer_frame_wire_field(frame.sender_peer_did(), "sender_peer_did")?;
    ensure_peer_frame_wire_field(frame.recipient_peer_did(), "recipient_peer_did")?;
    ensure_peer_frame_wire_field(frame.payload(), "payload")?;
    ensure_peer_frame_wire_field(frame.signature(), "signature")?;
    Ok(format!(
        "frame|{}|{}|{}|{}|{}|{}",
        frame.frame_id(),
        frame.sender_peer_did(),
        frame.recipient_peer_did(),
        frame.nonce(),
        frame.payload(),
        frame.signature()
    ))
}

pub(super) fn parse_peer_frame_wire(
    raw: &str,
) -> Result<(String, String, String, u64, String, String), AuthenticatedPeerFrameError> {
    let segments: Vec<_> = raw.split('|').collect();
    if segments.len() != 7 || segments[0] != "frame" {
        return Err(AuthenticatedPeerFrameError::InvalidWireFormat(
            raw.to_owned(),
        ));
    }
    let nonce = segments[4]
        .parse::<u64>()
        .map_err(|_| AuthenticatedPeerFrameError::InvalidWireFormat(raw.to_owned()))?;
    Ok((
        segments[1].to_owned(),
        segments[2].to_owned(),
        segments[3].to_owned(),
        nonce,
        segments[5].to_owned(),
        segments[6].to_owned(),
    ))
}
