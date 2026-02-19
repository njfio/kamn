//! Deterministic notifications consumer for Kolme websocket event streams.

use super::{
    compose_kolme_notifications_reconnect_exhausted_reason_contract,
    compose_kolme_notifications_websocket_url,
    is_kolme_valid_notifications_provider_input_contract,
    is_kolme_valid_notifications_reconnect_budget_contract,
    normalize_kolme_notifications_provider_input_contract, parse_kolme_notification_event_contract,
    KolmeRuntimeCommitError, KolmeRuntimeCommitNotificationEvent,
    KolmeRuntimeCommitProviderReceipt,
};
use kamn_kolme::{
    KolmeRuntimeCommitNotificationsConnection, KolmeRuntimeCommitNotificationsConnector,
    KolmeRuntimeCommitProviderError, KolmeRuntimeCommitTransportErrorKind,
};
use std::{thread, time::Duration};

const NOTIFICATIONS_RECONNECT_BASE_BACKOFF_MILLIS: u64 = 10;
const NOTIFICATIONS_RECONNECT_MAX_BACKOFF_MILLIS: u64 = 40;

/// Deterministic notifications consumer for Kolme websocket events with bounded reconnect policy.
pub struct KolmeRuntimeCommitNotificationsConsumer<C>
where
    C: KolmeRuntimeCommitNotificationsConnector,
{
    notifications_url: String,
    provider: String,
    max_reconnect_attempts: u32,
    connector: C,
    connection: Option<C::Connection>,
}

impl<C> KolmeRuntimeCommitNotificationsConsumer<C>
where
    C: KolmeRuntimeCommitNotificationsConnector,
{
    /// Builds notifications consumer from HTTP base URL and notifications path.
    pub fn new(
        base_url: &str,
        notifications_path: &str,
        provider: &str,
        max_reconnect_attempts: u32,
        connector: C,
    ) -> Result<Self, KolmeRuntimeCommitError> {
        if !is_kolme_valid_notifications_provider_input_contract(provider) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "notifications_provider",
                reason: "must not be empty",
            });
        }
        if !is_kolme_valid_notifications_reconnect_budget_contract(max_reconnect_attempts) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "notifications_max_reconnect_attempts",
                reason: "must be positive",
            });
        }
        let notifications_url =
            compose_kolme_notifications_websocket_url(base_url, notifications_path)
                .map_err(|error| KolmeRuntimeCommitProviderError::Unavailable {
                    reason: error.to_string(),
                })
                .map_err(|error| match error {
                    KolmeRuntimeCommitProviderError::Timeout => {
                        KolmeRuntimeCommitError::ProviderTransport {
                            kind: KolmeRuntimeCommitTransportErrorKind::Timeout,
                            detail: "provider request timed out".to_owned(),
                        }
                    }
                    KolmeRuntimeCommitProviderError::Unavailable { reason } => {
                        KolmeRuntimeCommitError::ProviderTransport {
                            kind: KolmeRuntimeCommitTransportErrorKind::Unavailable,
                            detail: reason,
                        }
                    }
                    KolmeRuntimeCommitProviderError::MalformedResponse { reason } => {
                        KolmeRuntimeCommitError::ProviderTransport {
                            kind: KolmeRuntimeCommitTransportErrorKind::MalformedResponse,
                            detail: reason,
                        }
                    }
                })?;
        Ok(Self {
            notifications_url,
            provider: normalize_kolme_notifications_provider_input_contract(provider).to_owned(),
            max_reconnect_attempts,
            connector,
            connection: None,
        })
    }

    pub(crate) fn provider(&self) -> &str {
        self.provider.as_str()
    }

    /// Reads and parses one notifications event, reconnecting when the stream drops.
    pub fn next_notification_event(
        &mut self,
    ) -> Result<KolmeRuntimeCommitNotificationEvent, KolmeRuntimeCommitProviderError> {
        let mut reconnect_attempts = 0_u32;

        loop {
            if self.connection.is_none() {
                match self.connector.connect(self.notifications_url.as_str()) {
                    Ok(connection) => self.connection = Some(connection),
                    Err(_) => {
                        reconnect_attempts += 1;
                        if reconnect_attempts >= self.max_reconnect_attempts {
                            return Err(KolmeRuntimeCommitProviderError::Unavailable {
                                reason:
                                    compose_kolme_notifications_reconnect_exhausted_reason_contract(
                                        self.max_reconnect_attempts,
                                    ),
                            });
                        }
                        maybe_sleep_notifications_reconnect_backoff(reconnect_attempts);
                        continue;
                    }
                }
            }

            let Some(connection) = self.connection.as_mut() else {
                reconnect_attempts += 1;
                if reconnect_attempts >= self.max_reconnect_attempts {
                    return Err(KolmeRuntimeCommitProviderError::Unavailable {
                        reason: compose_kolme_notifications_reconnect_exhausted_reason_contract(
                            self.max_reconnect_attempts,
                        ),
                    });
                }
                maybe_sleep_notifications_reconnect_backoff(reconnect_attempts);
                continue;
            };
            let result = connection.read_text_message();
            match result {
                Ok(Some(payload)) => {
                    let event = parse_kolme_notification_event_contract(payload.as_str()).map_err(
                        |error| KolmeRuntimeCommitProviderError::MalformedResponse {
                            reason: error.to_string(),
                        },
                    )?;
                    return Ok(event.into());
                }
                Ok(None) | Err(_) => {
                    self.connection = None;
                    reconnect_attempts += 1;
                    if reconnect_attempts >= self.max_reconnect_attempts {
                        return Err(KolmeRuntimeCommitProviderError::Unavailable {
                            reason: compose_kolme_notifications_reconnect_exhausted_reason_contract(
                                self.max_reconnect_attempts,
                            ),
                        });
                    }
                    maybe_sleep_notifications_reconnect_backoff(reconnect_attempts);
                }
            }
        }
    }

    /// Reads notification events until one can be mapped to a commit receipt.
    pub fn next_commit_receipt(
        &mut self,
    ) -> Result<KolmeRuntimeCommitProviderReceipt, KolmeRuntimeCommitProviderError> {
        loop {
            let event = self.next_notification_event()?;
            if let Some(receipt) = event.to_provider_receipt(self.provider.as_str()) {
                return Ok(receipt);
            }
        }
    }
}

fn deterministic_notifications_reconnect_backoff_millis(attempt: u32) -> u64 {
    let exponent = attempt.saturating_sub(1).min(4);
    let backoff = NOTIFICATIONS_RECONNECT_BASE_BACKOFF_MILLIS << exponent;
    backoff.min(NOTIFICATIONS_RECONNECT_MAX_BACKOFF_MILLIS)
}

fn maybe_sleep_notifications_reconnect_backoff(attempt: u32) {
    let backoff = deterministic_notifications_reconnect_backoff_millis(attempt);
    thread::sleep(Duration::from_millis(backoff));
}

#[cfg(test)]
mod tests {
    use super::deterministic_notifications_reconnect_backoff_millis;

    #[test]
    fn unit_notifications_reconnect_backoff_schedule_is_deterministic_and_bounded() {
        assert_eq!(deterministic_notifications_reconnect_backoff_millis(1), 10);
        assert_eq!(deterministic_notifications_reconnect_backoff_millis(2), 20);
        assert_eq!(deterministic_notifications_reconnect_backoff_millis(3), 40);
        assert_eq!(deterministic_notifications_reconnect_backoff_millis(8), 40);
    }
}
