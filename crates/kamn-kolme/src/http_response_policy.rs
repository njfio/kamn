//! HTTP response parsing policy contracts for Kolme transport paths.

use std::error::Error;
use std::fmt;

/// HTTP response policy error variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeHttpResponsePolicyError {
    /// Response status indicates timeout.
    Timeout,
    /// Response is unavailable due to upstream/client classification.
    Unavailable {
        /// Deterministic failure reason.
        reason: String,
    },
    /// Response shape/content is malformed.
    Malformed {
        /// Deterministic malformed reason.
        reason: String,
    },
}

impl fmt::Display for KolmeHttpResponsePolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout => f.write_str("provider request timed out"),
            Self::Unavailable { reason } => f.write_str(reason),
            Self::Malformed { reason } => f.write_str(reason),
        }
    }
}

impl Error for KolmeHttpResponsePolicyError {}

/// Parses raw HTTP response bytes and returns body for success statuses.
pub fn parse_http_response_body(
    response_bytes: Vec<u8>,
) -> Result<String, KolmeHttpResponsePolicyError> {
    let response_text = String::from_utf8(response_bytes).map_err(|error| {
        KolmeHttpResponsePolicyError::Malformed {
            reason: format!("http response body is not valid utf-8: {error}"),
        }
    })?;

    let (raw_headers, raw_body) = response_text.split_once("\r\n\r\n").ok_or_else(|| {
        KolmeHttpResponsePolicyError::Malformed {
            reason: "http response missing header/body separator".to_owned(),
        }
    })?;

    let mut header_lines = raw_headers.lines();
    let status_line =
        header_lines
            .next()
            .ok_or_else(|| KolmeHttpResponsePolicyError::Malformed {
                reason: "http response missing status line".to_owned(),
            })?;
    let mut status_parts = status_line.split_whitespace();
    let _http_version =
        status_parts
            .next()
            .ok_or_else(|| KolmeHttpResponsePolicyError::Malformed {
                reason: "http response status line is malformed".to_owned(),
            })?;
    let status_code_raw =
        status_parts
            .next()
            .ok_or_else(|| KolmeHttpResponsePolicyError::Malformed {
                reason: "http response status code is missing".to_owned(),
            })?;
    let status_code =
        status_code_raw
            .parse::<u16>()
            .map_err(|_| KolmeHttpResponsePolicyError::Malformed {
                reason: format!("http response status code is invalid: {status_code_raw}"),
            })?;

    if status_code == 408 || status_code == 504 {
        return Err(KolmeHttpResponsePolicyError::Timeout);
    }
    if status_code >= 500 {
        return Err(KolmeHttpResponsePolicyError::Unavailable {
            reason: format!("http response status indicates upstream failure: {status_code}"),
        });
    }
    if status_code >= 400 {
        return Err(map_http_client_status_error(status_code));
    }

    let mut declared_content_length = None;
    for line in header_lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if name.eq_ignore_ascii_case("Content-Length") {
            let parsed = value.trim().parse::<usize>().map_err(|_| {
                KolmeHttpResponsePolicyError::Malformed {
                    reason: "http response content-length is invalid".to_owned(),
                }
            })?;
            declared_content_length = Some(parsed);
            break;
        }
    }
    if let Some(declared) = declared_content_length {
        let observed = raw_body.len();
        if declared != observed {
            return Err(KolmeHttpResponsePolicyError::Malformed {
                reason: format!(
                    "http response content-length mismatch: declared {declared}, observed {observed}"
                ),
            });
        }
    }

    Ok(raw_body.to_owned())
}

fn map_http_client_status_error(status_code: u16) -> KolmeHttpResponsePolicyError {
    match status_code {
        401 | 403 => KolmeHttpResponsePolicyError::Unavailable {
            reason: format!("http response status indicates authorization failure: {status_code}"),
        },
        400 | 404 | 409 | 422 => KolmeHttpResponsePolicyError::Malformed {
            reason: format!("http response status indicates invalid request: {status_code}"),
        },
        429 => KolmeHttpResponsePolicyError::Unavailable {
            reason: format!("http response status indicates rate limited: {status_code}"),
        },
        _ => KolmeHttpResponsePolicyError::Unavailable {
            reason: format!("http response status indicates client failure: {status_code}"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_http_response_body, KolmeHttpResponsePolicyError};

    #[test]
    fn unit_parse_http_response_body_classifies_timeout_status() {
        let error = parse_http_response_body(b"HTTP/1.1 504 Gateway Timeout\r\n\r\n".to_vec())
            .expect_err("gateway timeout must map to timeout");
        assert_eq!(error, KolmeHttpResponsePolicyError::Timeout);
    }

    #[test]
    fn unit_parse_http_response_body_rejects_invalid_content_length() {
        let error = parse_http_response_body(
            b"HTTP/1.1 200 OK\r\nContent-Length: abc\r\n\r\nhello".to_vec(),
        )
        .expect_err("invalid content-length must fail");
        assert_eq!(
            error,
            KolmeHttpResponsePolicyError::Malformed {
                reason: "http response content-length is invalid".to_owned(),
            }
        );
    }
}
