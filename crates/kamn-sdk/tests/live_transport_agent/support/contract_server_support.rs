use super::*;

#[path = "contract_server_support/response_route_support.rs"]
mod response_route_support;

pub(crate) fn run_bound_live_transport_contract_server(
    listener: TcpListener,
    max_requests: u64,
    expected_agent_sender_did: &str,
    expected_message_body: Option<String>,
) -> Result<(), String> {
    configure_listener(&listener)?;
    let mut state = ContractServerState::new(expected_agent_sender_did, expected_message_body);
    serve_requests(listener, max_requests, &mut state)
}

pub(crate) struct ContractServerState {
    replay_guard: BTreeSet<(String, u64)>,
    expected_agent_sender_did: String,
    expected_message_body: Option<String>,
    registered_metadata: Option<(String, String, Vec<String>)>,
}

impl ContractServerState {
    fn new(expected_agent_sender_did: &str, expected_message_body: Option<String>) -> Self {
        Self {
            replay_guard: BTreeSet::new(),
            expected_agent_sender_did: expected_agent_sender_did.to_owned(),
            expected_message_body,
            registered_metadata: None,
        }
    }

    pub(crate) fn set_registered_metadata(&mut self, metadata: (String, String, Vec<String>)) {
        self.registered_metadata = Some(metadata);
    }

    pub(crate) fn registered_metadata(&self) -> Option<&(String, String, Vec<String>)> {
        self.registered_metadata.as_ref()
    }

    pub(crate) fn expected_message_body(&self) -> Option<&str> {
        self.expected_message_body.as_deref()
    }

    fn expected_agent_sender_did(&self) -> &str {
        self.expected_agent_sender_did.as_str()
    }
}

fn configure_listener(listener: &TcpListener) -> Result<(), String> {
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("nonblocking setup failed: {error}"))?;
    Ok(())
}

fn serve_requests(
    listener: TcpListener,
    max_requests: u64,
    state: &mut ContractServerState,
) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut served = 0_u64;
    while served < max_requests {
        if Instant::now() > deadline {
            return Err("server timed out before serving request budget".to_owned());
        }
        served = served.saturating_add(serve_once(&listener, state)?);
    }
    Ok(())
}

fn serve_once(listener: &TcpListener, state: &mut ContractServerState) -> Result<u64, String> {
    match listener.accept() {
        Ok((mut stream, _)) => serve_connection(&mut stream, state),
        Err(error) if matches!(error.kind(), ErrorKind::WouldBlock) => {
            thread::sleep(Duration::from_millis(5));
            Ok(0)
        }
        Err(error) => Err(format!("accept failed: {error}")),
    }
}

fn serve_connection(
    stream: &mut TcpStream,
    state: &mut ContractServerState,
) -> Result<u64, String> {
    let (method, path, body, headers) = parse_http_request(stream)?;
    let expected_sender = state.expected_agent_sender_did().to_owned();
    if let Err((status, error, reason_code, message)) = validate_auth(
        method.as_str(),
        path.as_str(),
        body.as_str(),
        &headers,
        &mut state.replay_guard,
        expected_sender.as_str(),
    ) {
        write_auth_failure(stream, status, error, reason_code, message)?;
        return Ok(1);
    }
    response_route_support::write_response(stream, &method, &path, &body, state)?;
    Ok(1)
}

fn write_auth_failure(
    stream: &mut TcpStream,
    status: u16,
    error: &str,
    reason_code: &str,
    message: &str,
) -> Result<(), String> {
    let payload = format!(
        "{{\"error\":\"{error}\",\"reason_code\":\"{reason_code}\",\"message\":\"{message}\"}}"
    );
    write_http_response(stream, status, payload.as_str())
}
