use super::*;

pub(crate) const DEFAULT_WEBSOCKET_EVENT_PAYLOAD: &str =
    r#"{"event":"state-transition","runtime_mode":"api","role":"processor","sequence":1}"#;

pub(crate) fn deterministic_tag(payload: &[u8]) -> u64 {
    let mut acc: u64 = 0xcbf29ce484222325;
    for byte in payload {
        acc = acc.wrapping_mul(0x00000100000001B3);
        acc ^= u64::from(*byte);
    }
    acc
}

pub(crate) fn write_websocket_upgrade_response(
    stream: &mut TcpStream,
    payload: &str,
) -> Result<(), String> {
    stream
        .write_all(websocket_handshake().as_bytes())
        .map_err(|error| format!("websocket handshake write failed: {error}"))?;
    let frame = websocket_frame(payload.as_bytes());
    stream
        .write_all(frame.as_slice())
        .map_err(|error| format!("websocket frame write failed: {error}"))
}

fn websocket_handshake() -> &'static str {
    "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: kamn-test-accept\r\nX-KAMN-WebSocket-Contract: v1\r\n\r\n"
}

fn websocket_frame(payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(10 + payload.len());
    frame.push(0x81);
    append_payload_len(&mut frame, payload.len());
    frame.extend_from_slice(payload);
    frame
}

fn append_payload_len(frame: &mut Vec<u8>, payload_len: usize) {
    if payload_len <= 125 {
        frame.push(payload_len as u8);
        return;
    }
    if payload_len <= u16::MAX as usize {
        frame.push(126);
        frame.extend_from_slice((payload_len as u16).to_be_bytes().as_slice());
        return;
    }
    frame.push(127);
    frame.extend_from_slice((payload_len as u64).to_be_bytes().as_slice());
}
