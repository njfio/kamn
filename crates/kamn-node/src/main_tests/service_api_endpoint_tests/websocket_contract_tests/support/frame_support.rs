pub(crate) fn parse_websocket_response_frames(response: &[u8]) -> (String, Vec<String>) {
    let header_end = websocket_header_end(response);
    let header = websocket_header(response, header_end);
    let frames = websocket_text_frames(&response[header_end..]);
    (header, frames)
}

pub(crate) fn parse_websocket_response(response: &[u8]) -> (String, String) {
    let (header, frames) = parse_websocket_response_frames(response);
    let payload = frames
        .into_iter()
        .next()
        .expect("websocket response should include at least one text frame");
    (header, payload)
}

fn websocket_header_end(response: &[u8]) -> usize {
    response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
        .expect("websocket response should include header terminator")
}

fn websocket_header(response: &[u8], header_end: usize) -> String {
    std::str::from_utf8(&response[..header_end])
        .expect("websocket header should be utf-8")
        .to_owned()
}

fn websocket_text_frames(frame_bytes: &[u8]) -> Vec<String> {
    let mut frames = Vec::new();
    let mut frame_index = 0_usize;
    while let Some((next_index, maybe_frame)) = next_websocket_frame(frame_bytes, frame_index) {
        frame_index = next_index;
        match maybe_frame {
            Some(frame) => frames.push(frame),
            None => break,
        }
    }
    frames
}

fn next_websocket_frame(frame_bytes: &[u8], frame_index: usize) -> Option<(usize, Option<String>)> {
    if frame_index + 2 > frame_bytes.len() {
        return None;
    }
    let first = frame_bytes[frame_index];
    let second = frame_bytes[frame_index + 1];
    assert_eq!(first & 0x80, 0x80, "fragmented websocket frames are not supported by test parser");
    assert_eq!(second & 0x80, 0, "server websocket frame must be unmasked");
    let opcode = first & 0x0f;
    let (payload_index, payload_len) = websocket_payload_bounds(frame_bytes, frame_index, second);
    let payload_slice = &frame_bytes[payload_index..payload_index + payload_len];
    let next_index = payload_index + payload_len;
    if opcode == 0x8 {
        return Some((next_index, None));
    }
    let frame = (opcode == 0x1).then(|| websocket_text_payload(payload_slice));
    Some((next_index, frame))
}

fn websocket_payload_bounds(frame_bytes: &[u8], frame_index: usize, second: u8) -> (usize, usize) {
    let (payload_index, payload_len) = match second & 0x7f {
        value @ 0..=125 => (frame_index + 2, value as usize),
        126 => websocket_u16_payload_bounds(frame_bytes, frame_index),
        127 => websocket_u64_payload_bounds(frame_bytes, frame_index),
        _ => unreachable!("websocket payload marker is constrained to 7 bits"),
    };
    assert!(frame_bytes.len() >= payload_index + payload_len, "websocket frame payload length must be available");
    (payload_index, payload_len)
}

fn websocket_u16_payload_bounds(frame_bytes: &[u8], frame_index: usize) -> (usize, usize) {
    assert!(frame_bytes.len() >= frame_index + 4, "websocket frame extended payload length must be available");
    let payload_len = u16::from_be_bytes([frame_bytes[frame_index + 2], frame_bytes[frame_index + 3]]) as usize;
    (frame_index + 4, payload_len)
}

fn websocket_u64_payload_bounds(frame_bytes: &[u8], frame_index: usize) -> (usize, usize) {
    assert!(frame_bytes.len() >= frame_index + 10, "websocket frame 64-bit payload length must be available");
    let payload_len = u64::from_be_bytes([
        frame_bytes[frame_index + 2],
        frame_bytes[frame_index + 3],
        frame_bytes[frame_index + 4],
        frame_bytes[frame_index + 5],
        frame_bytes[frame_index + 6],
        frame_bytes[frame_index + 7],
        frame_bytes[frame_index + 8],
        frame_bytes[frame_index + 9],
    ]) as usize;
    (frame_index + 10, payload_len)
}

fn websocket_text_payload(payload_slice: &[u8]) -> String {
    std::str::from_utf8(payload_slice)
        .expect("websocket payload should be utf-8")
        .to_owned()
}
