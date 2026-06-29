use super::*;

pub(crate) fn parse_http_request(
    stream: &mut TcpStream,
) -> Result<(String, String, String, BTreeMap<String, String>), String> {
    let request_text = read_request_text(stream)?;
    let (request_head, request_body) = split_request(request_text.as_str())?;
    let request_line = request_head
        .lines()
        .next()
        .ok_or_else(|| "request line missing".to_owned())?;
    let headers = parse_headers(request_head)?;
    let (method, path) = parse_request_line(request_line)?;
    Ok((method, path, request_body.to_owned(), headers))
}

fn read_request_text(stream: &mut TcpStream) -> Result<String, String> {
    let request_bytes = read_request_bytes(stream)?;
    String::from_utf8(request_bytes).map_err(|_| "request was not valid utf-8".to_owned())
}

fn read_request_bytes(stream: &mut TcpStream) -> Result<Vec<u8>, String> {
    let mut request = Vec::new();
    let mut chunk = [0_u8; 1024];
    let mut expected_total_bytes = None;
    let mut header_end = None;
    set_request_timeout(stream)?;
    while continue_reading(request.len(), expected_total_bytes) {
        if !read_request_chunk(
            stream,
            &mut chunk,
            &mut request,
            &mut expected_total_bytes,
            &mut header_end,
        )? {
            break;
        }
    }
    Ok(request)
}

fn set_request_timeout(stream: &mut TcpStream) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("request read-timeout failed: {error}"))
}

fn continue_reading(request_len: usize, expected_total_bytes: Option<usize>) -> bool {
    !request_complete(request_len, expected_total_bytes)
}

fn read_request_chunk(
    stream: &mut TcpStream,
    chunk: &mut [u8; 1024],
    request: &mut Vec<u8>,
    expected_total_bytes: &mut Option<usize>,
    header_end: &mut Option<usize>,
) -> Result<bool, String> {
    match stream.read(chunk) {
        Ok(0) => Ok(false),
        Ok(read_count) => {
            update_request_state(
                request,
                &chunk[..read_count],
                expected_total_bytes,
                header_end,
            )?;
            Ok(true)
        }
        Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
            Ok(false)
        }
        Err(error) => Err(format!("request read failed: {error}")),
    }
}

fn update_request_state(
    request: &mut Vec<u8>,
    chunk: &[u8],
    expected_total_bytes: &mut Option<usize>,
    header_end: &mut Option<usize>,
) -> Result<(), String> {
    request.extend_from_slice(chunk);
    if header_end.is_none() {
        *header_end = header_terminator(request);
        if let Some(header_end_index) = *header_end {
            *expected_total_bytes = Some(expected_request_bytes(request, header_end_index)?);
        }
    }
    Ok(())
}

fn header_terminator(request: &[u8]) -> Option<usize> {
    request
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
}

fn expected_request_bytes(request: &[u8], header_end_index: usize) -> Result<usize, String> {
    let header = String::from_utf8(request[..header_end_index].to_vec())
        .map_err(|_| "request header was not valid utf-8".to_owned())?;
    Ok(header_end_index + parse_content_length(header.as_str())?)
}

fn request_complete(request_len: usize, expected_total_bytes: Option<usize>) -> bool {
    expected_total_bytes.is_some_and(|total| request_len >= total)
}

fn split_request(request_text: &str) -> Result<(&str, &str), String> {
    request_text
        .split_once("\r\n\r\n")
        .ok_or_else(|| "request header terminator missing".to_owned())
}

fn parse_headers(request_head: &str) -> Result<BTreeMap<String, String>, String> {
    let mut headers = BTreeMap::new();
    for line in request_head.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| "request header line missing ':' separator".to_owned())?;
        headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_owned());
    }
    Ok(headers)
}

fn parse_request_line(request_line: &str) -> Result<(String, String), String> {
    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| "request method missing".to_owned())?
        .to_owned();
    let path = parts
        .next()
        .ok_or_else(|| "request path missing".to_owned())?
        .to_owned();
    Ok((method, path))
}

fn parse_content_length(header: &str) -> Result<usize, String> {
    let value = header
        .lines()
        .find_map(content_length_header)
        .unwrap_or("0");
    value
        .parse::<usize>()
        .map_err(|_| "invalid content-length header".to_owned())
}

fn content_length_header(line: &str) -> Option<&str> {
    let (name, raw_value) = line.split_once(':')?;
    name.eq_ignore_ascii_case("Content-Length")
        .then_some(raw_value.trim())
}

pub(crate) fn write_http_response(
    stream: &mut TcpStream,
    status: u16,
    body: &str,
) -> Result<(), String> {
    let status_text = status_text(status);
    let body_len = body.len();
    let payload = format!(
        "HTTP/1.1 {status_text}\r\nContent-Type: application/json\r\nContent-Length: {body_len}\r\nConnection: close\r\n\r\n{body}"
    );
    stream
        .write_all(payload.as_bytes())
        .map_err(|error| format!("response write failed: {error}"))
}

fn status_text(status: u16) -> &'static str {
    match status {
        200 => "200 OK",
        201 => "201 Created",
        202 => "202 Accepted",
        401 => "401 Unauthorized",
        404 => "404 Not Found",
        _ => "500 Internal Server Error",
    }
}
