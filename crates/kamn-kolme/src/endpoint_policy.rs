//! Endpoint normalization and validation contracts for Kolme transports.

use std::error::Error;
use std::fmt;

/// Endpoint-policy error used by transport URL normalization contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeEndpointPolicyError {
    /// Endpoint is unavailable because input validation failed.
    Unavailable {
        /// Deterministic validation failure reason.
        reason: String,
    },
}

impl fmt::Display for KolmeEndpointPolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable { reason } => f.write_str(reason),
        }
    }
}

impl Error for KolmeEndpointPolicyError {}

/// HTTP endpoint scheme for runtime-commit transport connections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KolmeHttpScheme {
    /// Plaintext HTTP.
    Http,
    /// TLS-backed HTTPS.
    Https,
}

impl KolmeHttpScheme {
    /// Default port for the given scheme.
    pub fn default_port(self) -> u16 {
        match self {
            Self::Http => 80,
            Self::Https => 443,
        }
    }
}

/// Parsed HTTP endpoint used by runtime-commit transport execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeParsedHttpEndpoint {
    /// Scheme selected from the base URL.
    pub scheme: KolmeHttpScheme,
    /// Host name used by socket dialing.
    pub host: String,
    /// Host header authority value.
    pub host_header: String,
    /// Port selected from authority/defaults.
    pub port: u16,
    /// Fully normalized request path.
    pub target_path: String,
}

/// Parsed websocket endpoint used by notifications consumer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeParsedWebsocketEndpoint {
    /// True when scheme is `wss://`.
    pub secure: bool,
    /// Host name used by socket dialing.
    pub host: String,
    /// Host header authority value.
    pub host_header: String,
    /// Port selected from authority/defaults.
    pub port: u16,
    /// Fully normalized request path.
    pub target_path: String,
}

/// Parses and validates one HTTP endpoint.
pub fn parse_http_endpoint(
    base_url: &str,
    path: &str,
) -> Result<KolmeParsedHttpEndpoint, KolmeEndpointPolicyError> {
    let base = base_url.trim();
    if base.is_empty() {
        return Err(KolmeEndpointPolicyError::Unavailable {
            reason: "base_url must not be empty".to_owned(),
        });
    }
    let (scheme, remainder) = if let Some(remainder) = base.strip_prefix("http://") {
        (KolmeHttpScheme::Http, remainder)
    } else if let Some(remainder) = base.strip_prefix("https://") {
        (KolmeHttpScheme::Https, remainder)
    } else {
        return Err(KolmeEndpointPolicyError::Unavailable {
            reason: "base_url scheme must be http:// or https://".to_owned(),
        });
    };

    let (authority, base_path) = match remainder.split_once('/') {
        Some((left, right)) => (left, format!("/{}", right.trim_start_matches('/'))),
        None => (remainder, "/".to_owned()),
    };

    if authority.is_empty() {
        return Err(KolmeEndpointPolicyError::Unavailable {
            reason: "base_url host must not be empty".to_owned(),
        });
    }

    let (host, port) = parse_authority(authority, scheme.default_port())?;
    let target_path = join_http_paths(base_path.as_str(), path);

    Ok(KolmeParsedHttpEndpoint {
        scheme,
        host,
        host_header: authority.to_owned(),
        port,
        target_path,
    })
}

/// Validates finality checker base URL input before runtime-commit polling.
pub fn is_valid_finality_base_url_input(base_url: &str) -> bool {
    !base_url.trim().is_empty()
}

/// Validates finality checker status-path input before runtime-commit polling.
pub fn is_valid_finality_status_path_input(status_path: &str) -> bool {
    !status_path.trim().is_empty()
}

/// Normalizes finality checker endpoint inputs for deterministic request composition.
pub fn normalize_finality_endpoint_inputs(base_url: &str, status_path: &str) -> (String, String) {
    (base_url.trim().to_owned(), status_path.trim().to_owned())
}

/// Validates live provider base URL input before broadcast submit requests.
pub fn is_valid_live_provider_base_url_input(base_url: &str) -> bool {
    !base_url.trim().is_empty()
}

