use super::*;

pub(crate) fn spawn_single_request_server(
    response_body: String,
    status_line: &str,
    handler: impl Fn(String) + Send + 'static,
) -> String {
    let listener = bind_loopback_listener();
    let addr = listener.local_addr().expect("local addr should resolve");
    let status_line = status_line.to_owned();
    thread::spawn(move || handle_single_response(listener, response_body, status_line, handler));
    format!("http://{addr}")
}

pub(super) fn bind_loopback_listener() -> TcpListener {
    TcpListener::bind("127.0.0.1:0").expect("listener should bind")
}

fn handle_single_response(
    listener: TcpListener,
    response_body: String,
    status_line: String,
    handler: impl Fn(String),
) {
    let (mut stream, request) = accept_request(listener);
    handler(request);
    write_plain_response(&mut stream, status_line.as_str(), response_body.as_str(), false);
}

fn accept_request(listener: TcpListener) -> (std::net::TcpStream, String) {
    let (mut stream, _) = listener.accept().expect("connection should be accepted");
    let request = read_http_request(&mut stream);
    (stream, request)
}

pub(super) fn write_plain_response(
    stream: &mut std::net::TcpStream,
    status_line: &str,
    response_body: &str,
    keep_alive: bool,
) {
    let connection = if keep_alive { "keep-alive" } else { "close" };
    let response = format!(
        "{status_line}\r\nContent-Length: {}\r\nConnection: {connection}\r\n\r\n{}",
        response_body.len(),
        response_body
    );
    stream
        .write_all(response.as_bytes())
        .expect("response should write");
}

pub(crate) fn spawn_server_with_raw_response(
    raw_response: String,
    handler: impl Fn(String) + Send + 'static,
) -> String {
    let listener = bind_loopback_listener();
    let addr = listener.local_addr().expect("local addr should resolve");
    thread::spawn(move || handle_raw_response(listener, raw_response, handler));
    format!("http://{addr}")
}

fn handle_raw_response(
    listener: TcpListener,
    raw_response: String,
    handler: impl Fn(String),
) {
    let (mut stream, request) = accept_request(listener);
    handler(request);
    stream
        .write_all(raw_response.as_bytes())
        .expect("response should write");
}

pub(crate) fn spawn_server_with_chunked_raw_response(
    first_chunk: String,
    second_chunk: String,
    chunk_delay: Duration,
    handler: impl Fn(String) + Send + 'static,
) -> String {
    let listener = bind_loopback_listener();
    let addr = listener.local_addr().expect("local addr should resolve");
    thread::spawn(move || {
        handle_chunked_response(listener, first_chunk, second_chunk, chunk_delay, handler)
    });
    format!("http://{addr}")
}

fn handle_chunked_response(
    listener: TcpListener,
    first_chunk: String,
    second_chunk: String,
    chunk_delay: Duration,
    handler: impl Fn(String),
) {
    let (mut stream, request) = accept_request(listener);
    handler(request);
    write_chunk(&mut stream, first_chunk.as_bytes(), true, "first response chunk should write");
    thread::sleep(chunk_delay);
    write_chunk(
        &mut stream,
        second_chunk.as_bytes(),
        false,
        "second response chunk should write",
    );
}

fn write_chunk(
    stream: &mut std::net::TcpStream,
    bytes: &[u8],
    flush: bool,
    message: &str,
) {
    stream.write_all(bytes).expect(message);
    if flush {
        stream.flush().expect("response chunk should flush");
    }
}
