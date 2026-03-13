use super::support::map_from_did_key_binding_error;
use crate::{AgentDid, SdkError};
use kamn_core::{
    service_auth_public_key_hex_from_private_key_hex, service_auth_sign_with_private_key_hex,
    service_auth_verify_with_public_key_hex, ServiceAuthSignatureError,
};

#[path = "envelope/parse.rs"]
mod parse;
use parse::{required_nonce, required_string, set_nonce, set_once};

/// Cryptographically signed envelope transported over TCP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TcpSignedEnvelope {
    pub from: AgentDid,
    pub to: AgentDid,
    pub nonce: u64,
    pub state_hash: String,
    pub body: String,
    pub signer_public_key: String,
    pub signature: String,
}

impl TcpSignedEnvelope {
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
        let signer_public_key = service_auth_public_key_hex_from_private_key_hex(
            signer_private_key_hex,
        )
        .map_err(|_| SdkError::InvalidInput {
            field: "signer_private_key",
            reason: "must be valid secp256k1 private key hex",
        })?;
        let signature = service_auth_sign_with_private_key_hex(
            from.as_str(),
            nonce,
            state_hash.as_str(),
            body.as_str(),
            signer_private_key_hex,
        )
        .map_err(|_| SdkError::InvalidInput {
            field: "signer_private_key",
            reason: "failed to sign tcp envelope fields",
        })?;
        let envelope = Self {
            from,
            to,
            nonce,
            state_hash,
            body,
            signer_public_key,
            signature,
        };
        envelope.verify_integrity()?;
        Ok(envelope)
    }

    pub fn to_wire_payload(&self) -> String {
        format!(
            "from={}\nto={}\nnonce={}\nstate_hash={}\nbody={}\nsigner_public_key={}\nsignature={}\n",
            self.from.as_str(), self.to.as_str(), self.nonce, self.state_hash, self.body,
            self.signer_public_key, self.signature
        )
    }

    pub fn parse_wire_payload(payload: &str) -> Result<Self, SdkError> {
        let mut from = None;
        let mut to = None;
        let mut nonce = None;
        let mut state_hash = None;
        let mut body = None;
        let mut signer_public_key = None;
        let mut signature = None;
        for raw_line in payload.lines() {
            if raw_line.trim().is_empty() {
                continue;
            }
            let (key, raw_value) = raw_line.split_once('=').ok_or(SdkError::InvalidInput {
                field: "wire_payload",
                reason: "line must contain key=value",
            })?;
            let value = raw_value.trim_end_matches('\r');
            match key {
                "from" => set_once(&mut from, value, "wire_payload", "from")?,
                "to" => set_once(&mut to, value, "wire_payload", "to")?,
                "nonce" => set_nonce(&mut nonce, value, "nonce")?,
                "state_hash" => set_once(&mut state_hash, value, "wire_payload", "state_hash")?,
                "body" => set_once(&mut body, value, "wire_payload", "body")?,
                "signer_public_key" => set_once(
                    &mut signer_public_key,
                    value,
                    "wire_payload",
                    "signer_public_key",
                )?,
                "signature" => set_once(&mut signature, value, "wire_payload", "signature")?,
                _ => {
                    return Err(SdkError::InvalidInput {
                        field: "wire_payload",
                        reason: "unknown key",
                    })
                }
            }
        }
        let envelope = Self {
            from: AgentDid::parse(required_string(from, "from")?.as_str())?,
            to: AgentDid::parse(required_string(to, "to")?.as_str())?,
            nonce: required_nonce(nonce, "nonce")?,
            state_hash: required_string(state_hash, "state_hash")?,
            body: required_string(body, "body")?,
            signer_public_key: required_string(signer_public_key, "signer_public_key")?,
            signature: required_string(signature, "signature")?,
        };
        envelope.verify_integrity()?;
        Ok(envelope)
    }

    pub fn verify_integrity(&self) -> Result<(), SdkError> {
        if self.state_hash.trim().is_empty() {
            return Err(SdkError::InvalidInput {
                field: "state_hash",
                reason: "must not be empty",
            });
        }
        if self.body.trim().is_empty() {
            return Err(SdkError::InvalidInput {
                field: "body",
                reason: "must not be empty",
            });
        }
        if self.state_hash.contains('\n') || self.state_hash.contains('\r') {
            return Err(SdkError::InvalidInput {
                field: "state_hash",
                reason: "must be single-line",
            });
        }
        if self.body.contains('\n') || self.body.contains('\r') {
            return Err(SdkError::InvalidInput {
                field: "body",
                reason: "must be single-line",
            });
        }
        if self.signer_public_key.trim().is_empty() {
            return Err(SdkError::InvalidInput {
                field: "signer_public_key",
                reason: "must not be empty",
            });
        }
        if self.signer_public_key.contains('\n') || self.signer_public_key.contains('\r') {
            return Err(SdkError::InvalidInput {
                field: "signer_public_key",
                reason: "must be single-line",
            });
        }
        match service_auth_verify_with_public_key_hex(
            self.signature.as_str(),
            self.from.as_str(),
            self.nonce,
            self.state_hash.as_str(),
            self.body.as_str(),
            self.signer_public_key.as_str(),
        ) {
            Ok(()) => Ok(()),
            Err(ServiceAuthSignatureError::InvalidPublicKeyHex)
            | Err(ServiceAuthSignatureError::EmptyField("expected_public_key_hex")) => {
                Err(SdkError::InvalidInput {
                    field: "signer_public_key",
                    reason: "must be valid compressed secp256k1 public key hex",
                })
            }
            Err(_) => Err(SdkError::InvalidInput {
                field: "signature",
                reason: "failed cryptographic envelope verification",
            }),
        }?;
        self.from
            .ensure_public_key_hex_binding(self.signer_public_key.as_str())
            .map_err(map_from_did_key_binding_error)?;
        Ok(())
    }
}

pub fn signature_for_fields(from: &str, nonce: u64, state_hash: &str, body: &str) -> String {
    format!(
        "sig:deterministic-v1:baseline-v1:{from}:{nonce}:{state_hash}:{}",
        body.len()
    )
}
