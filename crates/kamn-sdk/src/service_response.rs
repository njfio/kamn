use super::{
    HttpResponse, SdkError, REASON_CODE_AUTH_NONCE_HEADER_MISSING, REASON_CODE_AUTH_NONCE_INVALID,
    REASON_CODE_AUTH_NONCE_NON_POSITIVE, REASON_CODE_AUTH_REPLAY_NONCE_DETECTED,
    REASON_CODE_AUTH_SENDER_DID_HEADER_MISSING, REASON_CODE_AUTH_SIGNATURE_HEADER_MISSING,
    REASON_CODE_AUTH_SIGNATURE_VERIFICATION_FAILED, REASON_CODE_LEGACY_BAD_REQUEST,
    REASON_CODE_LEGACY_CONFLICT, REASON_CODE_LEGACY_UNAUTHORIZED, REASON_CODE_LEGACY_UNKNOWN,
    REASON_CODE_METHOD_NOT_ALLOWED, REASON_CODE_ROUTE_NOT_FOUND,
    REASON_CODE_WEBSOCKET_UPGRADE_REQUIRED, REQUEST_AUTH_NONCE_HEADER,
    REQUEST_AUTH_SENDER_DID_HEADER, REQUEST_AUTH_SIGNATURE_HEADER,
};
use serde_json::Value;

pub(super) fn parse_http_response(response: &str) -> Result<HttpResponse, SdkError> {
    let (header, body) = response
        .split_once("\r\n\r\n")
        .ok_or(SdkError::TransportFailure(
            "service response missing header terminator",
        ))?;
    let status = status_from_header(header).ok_or(SdkError::TransportFailure(
        "service response status line invalid",
    ))?;
    if status >= 400 {
        return map_non_success_response(Some(status), body);
    }

    Ok(HttpResponse {
        status,
        body: body.to_owned(),
    })
}

pub(super) fn status_from_header(header: &str) -> Option<u16> {
    let line = header.lines().next()?;
    let raw_code = line.split_whitespace().nth(1)?;
    raw_code.parse::<u16>().ok()
}

pub(super) fn map_non_success_response<T>(status: Option<u16>, body: &str) -> Result<T, SdkError> {
    if let Some(status_code) = status {
        if let Some((error, reason_code, message)) = parse_service_api_error_envelope(body)
            .or_else(|| parse_service_api_legacy_error_envelope(status_code, body))
        {
            return Err(SdkError::ServiceApiError {
                status: status_code,
                error,
                reason_code,
                message,
            });
        }
    }
    match status {
        Some(409) => Err(SdkError::Conflict("request rejected by service api")),
        Some(401) => Err(SdkError::TransportFailure(
            "request rejected by service api",
        )),
        Some(400) => Err(SdkError::TransportFailure(
            "request rejected by service api",
        )),
        Some(404) => Err(SdkError::NotFound {
            entity: "service-route",
            id: "requested-route".to_owned(),
        }),
        _ => Err(SdkError::TransportFailure(
            "request rejected by service api",
        )),
    }
}

fn parse_service_api_error_envelope(body: &str) -> Option<(String, String, String)> {
    let error = json_optional_string_field(body, "error")?;
    let reason_code = json_optional_string_field(body, "reason_code")?;
    let message = json_optional_string_field(body, "message")?;
    Some((error, reason_code, message))
}

fn parse_service_api_legacy_error_envelope(
    status: u16,
    body: &str,
) -> Option<(String, String, String)> {
    let error = json_optional_string_field(body, "error")?;
    let reason = json_optional_string_field(body, "reason")?;
    let reason_code =
        classify_legacy_service_api_reason_code(status, error.as_str(), reason.as_str()).to_owned();
    Some((error, reason_code, reason))
}

