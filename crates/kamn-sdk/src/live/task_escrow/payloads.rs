use super::aliases::validate_service_id;
use crate::{Artifact, EscrowConfig, SdkError, TaskDefinition};
use std::fmt::Write as _;

pub(crate) fn task_payload(task: &TaskDefinition) -> Result<String, SdkError> {
    if task.task_type.trim().is_empty() {
        return Err(SdkError::InvalidInput {
            field: "task_type",
            reason: "must not be empty",
        });
    }
    if task.description.trim().is_empty() {
        return Err(SdkError::InvalidInput {
            field: "description",
            reason: "must not be empty",
        });
    }

    Ok(format!(
        "{{\"creator\":\"{}\",\"task_type\":\"{}\",\"description\":\"{}\"}}",
        escape_json(task.creator.as_str()),
        escape_json(task.task_type.as_str()),
        escape_json(task.description.as_str()),
    ))
}

pub(crate) fn escrow_payload(escrow: &EscrowConfig) -> Result<String, SdkError> {
    if escrow.amount.0 == 0 {
        return Err(SdkError::InvalidInput {
            field: "escrow.amount",
            reason: "must be greater than zero",
        });
    }

    Ok(format!(
        "{{\"payer\":\"{}\",\"payee\":\"{}\",\"amount\":{}}}",
        escape_json(escrow.payer.as_str()),
        escape_json(escrow.payee.as_str()),
        escrow.amount.0,
    ))
}

pub(crate) fn artifact_payload(
    service_task_id: &str,
    artifact: &Artifact,
) -> Result<String, SdkError> {
    validate_service_id("task", service_task_id)?;
    validate_artifact(artifact)?;
    Ok(format!(
        "{{\"task_id\":\"{}\",\"artifact_name\":\"{}\",\"artifact_bytes_hex\":\"{}\"}}",
        escape_json(service_task_id),
        escape_json(artifact.name.as_str()),
        hex_encode(artifact.bytes.as_slice()),
    ))
}

fn validate_artifact(artifact: &Artifact) -> Result<(), SdkError> {
    if artifact.name.trim().is_empty() {
        return Err(SdkError::InvalidInput {
            field: "artifact.name",
            reason: "must not be empty",
        });
    }
    if artifact.bytes.is_empty() {
        return Err(SdkError::InvalidInput {
            field: "artifact.bytes",
            reason: "must not be empty",
        });
    }
    Ok(())
}

pub(crate) fn escape_json(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{0008}' => escaped.push_str("\\b"),
            '\u{000C}' => escaped.push_str("\\f"),
            value if value.is_control() => {
                let _ = write!(escaped, "\\u{:04x}", value as u32);
            }
            value => escaped.push(value),
        }
    }
    escaped
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}
