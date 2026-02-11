//! Notification event parsing contracts for Kolme websocket payloads.

use std::error::Error;
use std::fmt;

/// Typed notification event variants emitted by Kolme websocket streams.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeNotificationEvent {
    /// Finalized transaction notification emitted from a new block event.
    NewBlock {
        /// Transaction hash observed in the block payload.
        txhash: String,
        /// Optional block height where the transaction finalized.
        block_height: Option<u64>,
    },
    /// Failed transaction notification emitted by processor execution path.
    FailedTransaction {
        /// Transaction hash observed in failed-transaction payload.
        txhash: String,
        /// Optional proposed block height for the failed transaction.
        proposed_height: Option<u64>,
    },
    /// Latest block watermark notification.
    LatestBlock {
        /// Latest observed block height.
        height: u64,
    },
}

/// Error raised by notification parsing policy contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeNotificationPolicyError {
    /// Notification payload failed deterministic parse/validation.
    MalformedResponse {
        /// Parse/validation failure reason.
        reason: String,
    },
}

impl fmt::Display for KolmeNotificationPolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedResponse { reason } => f.write_str(reason),
        }
    }
}

impl Error for KolmeNotificationPolicyError {}

/// Parses one notification websocket payload into a typed event contract.
pub fn parse_notification_event(
    payload: &str,
) -> Result<KolmeNotificationEvent, KolmeNotificationPolicyError> {
    let trimmed = payload.trim();
    if trimmed.is_empty() {
        return Err(KolmeNotificationPolicyError::MalformedResponse {
            reason: "notification payload must not be empty".to_owned(),
        });
    }
    if trimmed.contains("\"NewBlock\"") {
        let block_height = find_notification_u64_field(trimmed, "height")?
            .or(find_escaped_notification_u64_field(trimmed, "height")?);
        if let Some(txhash) = find_notification_string_field(trimmed, "txhash")? {
            return Ok(KolmeNotificationEvent::NewBlock {
                txhash,
                block_height,
            });
        }
        if let Some(height) = block_height {
            return Ok(KolmeNotificationEvent::LatestBlock { height });
        }
        return Err(KolmeNotificationPolicyError::MalformedResponse {
            reason: "notification txhash or height field is missing".to_owned(),
        });
    }
    if trimmed.contains("\"FailedTransaction\"") {
        let txhash = find_notification_string_field(trimmed, "txhash")?.ok_or_else(|| {
            KolmeNotificationPolicyError::MalformedResponse {
                reason: "notification txhash field is missing".to_owned(),
            }
        })?;
        let proposed_height = find_notification_u64_field(trimmed, "proposed_height")?;
        return Ok(KolmeNotificationEvent::FailedTransaction {
            txhash,
            proposed_height,
        });
    }
    if trimmed.contains("\"LatestBlock\"") {
        let height = find_notification_u64_field(trimmed, "height")?.ok_or_else(|| {
            KolmeNotificationPolicyError::MalformedResponse {
                reason: "notification latest block height is missing".to_owned(),
            }
        })?;
        return Ok(KolmeNotificationEvent::LatestBlock { height });
    }
    Err(KolmeNotificationPolicyError::MalformedResponse {
        reason: "notification variant is unsupported".to_owned(),
    })
}

fn find_notification_string_field(
    payload: &str,
    field: &str,
) -> Result<Option<String>, KolmeNotificationPolicyError> {
    let pattern = format!("\"{field}\"");
    for (index, _) in payload.match_indices(pattern.as_str()) {
        let mut cursor = index + pattern.len();
        cursor = skip_ascii_whitespace(payload, cursor);
        if payload.as_bytes().get(cursor).copied() != Some(b':') {
            continue;
        }
        cursor += 1;
        cursor = skip_ascii_whitespace(payload, cursor);
        if payload.as_bytes().get(cursor).copied() != Some(b'"') {
            continue;
        }
        let mut end = cursor + 1;
        let mut escape = false;
        while let Some(byte) = payload.as_bytes().get(end).copied() {
            if escape {
                escape = false;
                end += 1;
                continue;
            }
            if byte == b'\\' {
                escape = true;
                end += 1;
                continue;
            }
            if byte == b'"' {
                let token = &payload[cursor..=end];
                let parsed = parse_json_string(token).map_err(|reason| {
                    KolmeNotificationPolicyError::MalformedResponse {
                        reason: format!("notification field '{field}' is invalid: {reason}"),
                    }
                })?;
                if parsed.trim().is_empty() {
                    return Err(KolmeNotificationPolicyError::MalformedResponse {
                        reason: format!("notification field '{field}' must not be empty"),
                    });
                }
                return Ok(Some(parsed));
            }
            end += 1;
        }
        return Err(KolmeNotificationPolicyError::MalformedResponse {
            reason: format!("notification field '{field}' is unterminated"),
        });
    }
    Ok(None)
}

