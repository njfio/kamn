use super::super::*;
use super::errors::AuthenticatedPeerFrameError;
use super::signing::{
    ensure_peer_frame_wire_field, expected_peer_frame_signature, parse_agent_did,
    parse_peer_frame_wire, serialize_peer_frame_wire, verify_peer_frame_signature, PeerDidRole,
};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Authenticated peer frame.
pub struct AuthenticatedPeerFrame {
    frame_id: String,
    sender_peer_did: String,
    recipient_peer_did: String,
    nonce: u64,
    payload: String,
    signature: String,
}

impl AuthenticatedPeerFrame {
    /// Handles new.
    pub fn new(
        frame_id: &str,
        sender_peer_did: &str,
        recipient_peer_did: &str,
        nonce: u64,
        payload: &str,
        signature: &str,
    ) -> Result<Self, AuthenticatedPeerFrameError> {
        validate_new_frame_inputs(
            frame_id,
            sender_peer_did,
            recipient_peer_did,
            nonce,
            payload,
            signature,
        )?;
        Ok(build_authenticated_peer_frame(
            frame_id,
            sender_peer_did,
            recipient_peer_did,
            nonce,
            payload,
            signature,
        ))
    }

    /// Handles signed.
    pub fn signed(
        frame_id: &str,
        sender_peer_did: &str,
        recipient_peer_did: &str,
        nonce: u64,
        payload: &str,
    ) -> Result<Self, AuthenticatedPeerFrameError> {
        let signature =
            expected_peer_frame_signature(sender_peer_did, recipient_peer_did, nonce, payload)?;
        Self::new(
            frame_id,
            sender_peer_did,
            recipient_peer_did,
            nonce,
            payload,
            &signature,
        )
    }

    /// Handles frame id.
    pub fn frame_id(&self) -> &str {
        &self.frame_id
    }

    /// Handles sender peer did.
    pub fn sender_peer_did(&self) -> &str {
        &self.sender_peer_did
    }

    /// Handles recipient peer did.
    pub fn recipient_peer_did(&self) -> &str {
        &self.recipient_peer_did
    }

    /// Handles nonce.
    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    /// Handles payload.
    pub fn payload(&self) -> &str {
        &self.payload
    }

    /// Handles signature.
    pub fn signature(&self) -> &str {
        &self.signature
    }

    /// Handles verify signature.
    pub fn verify_signature(&self) -> Result<(), AuthenticatedPeerFrameError> {
        verify_peer_frame_signature(self)
    }

    /// Handles to wire.
    pub fn to_wire(&self) -> Result<String, AuthenticatedPeerFrameError> {
        serialize_peer_frame_wire(self)
    }

    /// Handles from wire.
    pub fn from_wire(raw: &str) -> Result<Self, AuthenticatedPeerFrameError> {
        let (frame_id, sender_peer_did, recipient_peer_did, nonce, payload, signature) =
            parse_peer_frame_wire(raw)?;
        Self::new(
            &frame_id,
            &sender_peer_did,
            &recipient_peer_did,
            nonce,
            &payload,
            &signature,
        )
    }
}

fn validate_new_frame_inputs(
    frame_id: &str,
    sender_peer_did: &str,
    recipient_peer_did: &str,
    nonce: u64,
    payload: &str,
    signature: &str,
) -> Result<(), AuthenticatedPeerFrameError> {
    if frame_id.trim().is_empty() {
        return Err(AuthenticatedPeerFrameError::InvalidFrameId);
    }
    validate_peer_frame_dids(sender_peer_did, recipient_peer_did)?;
    validate_peer_frame_payload(nonce, payload, signature)?;
    validate_peer_frame_wire_fields(
        frame_id,
        sender_peer_did,
        recipient_peer_did,
        payload,
        signature,
    )
}

fn validate_peer_frame_dids(
    sender_peer_did: &str,
    recipient_peer_did: &str,
) -> Result<(), AuthenticatedPeerFrameError> {
    parse_agent_did(
        sender_peer_did,
        "sender_peer_did",
        RUNTIME_PEER_FRAME_INVALID_SENDER_DID_REASON_CODE,
        PeerDidRole::Sender,
    )?;
    parse_agent_did(
        recipient_peer_did,
        "recipient_peer_did",
        RUNTIME_PEER_FRAME_INVALID_RECIPIENT_DID_REASON_CODE,
        PeerDidRole::Recipient,
    )?;
    Ok(())
}

fn validate_peer_frame_payload(
    nonce: u64,
    payload: &str,
    signature: &str,
) -> Result<(), AuthenticatedPeerFrameError> {
    if nonce == 0 {
        return Err(AuthenticatedPeerFrameError::InvalidNonce);
    }
    if payload.trim().is_empty() {
        return Err(AuthenticatedPeerFrameError::EmptyPayload);
    }
    if signature.trim().is_empty() {
        return Err(AuthenticatedPeerFrameError::EmptySignature);
    }
    Ok(())
}

fn validate_peer_frame_wire_fields(
    frame_id: &str,
    sender_peer_did: &str,
    recipient_peer_did: &str,
    payload: &str,
    signature: &str,
) -> Result<(), AuthenticatedPeerFrameError> {
    ensure_peer_frame_wire_field(frame_id, "frame_id")?;
    ensure_peer_frame_wire_field(sender_peer_did, "sender_peer_did")?;
    ensure_peer_frame_wire_field(recipient_peer_did, "recipient_peer_did")?;
    ensure_peer_frame_wire_field(payload, "payload")?;
    ensure_peer_frame_wire_field(signature, "signature")
}

fn build_authenticated_peer_frame(
    frame_id: &str,
    sender_peer_did: &str,
    recipient_peer_did: &str,
    nonce: u64,
    payload: &str,
    signature: &str,
) -> AuthenticatedPeerFrame {
    AuthenticatedPeerFrame {
        frame_id: frame_id.to_owned(),
        sender_peer_did: sender_peer_did.to_owned(),
        recipient_peer_did: recipient_peer_did.to_owned(),
        nonce,
        payload: payload.to_owned(),
        signature: signature.to_owned(),
    }
}
