//! Codec contracts for Kolme runtime-commit payloads.

use std::error::Error;
use std::fmt;

/// Codec-level error for Kolme payload transforms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeCodecError {
    /// Payload is empty and cannot be encoded/decoded.
    EmptyPayload,
}

impl fmt::Display for KolmeCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPayload => f.write_str("payload must not be empty"),
        }
    }
}

impl Error for KolmeCodecError {}

/// Minimal codec boundary used by the runtime-commit pipeline scaffold.
pub trait KolmeWireCodec {
    /// Encodes an internal payload into wire format.
    fn encode(&self, payload: &[u8]) -> Result<Vec<u8>, KolmeCodecError>;

    /// Decodes a wire payload into internal representation.
    fn decode(&self, payload: &[u8]) -> Result<Vec<u8>, KolmeCodecError>;
}

/// A deterministic passthrough codec used for scaffold tests.
#[derive(Debug, Default, Clone, Copy)]
pub struct PassthroughCodec;

impl KolmeWireCodec for PassthroughCodec {
    fn encode(&self, payload: &[u8]) -> Result<Vec<u8>, KolmeCodecError> {
        if payload.is_empty() {
            return Err(KolmeCodecError::EmptyPayload);
        }
        Ok(payload.to_vec())
    }

    fn decode(&self, payload: &[u8]) -> Result<Vec<u8>, KolmeCodecError> {
        if payload.is_empty() {
            return Err(KolmeCodecError::EmptyPayload);
        }
        Ok(payload.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::{KolmeCodecError, KolmeWireCodec, PassthroughCodec};

    #[test]
    fn unit_passthrough_codec_rejects_empty_payload() {
        let codec = PassthroughCodec;
        assert_eq!(codec.encode(&[]), Err(KolmeCodecError::EmptyPayload));
        assert_eq!(codec.decode(&[]), Err(KolmeCodecError::EmptyPayload));
    }

    #[test]
    fn unit_passthrough_codec_roundtrips_non_empty_payload() {
        let codec = PassthroughCodec;
        let payload = b"runtime-commit";
        let encoded = codec.encode(payload).expect("encode should succeed");
        let decoded = codec.decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded, payload);
    }
}
