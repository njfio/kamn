use super::*;

pub(crate) type KeepAliveRequestLog = Arc<Mutex<Vec<String>>>;
pub(crate) type KeepAliveServerHandle = thread::JoinHandle<(usize, usize)>;
pub(crate) type KeepAliveServerSpawnResult = (String, KeepAliveRequestLog, KeepAliveServerHandle);

pub(crate) fn spawn_keep_alive_multi_request_server(
    response_body: String,
    status_line: &str,
    expected_requests: usize,
) -> KeepAliveServerSpawnResult {
    let listener = bind_keep_alive_listener();
    let addr = listener.local_addr().expect("local addr should resolve");
    let recorded_requests = Arc::new(Mutex::new(Vec::new()));
    let request_log = Arc::clone(&recorded_requests);
    let status_line = status_line.to_owned();
    let handle = thread::spawn(move || {
        serve_keep_alive_requests(listener, request_log, response_body, status_line, expected_requests)
    });
    (format!("http://{addr}"), recorded_requests, handle)
}

fn bind_keep_alive_listener() -> TcpListener {
    let listener = bind_loopback_listener();
    listener
        .set_nonblocking(true)
        .expect("listener should allow nonblocking accepts");
    listener
}

fn serve_keep_alive_requests(
    listener: TcpListener,
    request_log: KeepAliveRequestLog,
    response_body: String,
    status_line: String,
    expected_requests: usize,
) -> (usize, usize) {
    let mut accepted_connections = 0_usize;
    let mut handled_requests = 0_usize;
    let deadline = Instant::now() + Duration::from_secs(4);
    while handled_requests < expected_requests && Instant::now() < deadline {
        match listener.accept() {
            Ok((mut stream, _)) => {
                accepted_connections += 1;
                handled_requests += handle_keep_alive_connection(
                    &mut stream,
                    &request_log,
                    &response_body,
                    &status_line,
                    expected_requests - handled_requests,
                );
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => panic!("accept should succeed: {error}"),
        }
    }
    (accepted_connections, handled_requests)
}

fn handle_keep_alive_connection(
    stream: &mut std::net::TcpStream,
    request_log: &KeepAliveRequestLog,
    response_body: &str,
    status_line: &str,
    remaining_requests: usize,
) -> usize {
    let mut handled = 0_usize;
    while handled < remaining_requests {
        let request = read_http_request(stream);
        if request.is_empty() {
            break;
        }
        record_keep_alive_request(request_log, request);
        write_plain_response(stream, status_line, response_body, true);
        handled += 1;
    }
    handled
}

fn record_keep_alive_request(request_log: &KeepAliveRequestLog, request: String) {
    request_log
        .lock()
        .expect("request log mutex should lock")
        .push(request);
}
