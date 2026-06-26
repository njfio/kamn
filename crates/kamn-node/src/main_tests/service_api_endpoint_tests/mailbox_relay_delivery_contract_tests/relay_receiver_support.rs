use super::super::*;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;

pub(super) struct RelayReceiver {
    pub handle: thread::JoinHandle<String>,
    pub route_map: String,
}

pub(super) fn spawn_relay_receiver(recipient_did: &str) -> RelayReceiver {
    let listener = TcpListener::bind("127.0.0.1:0").expect("relay receiver listener should bind");
    let route_map = serde_json::json!({
        recipient_did: listener.local_addr().expect("relay receiver addr should resolve").to_string(),
    })
    .to_string();
    let handle = thread::spawn(move || read_relay_request(listener));
    RelayReceiver { handle, route_map }
}

fn read_relay_request(listener: TcpListener) -> String {
    let (mut stream, _) = listener
        .accept()
        .expect("relay receiver should accept forwarding connection");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("relay receiver read timeout should configure");
    let mut request = String::new();
    let mut chunk = [0_u8; 1024];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(count) => request.push_str(
                std::str::from_utf8(&chunk[..count]).expect("relay request should be utf-8"),
            ),
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break
            }
            Err(error) => panic!("relay receiver request read should succeed: {error}"),
        }
        if request.contains("\r\n\r\n") {
            break;
        }
    }
    stream
        .write_all(b"HTTP/1.1 202 Accepted\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}")
        .expect("relay receiver response should write");
    request
}
