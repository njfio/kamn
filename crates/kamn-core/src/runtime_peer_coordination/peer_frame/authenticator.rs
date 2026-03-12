use super::super::*;
use super::errors::AuthenticatedPeerFrameError;
use super::frame::AuthenticatedPeerFrame;
use super::signing::{parse_agent_did, PeerDidRole};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Validates inbound runtime peer frames against local recipient, allowlist, and nonce replay state.
pub struct PeerFrameAuthenticator {
    local_peer_did: String,
    allowed_sender_dids: BTreeSet<String>,
    last_nonce_by_sender: BTreeMap<String, u64>,
}

impl PeerFrameAuthenticator {
    /// Handles new.
    pub fn new(
        local_peer_did: &str,
        allowed_sender_dids: Vec<String>,
    ) -> Result<Self, AuthenticatedPeerFrameError> {
        validate_local_peer_did(local_peer_did)?;
        let allowlist = build_allowlist(allowed_sender_dids)?;
        Ok(Self {
            local_peer_did: local_peer_did.to_owned(),
            allowed_sender_dids: allowlist,
            last_nonce_by_sender: BTreeMap::new(),
        })
    }

    /// Handles validate inbound.
    pub fn validate_inbound(
        &mut self,
        frame: &AuthenticatedPeerFrame,
    ) -> Result<(), AuthenticatedPeerFrameError> {
        frame.verify_signature()?;
        ensure_expected_recipient(self, frame)?;
        ensure_authorized_sender(self, frame)?;
        ensure_monotonic_nonce(self, frame)?;
        self.last_nonce_by_sender
            .insert(frame.sender_peer_did().to_owned(), frame.nonce());
        Ok(())
    }
}

fn validate_local_peer_did(local_peer_did: &str) -> Result<(), AuthenticatedPeerFrameError> {
    parse_agent_did(
        local_peer_did,
        "local_peer_did",
        RUNTIME_PEER_FRAME_INVALID_LOCAL_PEER_DID_REASON_CODE,
        PeerDidRole::LocalPeer,
    )?;
    Ok(())
}

fn build_allowlist(
    allowed_sender_dids: Vec<String>,
) -> Result<BTreeSet<String>, AuthenticatedPeerFrameError> {
    if allowed_sender_dids.is_empty() {
        return Err(AuthenticatedPeerFrameError::EmptyAllowedSenders);
    }
    let mut allowlist = BTreeSet::new();
    for sender_did in allowed_sender_dids {
        parse_agent_did(
            &sender_did,
            "allowed_sender_dids[]",
            RUNTIME_PEER_FRAME_INVALID_SENDER_DID_REASON_CODE,
            PeerDidRole::Sender,
        )?;
        allowlist.insert(sender_did);
    }
    Ok(allowlist)
}

fn ensure_expected_recipient(
    authenticator: &PeerFrameAuthenticator,
    frame: &AuthenticatedPeerFrame,
) -> Result<(), AuthenticatedPeerFrameError> {
    if frame.recipient_peer_did() == authenticator.local_peer_did {
        return Ok(());
    }
    Err(AuthenticatedPeerFrameError::WrongRecipient {
        expected: authenticator.local_peer_did.clone(),
        found: frame.recipient_peer_did().to_owned(),
    })
}

fn ensure_authorized_sender(
    authenticator: &PeerFrameAuthenticator,
    frame: &AuthenticatedPeerFrame,
) -> Result<(), AuthenticatedPeerFrameError> {
    if authenticator.allowed_sender_dids.contains(frame.sender_peer_did()) {
        return Ok(());
    }
    Err(AuthenticatedPeerFrameError::UnauthorizedSender(
        frame.sender_peer_did().to_owned(),
    ))
}

fn ensure_monotonic_nonce(
    authenticator: &PeerFrameAuthenticator,
    frame: &AuthenticatedPeerFrame,
) -> Result<(), AuthenticatedPeerFrameError> {
    if let Some(last_nonce) = authenticator.last_nonce_by_sender.get(frame.sender_peer_did()) {
        if frame.nonce() <= *last_nonce {
            return Err(AuthenticatedPeerFrameError::ReplayNonce {
                sender_did: frame.sender_peer_did().to_owned(),
                last_nonce: *last_nonce,
                found: frame.nonce(),
            });
        }
    }
    Ok(())
}