fn classify_legacy_service_api_reason_code(status: u16, error: &str, reason: &str) -> &'static str {
    if reason.contains(REQUEST_AUTH_SENDER_DID_HEADER) {
        return REASON_CODE_AUTH_SENDER_DID_HEADER_MISSING;
    }
    if reason.contains(REQUEST_AUTH_NONCE_HEADER) && reason.contains("missing required header") {
        return REASON_CODE_AUTH_NONCE_HEADER_MISSING;
    }
    if reason.contains("invalid request nonce header") {
        return REASON_CODE_AUTH_NONCE_INVALID;
    }
    if reason.contains("request nonce must be positive") {
        return REASON_CODE_AUTH_NONCE_NON_POSITIVE;
    }
    if reason.contains(REQUEST_AUTH_SIGNATURE_HEADER) && reason.contains("missing required header")
    {
        return REASON_CODE_AUTH_SIGNATURE_HEADER_MISSING;
    }
    if reason.contains("signature verification failed") {
        return REASON_CODE_AUTH_SIGNATURE_VERIFICATION_FAILED;
    }
    if reason.contains("replay") {
        return REASON_CODE_AUTH_REPLAY_NONCE_DETECTED;
    }
    if reason.contains("websocket upgrade required") {
        return REASON_CODE_WEBSOCKET_UPGRADE_REQUIRED;
    }
    match status {
        404 => REASON_CODE_ROUTE_NOT_FOUND,
        405 => REASON_CODE_METHOD_NOT_ALLOWED,
        401 if error == "unauthorized" => REASON_CODE_LEGACY_UNAUTHORIZED,
        409 => REASON_CODE_LEGACY_CONFLICT,
        400 => REASON_CODE_LEGACY_BAD_REQUEST,
        _ => REASON_CODE_LEGACY_UNKNOWN,
    }
}

pub(super) fn expect_status(actual: u16, expected: u16) -> Result<(), SdkError> {
    if actual == expected {
        return Ok(());
    }
    if actual == 409 {
        return Err(SdkError::Conflict("request rejected by service api"));
    }
    Err(SdkError::TransportFailure(
        "request rejected by service api",
    ))
}

fn parse_json_root(payload: &str) -> Result<Value, SdkError> {
    serde_json::from_str(payload)
        .map_err(|_| SdkError::TransportFailure("service response payload was not valid json"))
}

pub(super) fn json_string_field(payload: &str, key: &str) -> Result<String, SdkError> {
    let root = parse_json_root(payload)?;
    let value = root.get(key).ok_or(SdkError::TransportFailure(
        "service response missing required field",
    ))?;
    if let Some(parsed) = value.as_str() {
        return Ok(parsed.to_owned());
    }
    if let Some(parsed) = value.as_u64() {
        return Ok(parsed.to_string());
    }
    Err(SdkError::TransportFailure(
        "service response field was not a string",
    ))
}

pub(super) fn json_u64_field(payload: &str, key: &str) -> Result<u64, SdkError> {
    let root = parse_json_root(payload)?;
    let value = root.get(key).ok_or(SdkError::TransportFailure(
        "service response missing required field",
    ))?;
    if let Some(parsed) = value.as_u64() {
        return Ok(parsed);
    }
    if let Some(parsed) = value.as_str().and_then(|raw| raw.parse::<u64>().ok()) {
        return Ok(parsed);
    }
    Err(SdkError::TransportFailure(
        "service response numeric field was malformed",
    ))
}

fn json_optional_string_field(payload: &str, key: &str) -> Option<String> {
    let root = serde_json::from_str::<Value>(payload).ok()?;
    let value = root.get(key)?;
    value.as_str().map(str::to_owned)
}

pub(super) fn json_string_array_field(payload: &str, key: &str) -> Result<Vec<String>, SdkError> {
    let root = parse_json_root(payload)?;
    let value = root.get(key).ok_or(SdkError::TransportFailure(
        "service response missing required field",
    ))?;
    let items = value.as_array().ok_or(SdkError::TransportFailure(
        "service response array field was malformed",
    ))?;
    let mut parsed = Vec::with_capacity(items.len());
    for item in items {
        let value = item.as_str().ok_or(SdkError::TransportFailure(
            "service response array item was malformed",
        ))?;
        parsed.push(value.to_owned());
    }
    Ok(parsed)
}
