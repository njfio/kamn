use super::{
    SdkError, ServiceRequestAuth, MAX_SERVICE_RESPONSE_BYTES, MAX_SERVICE_RESPONSE_READ_ITERATIONS,
    REQUEST_AUTH_NONCE_HEADER, REQUEST_AUTH_SCOPE_HEADER, REQUEST_AUTH_SENDER_DID_HEADER,
    REQUEST_AUTH_SIGNATURE_HEADER, REQUEST_AUTH_SIGNER_PUBLIC_KEY_HEADER,
    SERVICE_RESPONSE_READ_ITERATION_LIMIT_EXCEEDED, SERVICE_RESPONSE_SIZE_LIMIT_EXCEEDED,
    SERVICE_TLS_CERTIFICATE_VERIFICATION_FAILED, SERVICE_TLS_HANDSHAKE_FAILED,
};
use std::io::{ErrorKind, Read, Write};

fn classify_tls_io_error(error: &std::io::Error) -> &'static str {
    if let Some(rustls_error) = error
        .get_ref()
        .and_then(|source| source.downcast_ref::<rustls::Error>())
    {
        return match rustls_error {
            rustls::Error::InvalidCertificate(_) => SERVICE_TLS_CERTIFICATE_VERIFICATION_FAILED,
            _ => SERVICE_TLS_HANDSHAKE_FAILED,
        };
    }
    if matches!(error.kind(), ErrorKind::TimedOut) {
        return "failed to read service response payload";
    }
    SERVICE_TLS_HANDSHAKE_FAILED
}

fn map_stream_io_error(error: &std::io::Error, default_reason: &'static str) -> SdkError {
    if error
        .get_ref()
        .and_then(|source| source.downcast_ref::<rustls::Error>())
        .is_some()
    {
        return SdkError::TransportFailure(classify_tls_io_error(error));
    }
    SdkError::TransportFailure(default_reason)
}

pub(super) fn write_and_flush_request<W: Write>(
    stream: &mut W,
    payload: &[u8],
    failure_reason: &'static str,
) -> Result<(), SdkError> {
    stream
        .write_all(payload)
        .map_err(|error| map_stream_io_error(&error, failure_reason))?;
    stream
        .flush()
        .map_err(|error| map_stream_io_error(&error, failure_reason))?;
    Ok(())
}

pub(super) fn parse_host_port(
    authority: &str,
    default_port: u16,
) -> Result<(String, u16), SdkError> {
    if authority.starts_with('[') {
        let closing = authority.find(']').ok_or(SdkError::InvalidInput {
            field: "service.endpoint",
            reason: "unterminated ipv6 host",
        })?;
        let host = authority[..=closing].to_owned();
        validate_endpoint_host(host.as_str())?;
        let suffix = &authority[closing + 1..];
        if suffix.is_empty() {
            return Ok((host, default_port));
        }
        let port = suffix
            .strip_prefix(':')
            .ok_or(SdkError::InvalidInput {
                field: "service.endpoint",
                reason: "invalid ipv6 authority suffix",
            })?
            .parse::<u16>()
            .map_err(|_| SdkError::InvalidInput {
                field: "service.endpoint",
                reason: "port must be an unsigned integer in range",
            })?;
        return Ok((host, port));
    }

    match authority.rsplit_once(':') {
        Some((host, raw_port)) if !host.is_empty() => {
            validate_endpoint_host(host)?;
            let port = raw_port
                .parse::<u16>()
                .map_err(|_| SdkError::InvalidInput {
                    field: "service.endpoint",
                    reason: "port must be an unsigned integer in range",
                })?;
            Ok((host.to_owned(), port))
        }
        _ => {
            validate_endpoint_host(authority)?;
            Ok((authority.to_owned(), default_port))
        }
    }
}

pub(super) fn normalize_route_segment(
    field: &'static str,
    value: &str,
) -> Result<String, SdkError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(SdkError::InvalidInput {
            field,
            reason: "must not be empty",
        });
    }
    if normalized.bytes().any(|byte| {
        !matches!(
            byte,
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b':'
        )
    }) {
        return Err(SdkError::InvalidInput {
            field,
            reason: "contains characters not allowed in route segment",
        });
    }
    Ok(normalized.to_owned())
}

pub(super) fn validate_http_header_value(field: &'static str, value: &str) -> Result<(), SdkError> {
    if value.bytes().any(|byte| !matches!(byte, 0x20..=0x7e)) {
        return Err(SdkError::InvalidInput {
            field,
            reason: "contains invalid http header characters",
        });
    }
    Ok(())
}

