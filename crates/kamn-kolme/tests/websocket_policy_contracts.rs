use kamn_kolme::{
    try_take_websocket_frame, validate_websocket_handshake_response, KolmeWebsocketFrame,
};

#[test]
fn functional_websocket_frame_parser_maps_text_frame() {
    let mut buffer = vec![0x81, 0x05, b'h', b'e', b'l', b'l', b'o'];
    let frame = try_take_websocket_frame(&mut buffer)
        .expect("frame parsing should succeed")
        .expect("one frame should be available");
    assert_eq!(frame, KolmeWebsocketFrame::Text(b"hello".to_vec()));
    assert!(buffer.is_empty(), "frame bytes should be drained");
}

#[test]
fn regression_issue_1739_handshake_validation_fails_closed() {
    // Regression: #1739
    let error = validate_websocket_handshake_response(
        b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\n\r\n",
    )
    .expect_err("missing connection upgrade header must fail");
    assert_eq!(
        error.to_string(),
        "websocket handshake response missing upgrade headers"
    );
}
