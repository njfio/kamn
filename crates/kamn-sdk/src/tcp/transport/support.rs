use super::super::envelope::TcpSignedEnvelope;
use super::super::handshake::TcpHandshakeFrame;

pub(super) fn serialize_transport_payload(
    handshake: &TcpHandshakeFrame,
    envelope: &TcpSignedEnvelope,
) -> String {
    let mut payload = handshake.to_wire_payload();
    payload.push('\n');
    payload.push_str(&envelope.to_wire_payload());
    payload
}