pub(super) fn validate_endpoint_host(host: &str) -> Result<(), SdkError> {
    if host.trim().is_empty() {
        return Err(SdkError::InvalidInput {
            field: "service.endpoint",
            reason: "host is required",
        });
    }
    if host
        .bytes()
        .any(|byte| byte <= 0x20 || byte == 0x7f || matches!(byte, b'/' | b'\\' | b'?' | b'#'))
    {
        return Err(SdkError::InvalidInput {
            field: "service.endpoint",
            reason: "host contains invalid characters",
        });
    }
    Ok(())
}

pub(super) fn validate_request_method(method: &str) -> Result<(), SdkError> {
    if method.bytes().all(|byte| byte.is_ascii_uppercase()) {
        return Ok(());
    }
    Err(SdkError::InvalidInput {
        field: "service.request_method",
        reason: "contains invalid http method characters",
    })
}

pub(super) fn validate_request_path(path: &str) -> Result<(), SdkError> {
    if !path.starts_with('/') {
        return Err(SdkError::InvalidInput {
            field: "service.request_path",
            reason: "must start with '/'",
        });
    }
    if path
        .bytes()
        .any(|byte| byte <= 0x20 || byte == 0x7f || matches!(byte, b'?' | b'#'))
    {
        return Err(SdkError::InvalidInput {
            field: "service.request_path",
            reason: "contains invalid path characters",
        });
    }
    Ok(())
}

pub(super) fn render_auth_headers(auth: Option<&ServiceRequestAuth>) -> Result<String, SdkError> {
    let Some(auth) = auth else {
        return Ok(String::new());
    };
    validate_http_header_value("request_auth.sender_did", auth.sender_did().as_str())?;
    validate_http_header_value("request_auth.signature", auth.signature())?;
    let mut headers = String::new();
    headers.push_str(
        format!(
            "{REQUEST_AUTH_SENDER_DID_HEADER}: {}\r\n",
            auth.sender_did().as_str()
        )
        .as_str(),
    );
    headers.push_str(format!("{REQUEST_AUTH_NONCE_HEADER}: {}\r\n", auth.nonce()).as_str());
    headers.push_str(format!("{REQUEST_AUTH_SIGNATURE_HEADER}: {}\r\n", auth.signature()).as_str());
    if let Some(signer_public_key_hex) = auth.signer_public_key_hex() {
        validate_http_header_value("request_auth.signer_public_key", signer_public_key_hex)?;
        headers.push_str(
            format!("{REQUEST_AUTH_SIGNER_PUBLIC_KEY_HEADER}: {signer_public_key_hex}\r\n",)
                .as_str(),
        );
    }
    if let Some(scope) = auth.scope() {
        validate_http_header_value("request_auth.scope", scope)?;
        headers.push_str(format!("{REQUEST_AUTH_SCOPE_HEADER}: {scope}\r\n").as_str());
    }
    Ok(headers)
}

pub(super) fn read_response_bytes<R: Read>(stream: &mut R) -> Result<Vec<u8>, SdkError> {
    let mut response = Vec::new();
    let mut chunk = [0_u8; 1024];
    let mut read_iterations = 0_usize;
    loop {
        if read_iterations >= MAX_SERVICE_RESPONSE_READ_ITERATIONS {
            return Err(SdkError::TransportFailure(
                SERVICE_RESPONSE_READ_ITERATION_LIMIT_EXCEEDED,
            ));
        }
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read_count) => {
                read_iterations = read_iterations.saturating_add(1);
                response.extend_from_slice(&chunk[..read_count]);
                if response.len() > MAX_SERVICE_RESPONSE_BYTES {
                    return Err(SdkError::TransportFailure(
                        SERVICE_RESPONSE_SIZE_LIMIT_EXCEEDED,
                    ));
                }
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break;
            }
            Err(error)
                if matches!(
                    error.kind(),
                    ErrorKind::UnexpectedEof | ErrorKind::ConnectionReset
                ) && !response.is_empty() =>
            {
                break;
            }
            Err(error) => {
                return Err(map_stream_io_error(
                    &error,
                    "failed to read service response payload",
                ));
            }
        }
    }
    Ok(response)
}

pub(super) fn read_response_text<R: Read>(stream: &mut R) -> Result<String, SdkError> {
    String::from_utf8(read_response_bytes(stream)?)
        .map_err(|_| SdkError::TransportFailure("service response payload was not utf-8"))
}
