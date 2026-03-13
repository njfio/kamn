use super::super::envelope::TcpSignedEnvelope;
use super::super::support::constant_time_eq_bytes;
use super::TcpHandshakeFrame;
use crate::SdkError;

pub(super) fn verify_envelope_match(
    handshake: &TcpHandshakeFrame,
    envelope: &TcpSignedEnvelope,
) -> Result<(), SdkError> {
    verify_field_match(
        handshake.from == envelope.from,
        "handshake.from",
        "does not match envelope sender",
    )?;
    verify_field_match(
        handshake.to == envelope.to,
        "handshake.to",
        "does not match envelope recipient",
    )?;
    verify_field_match(
        handshake.nonce == envelope.nonce,
        "handshake.nonce",
        "does not match envelope nonce",
    )?;
    verify_field_match(
        constant_time_eq_bytes(
            handshake.signer_public_key.as_bytes(),
            envelope.signer_public_key.as_bytes(),
        ),
        "handshake.signer_public_key",
        "does not match envelope signer public key",
    )?;
    verify_field_match(
        constant_time_eq_bytes(
            handshake.signature.as_bytes(),
            envelope.signature.as_bytes(),
        ),
        "handshake.signature",
        "does not match envelope signature",
    )?;
    Ok(())
}

fn verify_field_match(
    matches: bool,
    field: &'static str,
    reason: &'static str,
) -> Result<(), SdkError> {
    if matches {
        Ok(())
    } else {
        Err(SdkError::InvalidInput { field, reason })
    }
}
