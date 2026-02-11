//! TLS policy contracts for Kolme HTTPS transport behavior.

use std::error::Error;
use std::fmt;

/// TLS policy error variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeTlsPolicyError {
    /// Configuration or policy makes TLS path unavailable.
    Unavailable {
        /// Deterministic unavailable reason.
        reason: String,
    },
}

impl fmt::Display for KolmeTlsPolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable { reason } => f.write_str(reason),
        }
    }
}

impl Error for KolmeTlsPolicyError {}

/// Parses optional CA-file environment value into deterministic policy output.
pub fn parse_tls_ca_file_env_value(
    value: Option<&str>,
) -> Result<Option<String>, KolmeTlsPolicyError> {
    let Some(raw) = value else {
        return Ok(None);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(KolmeTlsPolicyError::Unavailable {
            reason: "KAMN_KOLME_TLS_CA_FILE must not be empty".to_owned(),
        });
    }
    Ok(Some(trimmed.to_owned()))
}

/// Resolves one `KAMN_KOLME_TLS_CA_FILE` environment read result into deterministic policy output.
pub fn resolve_tls_ca_file_env_result(
    value: Result<String, std::env::VarError>,
) -> Result<Option<String>, KolmeTlsPolicyError> {
    match value {
        Ok(raw) => parse_tls_ca_file_env_value(Some(raw.as_str())),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(KolmeTlsPolicyError::Unavailable {
            reason: "KAMN_KOLME_TLS_CA_FILE must be valid utf-8".to_owned(),
        }),
    }
}

/// Classifies stderr output from TLS-backed transport into deterministic reason text.
pub fn classify_tls_failure_reason(stderr: &str) -> String {
    let normalized = stderr.to_ascii_lowercase();
    if normalized.contains("certificate verify failed")
        || normalized.contains("self-signed certificate")
        || normalized.contains("unable to get local issuer certificate")
        || normalized.contains("unable to verify the first certificate")
    {
        return "tls certificate verification failed".to_owned();
    }
    if normalized.contains("handshake failure")
        || normalized.contains("wrong version number")
        || normalized.contains("tlsv")
        || normalized.contains("ssl routines")
    {
        return "tls handshake failed".to_owned();
    }
    let first_line = stderr
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("tls request failed");
    format!("tls request failed: {}", first_line.trim())
}

#[cfg(test)]
mod tests {
    use super::{
        classify_tls_failure_reason, parse_tls_ca_file_env_value, resolve_tls_ca_file_env_result,
        KolmeTlsPolicyError,
    };

    #[test]
    fn unit_parse_tls_ca_file_env_value_accepts_trimmed_path() {
        assert_eq!(
            parse_tls_ca_file_env_value(Some(" /etc/ssl/custom.pem ")),
            Ok(Some("/etc/ssl/custom.pem".to_owned()))
        );
    }

    #[test]
    fn unit_parse_tls_ca_file_env_value_rejects_empty_value() {
        assert_eq!(
            parse_tls_ca_file_env_value(Some("  ")),
            Err(KolmeTlsPolicyError::Unavailable {
                reason: "KAMN_KOLME_TLS_CA_FILE must not be empty".to_owned(),
            })
        );
    }

    #[test]
    fn functional_classify_tls_failure_reason_detects_handshake_pattern() {
        assert_eq!(
            classify_tls_failure_reason("ssl routines:ssl3_get_record:wrong version number"),
            "tls handshake failed"
        );
    }

    #[test]
    fn functional_resolve_tls_ca_file_env_result_handles_not_present_and_valid_values() {
        assert_eq!(
            resolve_tls_ca_file_env_result(Err(std::env::VarError::NotPresent)),
            Ok(None)
        );
        assert_eq!(
            resolve_tls_ca_file_env_result(Ok("/etc/ssl/custom.pem".to_owned())),
            Ok(Some("/etc/ssl/custom.pem".to_owned()))
        );
    }

    #[test]
    fn regression_tls_env_resolution_rejects_non_utf8_env_value() {
        // Regression: #1850
        assert_eq!(
            resolve_tls_ca_file_env_result(Err(std::env::VarError::NotUnicode(
                std::ffi::OsString::from("invalid"),
            ))),
            Err(KolmeTlsPolicyError::Unavailable {
                reason: "KAMN_KOLME_TLS_CA_FILE must be valid utf-8".to_owned(),
            })
        );
    }
}
