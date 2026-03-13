use super::envelope::TcpSignedEnvelope;
use super::support::{constant_time_eq_bytes, TCP_HANDSHAKE_PROFILE, TCP_HANDSHAKE_VERSION};
use crate::{AgentDid, SdkError};

#[path = "handshake/parse.rs"]
mod parse;
use parse::{
    require_exact, required_nonce, required_string, set_nonce, set_once, verify_field_match,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TcpHandshakeFrame {
    from: AgentDid,
    to: AgentDid,
    nonce: u64,
    signer_public_key: String,
    signature: String,
}

impl TcpHandshakeFrame {
    pub(crate) fn from_envelope(envelope: &TcpSignedEnvelope) -> Self {
        Self {
            from: envelope.from.clone(),
            to: envelope.to.clone(),
            nonce: envelope.nonce,
            signer_public_key: envelope.signer_public_key.clone(),
            signature: envelope.signature.clone(),
        }
    }

    pub(crate) fn to_wire_payload(&self) -> String {
        format!(
            "frame=handshake\nversion={TCP_HANDSHAKE_VERSION}\nprofile={TCP_HANDSHAKE_PROFILE}\nfrom={}\nto={}\nnonce={}\nsigner_public_key={}\nsignature={}\n",
            self.from.as_str(), self.to.as_str(), self.nonce, self.signer_public_key, self.signature
        )
    }

    pub(crate) fn parse_wire_payload(payload: &str) -> Result<Self, SdkError> {
        let mut frame = None;
        let mut version = None;
        let mut profile = None;
        let mut from = None;
        let mut to = None;
        let mut nonce = None;
        let mut signer_public_key = None;
        let mut signature = None;
        for raw_line in payload.lines() {
            if raw_line.trim().is_empty() {
                continue;
            }
            let (key, raw_value) = raw_line.split_once('=').ok_or(SdkError::InvalidInput {
                field: "handshake_frame",
                reason: "line must contain key=value",
            })?;
            let value = raw_value.trim_end_matches('\r');
            match key {
                "frame" => set_once(&mut frame, value, "handshake_frame", "frame")?,
                "version" => set_once(&mut version, value, "handshake_frame", "version")?,
                "profile" => set_once(&mut profile, value, "handshake_frame", "profile")?,
                "from" => set_once(&mut from, value, "handshake_frame", "from")?,
                "to" => set_once(&mut to, value, "handshake_frame", "to")?,
                "nonce" => set_nonce(&mut nonce, value, "handshake.nonce")?,
                "signer_public_key" => set_once(
                    &mut signer_public_key,
                    value,
                    "handshake_frame",
                    "signer_public_key",
                )?,
                "signature" => set_once(&mut signature, value, "handshake_frame", "signature")?,
                _ => {
                    return Err(SdkError::InvalidInput {
                        field: "handshake_frame",
                        reason: "unknown key",
                    })
                }
            }
        }
        require_exact(
            required_string(frame, "handshake.frame")?,
            "handshake.frame",
            "handshake",
            "must equal handshake",
        )?;
        require_exact(
            required_string(version, "handshake.version")?,
            "handshake.version",
            TCP_HANDSHAKE_VERSION,
            "unsupported handshake version",
        )?;
        require_exact(
            required_string(profile, "handshake.profile")?,
            "handshake.profile",
            TCP_HANDSHAKE_PROFILE,
            "unsupported signature profile",
        )?;
        Ok(Self {
            from: AgentDid::parse(required_string(from, "handshake.from")?.as_str())?,
            to: AgentDid::parse(required_string(to, "handshake.to")?.as_str())?,
            nonce: required_nonce(nonce, "handshake.nonce")?,
            signer_public_key: required_string(signer_public_key, "handshake.signer_public_key")?,
            signature: required_string(signature, "handshake.signature")?,
        })
    }

    pub(crate) fn verify_matches_envelope(
        &self,
        envelope: &TcpSignedEnvelope,
    ) -> Result<(), SdkError> {
        verify_field_match(
            self.from == envelope.from,
            "handshake.from",
            "does not match envelope sender",
        )?;
        verify_field_match(
            self.to == envelope.to,
            "handshake.to",
            "does not match envelope recipient",
        )?;
        verify_field_match(
            self.nonce == envelope.nonce,
            "handshake.nonce",
            "does not match envelope nonce",
        )?;
        verify_field_match(
            constant_time_eq_bytes(
                self.signer_public_key.as_bytes(),
                envelope.signer_public_key.as_bytes(),
            ),
            "handshake.signer_public_key",
            "does not match envelope signer public key",
        )?;
        verify_field_match(
            constant_time_eq_bytes(self.signature.as_bytes(), envelope.signature.as_bytes()),
            "handshake.signature",
            "does not match envelope signature",
        )?;
        Ok(())
    }
}
