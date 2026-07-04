use std::collections::BTreeMap;
use std::io::{ErrorKind, Read};
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, Instant};

pub(crate) fn parse_http_request(
    stream: &mut TcpStream,
) -> Result<(String, String, String, BTreeMap<String, String>), String> {
    let request_text = String::from_utf8(read_request_bytes(stream)?)
        .map_err(|_| "request was not valid utf-8".to_owned())?;
    let (request_head, request_body) = split_request_text(&request_text)?;
    let (method, path) = parse_request_line(request_head.lines().next())?;
    Ok((
        method,
        path,
        request_body.to_owned(),
        parse_headers(request_head),
    ))
}

fn read_request_bytes(stream: &mut TcpStream) -> Result<Vec<u8>, String> {
    let mut request = Vec::new();
    let mut chunk = [0_u8; 1024];
    stream
        .set_read_timeout(Some(Duration::from_millis(100)))
        .map_err(|error| format!("request read-timeout failed: {error}"))?;
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut expected_total_bytes: Option<usize> = None;
    let mut header_end: Option<usize> = None;
    loop {
        let Some(read_count) = read_next_chunk(stream, &mut chunk, deadline)? else {
            break;
        };
        request.extend_from_slice(&chunk[..read_count]);
        if header_end.is_none() {
            header_end = find_header_end(&request);
            expected_total_bytes = expected_total_bytes_from_header(&request, header_end)?;
        }
        if expected_total_bytes.is_some_and(|total| request.len() >= total) {
            break;
        }
    }
    Ok(request)
}

fn read_next_chunk(
    stream: &mut TcpStream,
    chunk: &mut [u8],
    deadline: Instant,
) -> Result<Option<usize>, String> {
    match stream.read(chunk) {
        Ok(0) => Ok(None),
        Ok(read_count) => Ok(Some(read_count)),
        Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
            wait_for_request_bytes(deadline)?;
            Ok(Some(0))
        }
        Err(error) => Err(format!("request read failed: {error}")),
    }
}

fn wait_for_request_bytes(deadline: Instant) -> Result<(), String> {
    if Instant::now() > deadline {
        return Err("request read timed out before complete http payload".to_owned());
    }
    thread::sleep(Duration::from_millis(5));
    Ok(())
}

fn expected_total_bytes_from_header(
    request: &[u8],
    header_end: Option<usize>,
) -> Result<Option<usize>, String> {
    let Some(header_end_index) = header_end else {
        return Ok(None);
    };
    let header = String::from_utf8(request[..header_end_index].to_vec())
        .map_err(|_| "request header was not valid utf-8".to_owned())?;
    Ok(Some(
        header_end_index + parse_content_length(header.as_str())?,
    ))
}

fn find_header_end(request: &[u8]) -> Option<usize> {
    request
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
}

fn parse_content_length(header: &str) -> Result<usize, String> {
    let value = header
        .lines()
        .find_map(|line| {
            let (name, raw_value) = line.split_once(':')?;
            if name.eq_ignore_ascii_case("Content-Length") {
                return Some(raw_value.trim());
            }
            None
        })
        .unwrap_or("0");
    value
        .parse::<usize>()
        .map_err(|_| "invalid content-length header".to_owned())
}

fn split_request_text(request_text: &str) -> Result<(&str, &str), String> {
    request_text
        .split_once("\r\n\r\n")
        .ok_or_else(|| "request header terminator missing".to_owned())
}

fn parse_request_line(request_line: Option<&str>) -> Result<(String, String), String> {
    let request_line = request_line.ok_or_else(|| "request line missing".to_owned())?;
    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| "request method missing".to_owned())?;
    let path = parts
        .next()
        .ok_or_else(|| "request path missing".to_owned())?;
    Ok((method.to_owned(), path.to_owned()))
}

fn parse_headers(request_head: &str) -> BTreeMap<String, String> {
    let mut headers = BTreeMap::new();
    for line in request_head.lines().skip(1) {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_owned());
    }
    headers
}
