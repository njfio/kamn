use super::super::envelope::TcpSignedEnvelope;
use super::super::handshake::TcpHandshakeFrame;
use super::super::support::split_transport_payload;
use super::TcpReplayGuardState;
use crate::SdkError;
use std::io::Read;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub(super) fn read_envelope(
    stream: TcpStream,
    max_wire_bytes: usize,
    replay_guard_state: &Arc<Mutex<TcpReplayGuardState>>,
    verify_and_record: impl Fn(
        &Arc<Mutex<TcpReplayGuardState>>,
        &TcpSignedEnvelope,
    ) -> Result<(), SdkError>,
) -> Result<TcpSignedEnvelope, SdkError> {
    let payload = read_wire_payload(stream, max_wire_bytes)?;
    let (handshake_payload, envelope_payload) = split_transport_payload(payload.as_str())?;
    let handshake = TcpHandshakeFrame::parse_wire_payload(handshake_payload)?;
    let envelope = TcpSignedEnvelope::parse_wire_payload(envelope_payload)?;
    handshake.verify_matches_envelope(&envelope)?;
    verify_and_record(replay_guard_state, &envelope)?;
    Ok(envelope)
}

fn read_wire_payload(stream: TcpStream, max_wire_bytes: usize) -> Result<String, SdkError> {
    let mut payload = String::new();
    let mut limited_reader = stream.take((max_wire_bytes + 1) as u64);
    limited_reader
        .read_to_string(&mut payload)
        .map_err(|_| SdkError::TransportFailure("tcp read failed"))?;
    if payload.len() > max_wire_bytes {
        return Err(SdkError::InvalidInput {
            field: "wire_payload",
            reason: "exceeds max wire bytes",
        });
    }
    Ok(payload)
}
