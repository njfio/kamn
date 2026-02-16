use std::thread;
use std::time::Duration;

use kamn_core::{
    ConfigError, KolmeApiNextNonceRequest, KolmeRuntimeCommitHttpTransport,
    KolmeRuntimeCommitProviderError,
};

use crate::{logging::log_warn, KOLME_LIVE_NONCE_PATH};

const KOLME_LIVE_NONCE_RETRY_MAX_ATTEMPTS: u32 = 3;
const KOLME_LIVE_NONCE_RETRY_BASE_BACKOFF_MILLIS: u64 = 10;
const KOLME_LIVE_NONCE_RETRY_MAX_BACKOFF_MILLIS: u64 = 40;

pub(crate) fn resolve_kolme_live_nonce(
    base_url: &str,
    transport: &mut KolmeRuntimeCommitHttpTransport,
    pubkey: &str,
) -> Result<u64, ConfigError> {
    let request = KolmeApiNextNonceRequest::new(pubkey)
        .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
    let nonce_correlation_id = format!("kolme.live.nonce:{pubkey}");
    let mut attempt = 0_u32;
    loop {
        attempt += 1;
        match transport.fetch_next_nonce(base_url, KOLME_LIVE_NONCE_PATH, &request) {
            Ok(response) => return Ok(response.next_nonce),
            Err(error) => {
                if let Some(reason_code) = classify_nonce_retry_category(&error) {
                    if attempt < KOLME_LIVE_NONCE_RETRY_MAX_ATTEMPTS {
                        let attempt_label = attempt.to_string();
                        let max_attempts_label = KOLME_LIVE_NONCE_RETRY_MAX_ATTEMPTS.to_string();
                        log_warn(
                            "kolme.live.nonce.retry",
                            &[
                                ("correlation_id", nonce_correlation_id.as_str()),
                                ("pubkey", pubkey),
                                ("attempt", attempt_label.as_str()),
                                ("max_attempts", max_attempts_label.as_str()),
                                ("reason", reason_code),
                                ("reason_code", reason_code),
                            ],
                        )?;
                        maybe_sleep_nonce_retry_backoff(attempt);
                        continue;
                    }
                }
                return Err(map_nonce_provider_error(error));
            }
        }
    }
}

pub(super) fn classify_nonce_retry_category(
    error: &KolmeRuntimeCommitProviderError,
) -> Option<&'static str> {
    match error {
        KolmeRuntimeCommitProviderError::Timeout => Some("timeout"),
        KolmeRuntimeCommitProviderError::Unavailable { .. } => Some("unavailable"),
        KolmeRuntimeCommitProviderError::MalformedResponse { .. } => None,
    }
}

fn map_nonce_provider_error(error: KolmeRuntimeCommitProviderError) -> ConfigError {
    match error {
        KolmeRuntimeCommitProviderError::Timeout => {
            ConfigError::RuntimeKolmeLive("nonce request timed out".to_owned())
        }
        KolmeRuntimeCommitProviderError::Unavailable { reason } => {
            ConfigError::RuntimeKolmeLive(format!("nonce request unavailable: {reason}"))
        }
        KolmeRuntimeCommitProviderError::MalformedResponse { reason } => {
            ConfigError::RuntimeKolmeLive(format!("nonce response malformed: {reason}"))
        }
    }
}

pub(super) fn deterministic_nonce_retry_backoff_millis(attempt: u32) -> u64 {
    let exponent = attempt.saturating_sub(1).min(4);
    let multiplier = 1_u64 << exponent;
    (KOLME_LIVE_NONCE_RETRY_BASE_BACKOFF_MILLIS * multiplier)
        .min(KOLME_LIVE_NONCE_RETRY_MAX_BACKOFF_MILLIS)
}

fn maybe_sleep_nonce_retry_backoff(attempt: u32) {
    let backoff = deterministic_nonce_retry_backoff_millis(attempt);
    thread::sleep(Duration::from_millis(backoff));
}
