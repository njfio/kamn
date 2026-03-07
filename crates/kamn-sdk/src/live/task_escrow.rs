use crate::{AgentDid, EscrowConfig, EscrowId, SdkError, TaskDefinition, TaskId};
use std::collections::HashMap;
use std::fmt::Write as _;

#[derive(Debug, Clone)]
pub(crate) struct LiveTaskAlias {
    pub(crate) service_id: String,
    pub(crate) creator: AgentDid,
    pub(crate) assignee: Option<AgentDid>,
}

#[derive(Debug, Clone)]
pub(crate) struct LiveEscrowAlias {
    pub(crate) service_id: String,
    pub(crate) payer: AgentDid,
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

pub(crate) fn deterministic_u64_tag(value: &str) -> u64 {
    let mut acc: u64 = 0xcbf29ce484222325;
    for byte in value.as_bytes() {
        acc ^= u64::from(*byte);
        acc = acc.wrapping_mul(0x00000100000001B3);
    }
    acc
}

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

pub(crate) fn remember_task_alias(
    task_aliases: &mut HashMap<u64, LiveTaskAlias>,
    service_task_id: &str,
    creator: &AgentDid,
) -> Result<TaskId, SdkError> {
    remember_service_id("task", service_task_id)?;
    let alias = deterministic_u64_tag(service_task_id);
    match task_aliases.get(&alias) {
        Some(existing) if existing.service_id != service_task_id => {
            return Err(SdkError::Conflict(
                "service task id collision detected in sdk task alias map",
            ));
        }
        Some(_) => {}
        None => {
            task_aliases.insert(
                alias,
                LiveTaskAlias {
                    service_id: service_task_id.to_owned(),
                    creator: creator.clone(),
                    assignee: None,
                },
            );
        }
    }
    Ok(TaskId(alias))
}

pub(crate) fn prepare_task_accept(
    task_aliases: &mut HashMap<u64, LiveTaskAlias>,
    task_id: &TaskId,
    assignee: &AgentDid,
) -> Result<(String, AgentDid), SdkError> {
    let entry = task_aliases
        .get_mut(&task_id.0)
        .ok_or_else(|| SdkError::NotFound {
            entity: "task",
            id: task_id.0.to_string(),
        })?;
    entry.assignee = Some(assignee.clone());
    Ok((entry.service_id.clone(), assignee.clone()))
}

pub(crate) fn prepare_task_complete(
    task_aliases: &HashMap<u64, LiveTaskAlias>,
    task_id: &TaskId,
) -> Result<(String, AgentDid), SdkError> {
    let entry = task_aliases
        .get(&task_id.0)
        .ok_or_else(|| SdkError::NotFound {
            entity: "task",
            id: task_id.0.to_string(),
        })?;
    Ok((
        entry.service_id.clone(),
        entry
            .assignee
            .clone()
            .unwrap_or_else(|| entry.creator.clone()),
    ))
}

pub(crate) fn remember_escrow_alias(
    escrow_aliases: &mut HashMap<u64, LiveEscrowAlias>,
    service_escrow_id: &str,
    payer: &AgentDid,
) -> Result<EscrowId, SdkError> {
    remember_service_id("escrow", service_escrow_id)?;
    let alias = deterministic_u64_tag(service_escrow_id);
    match escrow_aliases.get(&alias) {
        Some(existing) if existing.service_id != service_escrow_id => {
            return Err(SdkError::Conflict(
                "service escrow id collision detected in sdk escrow alias map",
            ));
        }
        Some(_) => {}
        None => {
            escrow_aliases.insert(
                alias,
                LiveEscrowAlias {
                    service_id: service_escrow_id.to_owned(),
                    payer: payer.clone(),
                },
            );
        }
    }
    Ok(EscrowId(alias))
}

pub(crate) fn prepare_escrow_release(
    escrow_aliases: &HashMap<u64, LiveEscrowAlias>,
    escrow_id: &EscrowId,
) -> Result<(String, AgentDid), SdkError> {
    let entry = escrow_aliases
        .get(&escrow_id.0)
        .ok_or_else(|| SdkError::NotFound {
            entity: "escrow",
            id: escrow_id.0.to_string(),
        })?;
    Ok((entry.service_id.clone(), entry.payer.clone()))
}

fn remember_service_id(entity: &'static str, service_id: &str) -> Result<(), SdkError> {
    if service_id.trim().is_empty() {
        return Err(SdkError::TransportFailure(match entity {
            "task" => "service returned empty task_id in task response",
            "escrow" => "service returned empty escrow_id in escrow response",
            _ => "service returned empty id in response",
        }));
    }
    Ok(())
}
