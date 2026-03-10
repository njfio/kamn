use super::*;

pub(crate) fn read_http_request(stream: &mut std::net::TcpStream) -> String {
    configure_request_timeout(stream);
    let buffer = read_http_request_bytes(stream);
    String::from_utf8(buffer).expect("request should be valid utf-8")
}

fn configure_request_timeout(stream: &std::net::TcpStream) {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("read timeout should be set");
}

fn read_http_request_bytes(stream: &mut std::net::TcpStream) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];
    let mut header_end = None;
    let mut expected_total = None;
    loop {
        let read_count = read_request_chunk(stream, &mut chunk);
        if read_count == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read_count]);
        update_expected_total(&buffer, &mut header_end, &mut expected_total);
        if request_complete(expected_total, buffer.len()) {
            break;
        }
    }
    buffer
}

fn read_request_chunk(stream: &mut std::net::TcpStream, chunk: &mut [u8; 1024]) -> usize {
    stream
        .read(chunk)
        .expect("request bytes should be readable")
}

fn update_expected_total(
    buffer: &[u8],
    header_end: &mut Option<usize>,
    expected_total: &mut Option<usize>,
) {
    if header_end.is_some() {
        return;
    }
    *header_end = find_header_end(buffer);
    if let Some(end) = *header_end {
        *expected_total = Some(end + parse_content_length(&buffer[..end]));
    }
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|pos| pos + 4)
}

fn parse_content_length(headers: &[u8]) -> usize {
    String::from_utf8_lossy(headers)
        .lines()
        .find_map(content_length_from_line)
        .unwrap_or(0)
}

fn content_length_from_line(line: &str) -> Option<usize> {
    let (name, value) = line.split_once(':')?;
    if !name.eq_ignore_ascii_case("Content-Length") {
        return None;
    }
    value.trim().parse::<usize>().ok()
}

fn request_complete(expected_total: Option<usize>, observed: usize) -> bool {
    expected_total.is_some_and(|total| observed >= total)
}