/// Validates live provider submit path input before broadcast submit requests.
pub fn is_valid_live_provider_submit_path_input(submit_path: &str) -> bool {
    !submit_path.trim().is_empty()
}

/// Composes websocket notifications URL from HTTP base URL + notifications path.
pub fn compose_notifications_websocket_url(
    base_url: &str,
    notifications_path: &str,
) -> Result<String, KolmeEndpointPolicyError> {
    if notifications_path.trim().is_empty() {
        return Err(KolmeEndpointPolicyError::Unavailable {
            reason: "notifications_path must not be empty".to_owned(),
        });
    }
    let endpoint = parse_http_endpoint(base_url, notifications_path)?;
    let scheme = match endpoint.scheme {
        KolmeHttpScheme::Http => "ws",
        KolmeHttpScheme::Https => "wss",
    };
    Ok(format!(
        "{scheme}://{}{}",
        endpoint.host_header, endpoint.target_path
    ))
}

/// Composes one finality status path by appending encoded `commit_id` query.
pub fn compose_finality_status_path(
    status_path: &str,
    commit_id: &str,
) -> Result<String, KolmeEndpointPolicyError> {
    let commit_id = commit_id.trim();
    if commit_id.is_empty() {
        return Err(KolmeEndpointPolicyError::Unavailable {
            reason: "commit_id must not be empty".to_owned(),
        });
    }
    let encoded_commit_id = percent_encode(commit_id);
    let separator = if status_path.contains('?') { "&" } else { "?" };
    Ok(format!(
        "{status_path}{separator}commit_id={encoded_commit_id}"
    ))
}

/// Parses and validates one websocket notifications endpoint URL.
pub fn parse_websocket_endpoint(
    notifications_url: &str,
) -> Result<KolmeParsedWebsocketEndpoint, KolmeEndpointPolicyError> {
    let raw = notifications_url.trim();
    if raw.is_empty() {
        return Err(KolmeEndpointPolicyError::Unavailable {
            reason: "notifications_url must not be empty".to_owned(),
        });
    }

    let (secure, remainder) = if let Some(remainder) = raw.strip_prefix("ws://") {
        (false, remainder)
    } else if let Some(remainder) = raw.strip_prefix("wss://") {
        (true, remainder)
    } else {
        return Err(KolmeEndpointPolicyError::Unavailable {
            reason: "notifications_url scheme must be ws:// or wss://".to_owned(),
        });
    };

    let (authority, target_path) = match remainder.split_once('/') {
        Some((left, right)) => (left, format!("/{}", right.trim_start_matches('/'))),
        None => (remainder, "/".to_owned()),
    };
    if authority.trim().is_empty() {
        return Err(KolmeEndpointPolicyError::Unavailable {
            reason: "notifications_url host must not be empty".to_owned(),
        });
    }
    let default_port = if secure { 443 } else { 80 };
    let (host, port) = parse_authority(authority, default_port)?;

    Ok(KolmeParsedWebsocketEndpoint {
        secure,
        host,
        host_header: authority.to_owned(),
        port,
        target_path,
    })
}

fn parse_authority(
    authority: &str,
    default_port: u16,
) -> Result<(String, u16), KolmeEndpointPolicyError> {
    if authority.starts_with('[') {
        return Err(KolmeEndpointPolicyError::Unavailable {
            reason: "ipv6 host syntax is not supported".to_owned(),
        });
    }
    if let Some((host, port_raw)) = authority.rsplit_once(':') {
        if !port_raw.is_empty() && port_raw.chars().all(|ch| ch.is_ascii_digit()) {
            let port =
                port_raw
                    .parse::<u16>()
                    .map_err(|_| KolmeEndpointPolicyError::Unavailable {
                        reason: "base_url port is invalid".to_owned(),
                    })?;
            if host.trim().is_empty() {
                return Err(KolmeEndpointPolicyError::Unavailable {
                    reason: "base_url host must not be empty".to_owned(),
                });
            }
            return Ok((host.to_owned(), port));
        }
        if !port_raw.is_empty() {
            return Err(KolmeEndpointPolicyError::Unavailable {
                reason: "base_url port is invalid".to_owned(),
            });
        }
    }
    Ok((authority.to_owned(), default_port))
}

