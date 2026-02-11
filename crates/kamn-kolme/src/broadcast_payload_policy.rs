//! `/broadcast` payload normalization policy contracts.

use crate::{
    parse_flat_json_value_fields, parse_provider_key_value_fields, required_json_string_field,
    required_positive_u64_json_field, required_provider_response_field,
    validate_direct_signed_transaction_message, KolmeApiBroadcastRequest, KolmeApiCodecError,
    KolmeFlatJsonPolicyError, KolmeFlatJsonValue, KolmeProviderOutcomePolicyError,
    KolmeProviderResponsePolicyError,
};
use std::error::Error;
use std::fmt;

/// Error raised by broadcast payload normalization policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeBroadcastPayloadPolicyError {
    /// Payload failed deterministic parse/validation.
    MalformedResponse {
        /// Parse/validation failure reason.
        reason: String,
    },
}

impl fmt::Display for KolmeBroadcastPayloadPolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedResponse { reason } => f.write_str(reason),
        }
    }
}

impl Error for KolmeBroadcastPayloadPolicyError {}

/// Normalizes runtime commit wire payload into canonical Kolme `/broadcast` JSON payload.
pub fn normalize_broadcast_payload(
    wire_payload: &str,
    idempotency_key: &str,
) -> Result<String, KolmeBroadcastPayloadPolicyError> {
    let payload = wire_payload.trim();
    if payload.is_empty() {
        return Err(KolmeBroadcastPayloadPolicyError::MalformedResponse {
            reason: "wire_payload must not be empty".to_owned(),
        });
    }
    let idempotency_key = idempotency_key.trim();
    if idempotency_key.is_empty() {
        return Err(KolmeBroadcastPayloadPolicyError::MalformedResponse {
            reason: "idempotency_key must not be empty".to_owned(),
        });
    }

    if payload.starts_with('{') {
        let fields = parse_flat_json_value_fields(payload).map_err(map_flat_json_error)?;
        if let Some(payload_idempotency_key) = fields.get("idempotency_key") {
            let payload_idempotency_key = match payload_idempotency_key {
                KolmeFlatJsonValue::String(value) => value.trim(),
                _ => {
                    return Err(KolmeBroadcastPayloadPolicyError::MalformedResponse {
                        reason: "field must be a string: idempotency_key".to_owned(),
                    });
                }
            };
            if payload_idempotency_key.is_empty() {
                return Err(KolmeBroadcastPayloadPolicyError::MalformedResponse {
                    reason: "field must not be empty: idempotency_key".to_owned(),
                });
            }
            if payload_idempotency_key != idempotency_key {
                return Err(KolmeBroadcastPayloadPolicyError::MalformedResponse {
                    reason: "wire_payload idempotency_key does not match transport idempotency key"
                        .to_owned(),
                });
            }
        }

        let message =
            required_json_string_field(&fields, "message").map_err(map_flat_json_error)?;
        let signature =
            required_json_string_field(&fields, "signature").map_err(map_flat_json_error)?;
        let recovery_id_u64 = required_positive_u64_json_field(&fields, "recovery_id")
            .map_err(map_flat_json_error)?;
        let recovery_id = u8::try_from(recovery_id_u64).map_err(|_| {
            KolmeBroadcastPayloadPolicyError::MalformedResponse {
                reason: "recovery_id must be within u8 range".to_owned(),
            }
        })?;

        if fields.contains_key("signer_key_id") {
            let _signer_key_id = required_json_string_field(&fields, "signer_key_id")
                .map_err(map_flat_json_error)?;

            let message_fields = parse_provider_key_value_fields(message.as_str())
                .map_err(map_provider_response_error)?;
            let message_idempotency_key =
                required_provider_response_field(&message_fields, "idempotency_key")
                    .map_err(map_provider_outcome_error)?;
            if message_idempotency_key != idempotency_key {
                return Err(KolmeBroadcastPayloadPolicyError::MalformedResponse {
                    reason:
                        "signed message idempotency_key does not match transport idempotency key"
                            .to_owned(),
                });
            }

            let request =
                KolmeApiBroadcastRequest::new(message.as_str(), signature.as_str(), recovery_id)
                    .map_err(map_codec_error)?;
            return Ok(request.to_json_payload());
        }

        if !message.trim().starts_with('{') || !message.trim().ends_with('}') {
            return Err(KolmeBroadcastPayloadPolicyError::MalformedResponse {
                reason: "direct signed payload message must be a JSON object string".to_owned(),
            });
        }
        validate_direct_signed_transaction_message(message.as_str()).map_err(map_codec_error)?;

        let request =
            KolmeApiBroadcastRequest::new(message.as_str(), signature.as_str(), recovery_id)
                .map_err(map_codec_error)?;
        return Ok(request.to_json_payload());
    }

    let fields = parse_provider_key_value_fields(payload).map_err(map_provider_response_error)?;
    if let Some(payload_idempotency_key) = fields.get("idempotency_key") {
        let payload_idempotency_key = payload_idempotency_key.trim();
        if payload_idempotency_key != idempotency_key {
            return Err(KolmeBroadcastPayloadPolicyError::MalformedResponse {
                reason: "wire_payload idempotency_key does not match transport idempotency key"
                    .to_owned(),
            });
        }
    }

    let request =
        KolmeApiBroadcastRequest::new(payload, idempotency_key, 1).map_err(map_codec_error)?;
    Ok(request.to_json_payload())
}

fn map_flat_json_error(error: KolmeFlatJsonPolicyError) -> KolmeBroadcastPayloadPolicyError {
    match error {
        KolmeFlatJsonPolicyError::MalformedResponse { reason } => {
            KolmeBroadcastPayloadPolicyError::MalformedResponse { reason }
        }
    }
}

fn map_provider_response_error(
    error: KolmeProviderResponsePolicyError,
) -> KolmeBroadcastPayloadPolicyError {
    match error {
        KolmeProviderResponsePolicyError::MalformedResponse { reason } => {
            KolmeBroadcastPayloadPolicyError::MalformedResponse { reason }
        }
    }
}

fn map_provider_outcome_error(
    error: KolmeProviderOutcomePolicyError,
) -> KolmeBroadcastPayloadPolicyError {
    match error {
        KolmeProviderOutcomePolicyError::MalformedResponse { reason } => {
            KolmeBroadcastPayloadPolicyError::MalformedResponse { reason }
        }
    }
}

fn map_codec_error(error: KolmeApiCodecError) -> KolmeBroadcastPayloadPolicyError {
    match error {
        KolmeApiCodecError::InvalidRequest { field, reason } => {
            KolmeBroadcastPayloadPolicyError::MalformedResponse {
                reason: format!("invalid request {field}: {reason}"),
            }
        }
        KolmeApiCodecError::MalformedResponse { reason } => {
            KolmeBroadcastPayloadPolicyError::MalformedResponse { reason }
        }
    }
}