fn find_notification_u64_field(
    payload: &str,
    field: &str,
) -> Result<Option<u64>, KolmeNotificationPolicyError> {
    let pattern = format!("\"{field}\"");
    for (index, _) in payload.match_indices(pattern.as_str()) {
        let mut cursor = index + pattern.len();
        cursor = skip_ascii_whitespace(payload, cursor);
        if payload.as_bytes().get(cursor).copied() != Some(b':') {
            continue;
        }
        cursor += 1;
        cursor = skip_ascii_whitespace(payload, cursor);
        let Some(first) = payload.as_bytes().get(cursor).copied() else {
            return Err(KolmeNotificationPolicyError::MalformedResponse {
                reason: format!("notification field '{field}' value is missing"),
            });
        };

        if first == b'"' {
            let mut end = cursor + 1;
            let mut escape = false;
            while let Some(byte) = payload.as_bytes().get(end).copied() {
                if escape {
                    escape = false;
                    end += 1;
                    continue;
                }
                if byte == b'\\' {
                    escape = true;
                    end += 1;
                    continue;
                }
                if byte == b'"' {
                    let token = &payload[cursor..=end];
                    let parsed = parse_json_string(token).map_err(|reason| {
                        KolmeNotificationPolicyError::MalformedResponse {
                            reason: format!("notification field '{field}' is invalid: {reason}"),
                        }
                    })?;
                    return parse_notification_positive_u64(parsed.as_str(), field).map(Some);
                }
                end += 1;
            }
            return Err(KolmeNotificationPolicyError::MalformedResponse {
                reason: format!("notification field '{field}' is unterminated"),
            });
        }

        let mut end = cursor;
        while let Some(byte) = payload.as_bytes().get(end).copied() {
            if byte.is_ascii_digit() {
                end += 1;
                continue;
            }
            break;
        }
        if end == cursor {
            return Err(KolmeNotificationPolicyError::MalformedResponse {
                reason: format!("notification field '{field}' must be a positive integer"),
            });
        }
        let token = &payload[cursor..end];
        return parse_notification_positive_u64(token, field).map(Some);
    }
    Ok(None)
}

fn find_escaped_notification_u64_field(
    payload: &str,
    field: &str,
) -> Result<Option<u64>, KolmeNotificationPolicyError> {
    let pattern = format!("\\\"{field}\\\":");
    for (index, _) in payload.match_indices(pattern.as_str()) {
        let mut cursor = index + pattern.len();
        cursor = skip_ascii_whitespace(payload, cursor);
        let mut end = cursor;
        while let Some(byte) = payload.as_bytes().get(end).copied() {
            if byte.is_ascii_digit() {
                end += 1;
                continue;
            }
            break;
        }
        if end == cursor {
            continue;
        }
        let token = &payload[cursor..end];
        return parse_notification_positive_u64(token, field).map(Some);
    }
    Ok(None)
}

fn parse_notification_positive_u64(
    token: &str,
    field: &str,
) -> Result<u64, KolmeNotificationPolicyError> {
    let trimmed = token.trim();
    let parsed =
        trimmed
            .parse::<u64>()
            .map_err(|_| KolmeNotificationPolicyError::MalformedResponse {
                reason: format!("notification field '{field}' must be a positive integer"),
            })?;
    if parsed == 0 {
        return Err(KolmeNotificationPolicyError::MalformedResponse {
            reason: format!("notification field '{field}' must be a positive integer"),
        });
    }
    Ok(parsed)
}

fn skip_ascii_whitespace(value: &str, mut cursor: usize) -> usize {
    while let Some(byte) = value.as_bytes().get(cursor).copied() {
        if byte.is_ascii_whitespace() {
            cursor += 1;
            continue;
        }
        break;
    }
    cursor
}

fn parse_json_string(token: &str) -> Result<String, &'static str> {
    let trimmed = token.trim();
    if !(trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2) {
        return Err("token must be a quoted string");
    }
    let mut output = String::new();
    let mut escape = false;
    for ch in trimmed[1..trimmed.len() - 1].chars() {
        if escape {
            let mapped = match ch {
                '\\' => '\\',
                '"' => '"',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                _ => return Err("unsupported escape sequence"),
            };
            output.push(mapped);
            escape = false;
            continue;
        }
        if ch == '\\' {
            escape = true;
            continue;
        }
        output.push(ch);
    }
    if escape {
        return Err("unterminated escape sequence");
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::{parse_notification_event, KolmeNotificationEvent, KolmeNotificationPolicyError};

    #[test]
    fn unit_parse_notification_event_rejects_empty_payload() {
        assert_eq!(
            parse_notification_event(" "),
            Err(KolmeNotificationPolicyError::MalformedResponse {
                reason: "notification payload must not be empty".to_owned(),
            })
        );
    }

    #[test]
    fn functional_parse_notification_event_maps_failed_transaction_variant() {
        assert_eq!(
            parse_notification_event(
                r#"{"event":"FailedTransaction","txhash":"0xdeadbeef","proposed_height":44}"#
            )
            .expect("failed transaction should parse"),
            KolmeNotificationEvent::FailedTransaction {
                txhash: "0xdeadbeef".to_owned(),
                proposed_height: Some(44),
            }
        );
    }
}
