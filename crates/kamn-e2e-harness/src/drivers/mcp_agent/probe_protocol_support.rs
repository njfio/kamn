pub(crate) fn build_framed_jsonrpc_request(payload: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{}", payload.len(), payload)
}

pub(crate) fn validate_probe_initialize_response(payload: &str) -> Result<(), String> {
    require_marker(payload, r#""jsonrpc":"2.0""#, "jsonrpc")?;
    require_marker(payload, r#""serverInfo""#, "serverInfo")
}

#[cfg(test)]
pub(crate) fn validate_probe_health_response(payload: &str) -> Result<(), String> {
    if super::json_optional_bool_field(payload, "ok").unwrap_or(false) {
        return Ok(());
    }
    Err(format!(
        "mcp live probe returned non-success health payload: {payload}"
    ))
}

pub(crate) fn parse_framed_jsonrpc_payloads(stream: &str) -> Result<Vec<String>, String> {
    let mut payloads = Vec::new();
    let mut cursor = skip_leading_newlines(stream.as_bytes(), 0)?;
    while cursor < stream.len() {
        let remaining = slice_from(stream, cursor, "framed stream cursor out of bounds")?;
        let (header_end, content_length) = parse_frame_header(remaining)?;
        let payload_start = add_cursor(cursor, header_end + 4, "framed payload start overflow")?;
        let payload_end = add_cursor(
            payload_start,
            content_length,
            "content-length overflows stream cursor",
        )?;
        payloads.push(read_payload(stream, payload_start, payload_end)?.to_owned());
        cursor = skip_leading_newlines(stream.as_bytes(), payload_end)?;
    }
    if payloads.is_empty() {
        return Err("no framed payloads parsed".to_owned());
    }
    Ok(payloads)
}

pub(crate) fn json_optional_string_field(payload: &str, key: &str) -> Option<String> {
    let marker = format!("\"{key}\":\"");
    let start = payload.find(marker.as_str())? + marker.len();
    let rest = payload.get(start..)?;
    let end = rest.find('"')?;
    Some(rest[..end].to_owned())
}

pub(crate) fn json_optional_u64_field(payload: &str, key: &str) -> Option<u64> {
    let marker = format!("\"{key}\":");
    let start = payload.find(marker.as_str())? + marker.len();
    let rest = payload.get(start..)?.trim_start();
    let digits = rest
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u64>().ok()
}

pub(crate) fn escape_json_scalar(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

pub(crate) fn require_marker(payload: &str, marker: &str, label: &str) -> Result<(), String> {
    if payload.contains(marker) {
        return Ok(());
    }
    Err(format!(
        "mcp live probe initialize response missing {label} marker: {payload}"
    ))
}

pub(crate) fn skip_leading_newlines(bytes: &[u8], mut cursor: usize) -> Result<usize, String> {
    while matches!(bytes.get(cursor), Some(b'\r' | b'\n')) {
        cursor = add_cursor(cursor, 1, "framed stream cursor overflow")?;
    }
    Ok(cursor)
}

pub(crate) fn parse_frame_header(remaining: &str) -> Result<(usize, usize), String> {
    let header_end = remaining
        .find("\r\n\r\n")
        .ok_or_else(|| "missing framed header terminator".to_owned())?;
    let header = &remaining[..header_end];
    let length_value = header
        .lines()
        .find_map(|line| line.strip_prefix("Content-Length: "))
        .ok_or_else(|| "missing content-length header".to_owned())?;
    let content_length = length_value
        .trim()
        .parse::<usize>()
        .map_err(|_| "content-length must be numeric".to_owned())?;
    Ok((header_end, content_length))
}

pub(crate) fn read_payload(stream: &str, start: usize, end: usize) -> Result<&str, String> {
    stream
        .get(start..end)
        .ok_or_else(|| "content-length exceeds available framed payload bytes".to_owned())
}

pub(crate) fn slice_from<'a>(
    stream: &'a str,
    start: usize,
    error: &str,
) -> Result<&'a str, String> {
    stream.get(start..).ok_or_else(|| error.to_owned())
}

pub(crate) fn add_cursor(base: usize, add: usize, error: &str) -> Result<usize, String> {
    base.checked_add(add).ok_or_else(|| error.to_owned())
}
