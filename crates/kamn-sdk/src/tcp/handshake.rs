use super::envelope::TcpSignedEnvelope;
use super::support::{TCP_HANDSHAKE_PROFILE, TCP_HANDSHAKE_VERSION};
use crate::{AgentDid, SdkError};

#[path = "handshake/field_support.rs"]
mod field_support;
#[path = "handshake/parse.rs"]
mod parse;
#[path = "handshake/verify.rs"]
mod verify;
use parse::{build_handshake_frame, parse_handshake_fields};
use verify::verify_envelope_match;

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
        let fields = parse_handshake_fields(payload)?;
        build_handshake_frame(fields)
    }

    pub(crate) fn verify_matches_envelope(
        &self,
        envelope: &TcpSignedEnvelope,
    ) -> Result<(), SdkError> {
        verify_envelope_match(self, envelope)
    }
}
