use crate::SdkError;
use kamn_types::AgentDidKeyBindingError;
use std::io::ErrorKind;

pub(crate) const DEFAULT_CONNECT_RETRIES: u32 = 20;
pub(crate) const DEFAULT_RETRY_DELAY_MILLIS: u64 = 100;
pub(crate) const DEFAULT_MAX_WIRE_BYTES: usize = 32 * 1024;
pub(crate) const TCP_HANDSHAKE_VERSION: &str = "1";
pub(crate) const TCP_HANDSHAKE_PROFILE: &str = "secp256k1:baseline-v2";
pub(crate) const TCP_FRAME_DELIMITER: &str = "\n\n";
pub(crate) const FROM_DID_KEY_BINDING_REASON: &str =
    "must include key-binding fingerprint matching signer_public_key";

pub(crate) fn constant_time_eq_bytes(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut diff = 0_u8;
    for (lhs, rhs) in left.iter().zip(right.iter()) {
        diff |= lhs ^ rhs;
    }
    diff == 0
}

pub(crate) fn map_from_did_key_binding_error(error: AgentDidKeyBindingError) -> SdkError {
    match error {
        AgentDidKeyBindingError::InvalidPublicKeyHex => SdkError::InvalidInput {
            field: "signer_public_key",
            reason: "must be valid compressed secp256k1 public key hex",
        },
        AgentDidKeyBindingError::MissingKeyBinding
        | AgentDidKeyBindingError::KeyBindingMismatch { .. }
        | AgentDidKeyBindingError::InvalidMethodSpecificId(_) => SdkError::InvalidInput {
            field: "from",
            reason: FROM_DID_KEY_BINDING_REASON,
        },
    }
}

pub(crate) fn is_benign_tcp_shutdown_error(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        ErrorKind::NotConnected
            | ErrorKind::BrokenPipe
            | ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
    )
}

pub(crate) fn split_transport_payload(payload: &str) -> Result<(&str, &str), SdkError> {
    payload
        .split_once(TCP_FRAME_DELIMITER)
        .ok_or(SdkError::InvalidInput {
            field: "wire_payload",
            reason: "missing handshake frame delimiter",
        })
}
