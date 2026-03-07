use std::collections::BTreeMap;
use std::io::{ErrorKind, Read};
use std::net::TcpStream;
use std::time::Duration;

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
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("request read-timeout failed: {error}"))?;
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read_count) => {
                request.extend_from_slice(&chunk[..read_count]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break;
            }
            Err(error) => return Err(format!("request read failed: {error}")),
        }
    }
    Ok(request)
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
