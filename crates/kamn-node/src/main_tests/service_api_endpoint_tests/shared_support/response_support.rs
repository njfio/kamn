use super::super::*;
use super::auth_fixture_support::ServiceApiErrorEnvelope;

pub(crate) fn parse_http_content_length(response_head: &str) -> usize {
    response_head
        .lines()
        .find_map(parse_content_length_line)
        .unwrap_or(0)
}

pub(crate) fn extract_http_response_body(response: &str) -> &str {
    response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or("")
}

pub(crate) fn parse_error_envelope(body: &str) -> ServiceApiErrorEnvelope {
    serde_json::from_str(body).expect("error payload should deserialize")
}

pub(crate) fn parse_error_envelope_from_http_response(response: &str) -> ServiceApiErrorEnvelope {
    parse_error_envelope(extract_http_response_body(response))
}

pub(crate) fn parse_scalar_metric_value(response: &str, metric_name: &str) -> Option<u64> {
    let expected_prefix = format!("{metric_name} ");
    extract_http_response_body(response)
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix(expected_prefix.as_str())?
                .parse::<u64>()
                .ok()
        })
}

pub(crate) fn read_single_http_response(stream: &mut TcpStream) -> String {
    String::from_utf8(read_http_response_bytes(stream)).expect("http response should be utf-8")
}

pub(crate) fn wait_for_endpoint_ready(addr: &str) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if TcpStream::connect(addr).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("endpoint did not become ready within timeout");
}

fn parse_content_length_line(line: &str) -> Option<usize> {
    let (name, value) = line.split_once(':')?;
    name.eq_ignore_ascii_case("Content-Length")
        .then(|| value.trim().parse::<usize>().unwrap_or(0))
}

fn read_http_response_bytes(stream: &mut TcpStream) -> Vec<u8> {
    let mut response = Vec::new();
    let mut chunk = [0_u8; 1024];
    let mut expected_len = None;
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(count) => extend_response(stream, &mut response, &chunk, count, &mut expected_len),
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break
            }
            Err(error) => panic!("response should be readable: {error}"),
        }
        if expected_len.is_some_and(|total| response.len() >= total) {
            break;
        }
    }
    response
}

fn extend_response(
    _stream: &mut TcpStream,
    response: &mut Vec<u8>,
    chunk: &[u8; 1024],
    count: usize,
    expected_len: &mut Option<usize>,
) {
    response.extend_from_slice(&chunk[..count]);
    if expected_len.is_none() {
        *expected_len = expected_response_len(response);
    }
}

fn expected_response_len(response: &[u8]) -> Option<usize> {
    let header_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")?
        + 4;
    let head = String::from_utf8_lossy(&response[..header_end]);
    Some(header_end + parse_http_content_length(head.as_ref()))
}
