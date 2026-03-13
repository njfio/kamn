use crate::{AgentDid, SdkError};
use kamn_core::service_auth_verify_with_public_key_hex;

#[path = "envelope/parse.rs"]
mod parse;
#[path = "envelope/verify.rs"]
mod verify;
use parse::{build_envelope, parse_envelope_fields};
use verify::{
    derive_signer_public_key, map_signature_verify_error, sign_envelope_fields, verify_body_shape,
    verify_did_key_binding, verify_signer_public_key_shape, verify_state_hash_shape,
};

/// Cryptographically signed envelope transported over TCP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TcpSignedEnvelope {
    /// Sender DID.
    pub from: AgentDid,
    /// Recipient DID.
    pub to: AgentDid,
    /// Monotonic nonce.
    pub nonce: u64,
    /// Runtime state hash marker.
    pub state_hash: String,
    /// Message body.
    pub body: String,
    /// Compressed secp256k1 signer public key hex.
    pub signer_public_key: String,
    /// Cryptographic envelope signature marker.
    pub signature: String,
}

impl TcpSignedEnvelope {
    /// Builds and signs a cryptographic TCP envelope.
    pub fn new(
        from: AgentDid,
        to: AgentDid,
        nonce: u64,
        state_hash: impl Into<String>,
        body: impl Into<String>,
        signer_private_key_hex: &str,
    ) -> Result<Self, SdkError> {
        let state_hash = state_hash.into();
        let body = body.into();
        let envelope =
            Self::build_signed(from, to, nonce, state_hash, body, signer_private_key_hex)?;
        envelope.verify_integrity()?;
        Ok(envelope)
    }

    /// Returns wire payload in canonical field order.
    pub fn to_wire_payload(&self) -> String {
        format!(
            "from={}\nto={}\nnonce={}\nstate_hash={}\nbody={}\nsigner_public_key={}\nsignature={}\n",
            self.from.as_str(), self.to.as_str(), self.nonce, self.state_hash, self.body,
            self.signer_public_key, self.signature
        )
    }

    /// Parses and verifies wire payload.
    pub fn parse_wire_payload(payload: &str) -> Result<Self, SdkError> {
        let fields = parse_envelope_fields(payload)?;
        let envelope = build_envelope(fields)?;
        envelope.verify_integrity()?;
        Ok(envelope)
    }

    /// Verifies payload shape and cryptographic signature.
    pub fn verify_integrity(&self) -> Result<(), SdkError> {
        verify_state_hash_shape(self.state_hash.as_str())?;
        verify_body_shape(self.body.as_str())?;
        verify_signer_public_key_shape(self.signer_public_key.as_str())?;
        service_auth_verify_with_public_key_hex(
            self.signature.as_str(),
            self.from.as_str(),
            self.nonce,
            self.state_hash.as_str(),
            self.body.as_str(),
            self.signer_public_key.as_str(),
        )
        .map_err(map_signature_verify_error)?;
        verify_did_key_binding(&self.from, self.signer_public_key.as_str())?;
        Ok(())
    }

    fn build_signed(
        from: AgentDid,
        to: AgentDid,
        nonce: u64,
        state_hash: String,
        body: String,
        signer_private_key_hex: &str,
    ) -> Result<Self, SdkError> {
        Ok(Self {
            signer_public_key: derive_signer_public_key(signer_private_key_hex)?,
            signature: sign_envelope_fields(
                from.as_str(),
                nonce,
                state_hash.as_str(),
                body.as_str(),
                signer_private_key_hex,
            )?,
            from,
            to,
            nonce,
            state_hash,
            body,
        })
    }
}

/// Legacy deterministic signature fixture marker for TCP envelope fields.
///
/// This helper is kept for compatibility fixtures and must not be used as
/// a valid `TcpSignedEnvelope` signature.
pub fn signature_for_fields(from: &str, nonce: u64, state_hash: &str, body: &str) -> String {
    format!(
        "sig:deterministic-v1:baseline-v1:{from}:{nonce}:{state_hash}:{}",
        body.len()
    )
}
