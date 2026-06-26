use super::super::*;
use super::route_scope_support::enrich_signed_headers_with_scope;

pub(crate) fn reserve_loopback_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    drop(listener);
    addr.to_string()
}

pub(crate) fn send_http_request(addr: &str, method: &str, path: &str, body: &str) -> String {
    send_http_request_with_headers(addr, method, path, body, &[])
}

pub(crate) fn send_http_request_with_headers(
    addr: &str,
    method: &str,
    path: &str,
    body: &str,
    headers: &[(&str, &str)],
) -> String {
    let enriched = enrich_signed_headers_with_scope(method, path, headers);
    let refs = header_refs(&enriched);
    send_http_request_with_headers_raw(addr, method, path, body, refs.as_slice())
}

pub(crate) fn send_http_request_with_headers_raw(
    addr: &str,
    method: &str,
    path: &str,
    body: &str,
    headers: &[(&str, &str)],
) -> String {
    let mut stream = connect_http_stream(addr);
    let request = render_http_request(addr, method, path, body, headers);
    stream
        .write_all(request.as_bytes())
        .expect("request should write");
    read_response_until_timeout(&mut stream)
}

pub(crate) async fn send_http_request_with_headers_async(
    addr: &str,
    method: &str,
    path: &str,
    body: &str,
    headers: &[(&str, &str)],
) -> Result<String, String> {
    let enriched = enrich_signed_headers_with_scope(method, path, headers);
    let request = render_http_request(addr, method, path, body, header_refs(&enriched).as_slice());
    let mut stream = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|error| format!("async http connect should succeed: {error}"))?;
    write_async_request(&mut stream, request.as_bytes()).await?;
    read_async_response(&mut stream).await
}

fn connect_http_stream(addr: &str) -> TcpStream {
    let stream = TcpStream::connect(addr).expect("endpoint should accept connections");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("read timeout should be configurable");
    stream
}

fn header_refs(headers: &[(String, String)]) -> Vec<(&str, &str)> {
    headers
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()))
        .collect()
}

pub(crate) fn render_http_request(
    host: &str,
    method: &str,
    path: &str,
    body: &str,
    headers: &[(&str, &str)],
) -> String {
    let header_lines = render_header_lines(headers);
    format!(
        "{method} {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\n{header_lines}Content-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )
}

pub(crate) fn render_header_lines(headers: &[(&str, &str)]) -> String {
    headers
        .iter()
        .map(|(name, value)| format!("{name}: {value}\r\n"))
        .collect()
}

fn read_response_until_timeout(stream: &mut TcpStream) -> String {
    let mut response = String::new();
    let mut chunk = [0_u8; 1024];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(count) => response
                .push_str(std::str::from_utf8(&chunk[..count]).expect("response must be utf-8")),
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break
            }
            Err(error) => panic!("response should be readable: {error}"),
        }
    }
    response
}

async fn write_async_request(
    stream: &mut tokio::net::TcpStream,
    request: &[u8],
) -> Result<(), String> {
    stream
        .write_all(request)
        .await
        .map_err(|error| format!("async http request should write: {error}"))?;
    stream
        .flush()
        .await
        .map_err(|error| format!("async http request should flush: {error}"))
}

async fn read_async_response(stream: &mut tokio::net::TcpStream) -> Result<String, String> {
    let mut response = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        match tokio::time::timeout(Duration::from_secs(2), stream.read(&mut chunk)).await {
            Ok(Ok(0)) => break,
            Ok(Ok(count)) => response.extend_from_slice(&chunk[..count]),
            Ok(Err(error)) => return Err(format!("async http response read failed: {error}")),
            Err(_) => break,
        }
    }
    String::from_utf8(response)
        .map_err(|error| format!("async http response was not utf-8: {error}"))
}
