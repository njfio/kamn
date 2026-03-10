use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub(crate) struct MockHttpReply {
    pub(crate) status_line: &'static str,
    pub(crate) body: String,
}

impl MockHttpReply {
    pub(crate) fn ok(body: &str) -> Self {
        Self {
            status_line: "HTTP/1.1 200 OK",
            body: body.to_owned(),
        }
    }
}

fn read_http_request(stream: &mut std::net::TcpStream) -> String {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];
    let mut header_end = None;
    let mut expected_total = None;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("read timeout should be set");

    loop {
        let Some(read_count) = read_request_chunk(stream, &mut chunk) else {
            break;
        };
        buffer.extend_from_slice(&chunk[..read_count]);
        update_expected_total(&buffer, &mut header_end, &mut expected_total);
        if expected_total.is_some_and(|total| buffer.len() >= total) {
            break;
        }
    }

    String::from_utf8(buffer).expect("request should be valid utf-8")
}

fn read_request_chunk(stream: &mut std::net::TcpStream, chunk: &mut [u8; 1024]) -> Option<usize> {
    match stream.read(chunk) {
        Ok(0) => None,
        Ok(read_count) => Some(read_count),
        Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => None,
        Err(error) => panic!("request bytes should be readable: {error}"),
    }
}

fn update_expected_total(
    buffer: &[u8],
    header_end: &mut Option<usize>,
    expected_total: &mut Option<usize>,
) {
    if header_end.is_some() {
        return;
    }
    *header_end = buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|pos| pos + 4);
    *expected_total = header_end.map(|end| end + content_length(&buffer[..end]));
}

fn content_length(headers: &[u8]) -> usize {
    String::from_utf8_lossy(headers)
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("Content-Length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0)
}

pub(crate) fn request_body(raw_request: &str) -> &str {
    raw_request
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or("")
}

pub(crate) fn spawn_kolme_live_mock_server(
    replies: Vec<MockHttpReply>,
) -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    listener
        .set_nonblocking(true)
        .expect("listener should allow nonblocking accepts");
    let addr = listener.local_addr().expect("local addr should resolve");
    let recorded_requests = Arc::new(Mutex::new(Vec::new()));
    let recorded_requests_ref = Arc::clone(&recorded_requests);
    thread::spawn(move || serve_mock_requests(listener, replies, recorded_requests_ref));
    (format!("http://{addr}"), recorded_requests)
}

fn serve_mock_requests(
    listener: TcpListener,
    replies: Vec<MockHttpReply>,
    recorded_requests: Arc<Mutex<Vec<String>>>,
) {
    for reply in replies {
        if let Some(mut stream) = accept_mock_stream(&listener) {
            let request = read_http_request(&mut stream);
            recorded_requests
                .lock()
                .expect("request mutex should lock")
                .push(request);
            write_mock_reply(&mut stream, reply);
        } else {
            return;
        }
    }
}

fn accept_mock_stream(listener: &TcpListener) -> Option<std::net::TcpStream> {
    let accept_deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match listener.accept() {
            Ok((stream, _)) => return Some(stream),
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                if Instant::now() >= accept_deadline {
                    return None;
                }
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => panic!("accept should succeed: {error}"),
        }
    }
}

fn write_mock_reply(stream: &mut std::net::TcpStream, reply: MockHttpReply) {
    let response = format!(
        "{}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        reply.status_line,
        reply.body.len(),
        reply.body
    );
    stream
        .write_all(response.as_bytes())
        .expect("response should write");
}
