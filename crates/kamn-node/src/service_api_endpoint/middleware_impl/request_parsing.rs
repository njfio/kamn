use super::*;

pub(super) async fn parse_service_api_request(
    request: Request,
    is_websocket_route: bool,
    body_limit_bytes: usize,
) -> Result<(Request, ParsedRequest), ServiceApiReasonedError> {
    let method_label = request.method().to_string();
    let path = request.uri().path().to_owned();
    let headers = request.headers().clone();
    if is_websocket_route {
        let parsed_request =
            build_parsed_request(method_label.as_str(), path.as_str(), &headers, Bytes::new())?;
        return Ok((request, parsed_request));
    }
    let (parts, body) = request.into_parts();
    let body = to_bytes(body, body_limit_bytes)
        .await
        .map_err(|error| map_body_read_error(body_limit_bytes, error))?;
    let parsed_request =
        build_parsed_request(method_label.as_str(), path.as_str(), &headers, body.clone())?;
    Ok((Request::from_parts(parts, Body::from(body)), parsed_request))
}

fn map_body_read_error(body_limit_bytes: usize, error: axum::Error) -> ServiceApiReasonedError {
    let message = error.to_string();
    if message.contains("length limit exceeded") {
        return ServiceApiReasonedError::new(
            REASON_CODE_INGRESS_BODY_SIZE_LIMIT_EXCEEDED,
            format!("request body size limit exceeded: {body_limit_bytes} bytes"),
        );
    }
    ServiceApiReasonedError::new(
        REASON_CODE_REQUEST_READ_FAILED,
        format!("request read failed: {error}"),
    )
}

pub(super) fn build_parsed_request(
    method: &str,
    path: &str,
    headers: &HeaderMap,
    body: Bytes,
) -> Result<ParsedRequest, ServiceApiReasonedError> {
    Ok(ParsedRequest {
        method: method.to_owned(),
        path: path.to_owned(),
        body: parse_utf8_body(body)?,
        headers: normalize_headers(headers)?,
    })
}

fn normalize_headers(
    headers: &HeaderMap,
) -> Result<BTreeMap<String, String>, ServiceApiReasonedError> {
    let mut normalized_headers = BTreeMap::new();
    for (header_name, header_value) in headers {
        normalized_headers.insert(
            header_name.as_str().to_ascii_lowercase(),
            parse_utf8_header_value(header_name.as_str(), header_value)?,
        );
    }
    Ok(normalized_headers)
}

fn parse_utf8_header_value(
    header_name: &str,
    header_value: &HeaderValue,
) -> Result<String, ServiceApiReasonedError> {
    header_value
        .to_str()
        .map(|value| value.trim().to_owned())
        .map_err(|_| {
            ServiceApiReasonedError::new(
                REASON_CODE_REQUEST_HEADER_UTF8_INVALID,
                format!("request header value was not valid utf-8: {header_name}"),
            )
        })
}

fn parse_utf8_body(body: Bytes) -> Result<String, ServiceApiReasonedError> {
    String::from_utf8(body.to_vec()).map_err(|_| {
        ServiceApiReasonedError::new(
            REASON_CODE_REQUEST_BODY_UTF8_INVALID,
            "request was not valid utf-8",
        )
    })
}
