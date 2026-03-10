use super::*;
pub(crate) fn read_http_request(stream: &mut std::net::TcpStream) -> String {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];
    let mut header_end = None;
    let mut expected_total = None;

    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("read timeout should be set");

    loop {
        let read_count = stream
            .read(&mut chunk)
            .expect("request bytes should be readable");
        if read_count == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read_count]);

        if header_end.is_none() {
            header_end = buffer
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
                .map(|pos| pos + 4);
            if let Some(end) = header_end {
                let headers = String::from_utf8_lossy(&buffer[..end]);
                let content_length = headers
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        if name.eq_ignore_ascii_case("Content-Length") {
                            return value.trim().parse::<usize>().ok();
                        }
                        None
                    })
                    .unwrap_or(0);
                expected_total = Some(end + content_length);
            }
        }

        if let Some(total) = expected_total {
            if buffer.len() >= total {
                break;
            }
        }
    }

    String::from_utf8(buffer).expect("request should be valid utf-8")
}

pub(crate) fn spawn_single_request_server(
    response_body: String,
    status_line: &str,
    handler: impl Fn(String) + Send + 'static,
) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    let status_line = status_line.to_owned();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("connection should be accepted");
        let request = read_http_request(&mut stream);
        handler(request);

        let response = format!(
            "{status_line}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            response_body.len(),
            response_body
        );
        stream
            .write_all(response.as_bytes())
            .expect("response should write");
    });
    format!("http://{addr}")
}

pub(crate) fn spawn_server_with_raw_response(
    raw_response: String,
    handler: impl Fn(String) + Send + 'static,
) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("connection should be accepted");
        let request = read_http_request(&mut stream);
        handler(request);
        stream
            .write_all(raw_response.as_bytes())
            .expect("response should write");
    });
    format!("http://{addr}")
}

pub(crate) fn spawn_server_with_chunked_raw_response(
    first_chunk: String,
    second_chunk: String,
    chunk_delay: Duration,
    handler: impl Fn(String) + Send + 'static,
) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("connection should be accepted");
        let request = read_http_request(&mut stream);
        handler(request);
        stream
            .write_all(first_chunk.as_bytes())
            .expect("first response chunk should write");
        stream.flush().expect("first response chunk should flush");
        thread::sleep(chunk_delay);
        stream
            .write_all(second_chunk.as_bytes())
            .expect("second response chunk should write");
    });
    format!("http://{addr}")
}

pub(crate) type KeepAliveRequestLog = Arc<Mutex<Vec<String>>>;
pub(crate) type KeepAliveServerHandle = thread::JoinHandle<(usize, usize)>;
pub(crate) type KeepAliveServerSpawnResult = (String, KeepAliveRequestLog, KeepAliveServerHandle);

pub(crate) fn spawn_keep_alive_multi_request_server(
    response_body: String,
    status_line: &str,
    expected_requests: usize,
) -> KeepAliveServerSpawnResult {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    listener
        .set_nonblocking(true)
        .expect("listener should allow nonblocking accepts");
    let addr = listener.local_addr().expect("local addr should resolve");
    let recorded_requests = Arc::new(Mutex::new(Vec::new()));
    let recorded_requests_ref = Arc::clone(&recorded_requests);
    let status_line = status_line.to_owned();
    let handle = thread::spawn(move || {
        let mut accepted_connections = 0_usize;
        let mut handled_requests = 0_usize;
        let deadline = Instant::now() + Duration::from_secs(4);
        while handled_requests < expected_requests && Instant::now() < deadline {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    accepted_connections += 1;
                    loop {
                        let request = read_http_request(&mut stream);
                        if request.is_empty() {
                            break;
                        }
                        recorded_requests_ref
                            .lock()
                            .expect("request log mutex should lock")
                            .push(request);
                        handled_requests += 1;
                        let response = format!(
                            "{status_line}\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\r\n{}",
                            response_body.len(),
                            response_body
                        );
                        stream
                            .write_all(response.as_bytes())
                            .expect("response should write");
                        if handled_requests >= expected_requests {
                            break;
                        }
                    }
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("accept should succeed: {error}"),
            }
        }
        (accepted_connections, handled_requests)
    });
    (format!("http://{addr}"), recorded_requests, handle)
}