fn join_http_paths(base_path: &str, request_path: &str) -> String {
    let base = if base_path.trim().is_empty() {
        "/".to_owned()
    } else if base_path.starts_with('/') {
        base_path.to_owned()
    } else {
        format!("/{base_path}")
    };

    let request = request_path.trim();
    if request.is_empty() || request == "/" {
        return base;
    }

    if request.starts_with('/') {
        if base == "/" {
            return request.to_owned();
        }
        return format!("{}{}", base.trim_end_matches('/'), request);
    }

    if base == "/" {
        format!("/{request}")
    } else {
        format!("{}/{}", base.trim_end_matches('/'), request)
    }
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        let ch = byte as char;
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '~') {
            encoded.push(ch);
        } else {
            encoded.push('%');
            encoded.push_str(format!("{byte:02X}").as_str());
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::{
        compose_finality_status_path, compose_notifications_websocket_url,
        is_valid_finality_base_url_input, is_valid_finality_status_path_input,
        is_valid_live_provider_base_url_input, is_valid_live_provider_submit_path_input,
        parse_http_endpoint, parse_websocket_endpoint, KolmeEndpointPolicyError, KolmeHttpScheme,
    };

    #[test]
    fn unit_parse_http_endpoint_rejects_invalid_scheme() {
        assert_eq!(
            parse_http_endpoint("ftp://kolme.local", "/status"),
            Err(KolmeEndpointPolicyError::Unavailable {
                reason: "base_url scheme must be http:// or https://".to_owned(),
            })
        );
    }

    #[test]
    fn functional_compose_notifications_websocket_url_maps_https_to_wss() {
        let url =
            compose_notifications_websocket_url("https://kolme.local/runtime", "/notifications")
                .expect("notifications url should compose");
        assert_eq!(url, "wss://kolme.local/runtime/notifications");
    }

    #[test]
    fn regression_parse_websocket_endpoint_rejects_non_websocket_scheme() {
        // Regression: #1729
        assert_eq!(
            parse_websocket_endpoint("http://kolme.local/notifications"),
            Err(KolmeEndpointPolicyError::Unavailable {
                reason: "notifications_url scheme must be ws:// or wss://".to_owned(),
            })
        );
    }

    #[test]
    fn unit_parse_http_endpoint_normalizes_host_port_path() {
        let endpoint =
            parse_http_endpoint("https://kolme.local:7443/base", "runtime-commit/status")
                .expect("endpoint should parse");
        assert_eq!(endpoint.scheme, KolmeHttpScheme::Https);
        assert_eq!(endpoint.host, "kolme.local");
        assert_eq!(endpoint.host_header, "kolme.local:7443");
        assert_eq!(endpoint.port, 7443);
        assert_eq!(endpoint.target_path, "/base/runtime-commit/status");
    }

    #[test]
    fn unit_compose_finality_status_path_rejects_empty_commit_id() {
        assert_eq!(
            compose_finality_status_path("/runtime-commit/status", " "),
            Err(KolmeEndpointPolicyError::Unavailable {
                reason: "commit_id must not be empty".to_owned(),
            })
        );
    }

    #[test]
    fn unit_validates_finality_endpoint_inputs() {
        assert!(is_valid_finality_base_url_input("https://kolme.local"));
        assert!(!is_valid_finality_base_url_input(" "));
        assert!(is_valid_finality_status_path_input(
            "/runtime-commit/status"
        ));
        assert!(!is_valid_finality_status_path_input(" "));
    }

    #[test]
    fn unit_validates_live_provider_endpoint_inputs() {
        assert!(is_valid_live_provider_base_url_input("https://kolme.local"));
        assert!(!is_valid_live_provider_base_url_input(" "));
        assert!(is_valid_live_provider_submit_path_input(
            "/broadcast/runtime-commit"
        ));
        assert!(!is_valid_live_provider_submit_path_input(""));
    }
}
