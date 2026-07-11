use super::super::super::*;

pub(super) fn validate_grant(grant: &ServiceApiPersistedAgentGrantRecord) -> Result<(), String> {
    validate_non_empty_fields(grant)?;
    AgentDid::parse(grant.did.as_str())
        .map_err(|error| format!("agent grant did is invalid: {error}"))?;
    if !matches!(grant.status.as_str(), "active" | "revoked") {
        return Err(format!("agent grant status is invalid: {}", grant.status));
    }
    validate_action_role_resource(grant)
}

pub(super) fn validate_persisted_grants(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
) -> Result<(), String> {
    for (key, grant) in &snapshot.agent_grants {
        validate_grant(grant)?;
        if key != &grant.idempotency_key {
            return Err(format!(
                "agent grant key does not match idempotency key: {key}"
            ));
        }
    }
    validate_semantic_uniqueness(snapshot)
}

#[cfg(test)]
pub(super) fn reject_semantic_duplicate(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
    requested: &ServiceApiPersistedAgentGrantRecord,
) -> Result<(), String> {
    if snapshot.agent_grants.values().any(|existing| {
        existing.idempotency_key != requested.idempotency_key
            && semantic_grants_match(existing, requested)
    }) {
        return Err("duplicate semantic grant authority is forbidden".to_owned());
    }
    Ok(())
}

fn validate_semantic_uniqueness(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
) -> Result<(), String> {
    let grants: Vec<_> = snapshot.agent_grants.values().collect();
    for (index, grant) in grants.iter().enumerate() {
        if grants[index + 1..]
            .iter()
            .any(|candidate| semantic_grants_match(grant, candidate))
        {
            return Err("duplicate semantic grant authority is forbidden".to_owned());
        }
    }
    Ok(())
}

fn semantic_grants_match(
    left: &ServiceApiPersistedAgentGrantRecord,
    right: &ServiceApiPersistedAgentGrantRecord,
) -> bool {
    left.did == right.did
        && left.resource == right.resource
        && left.role == right.role
        && left.action == right.action
}

#[cfg(test)]
pub(super) fn identical_or_conflict(
    existing: &ServiceApiPersistedAgentGrantRecord,
    requested: &ServiceApiPersistedAgentGrantRecord,
) -> Result<(), String> {
    if existing == requested {
        return Ok(());
    }
    Err(format!(
        "agent grant idempotency key conflict: {}",
        requested.idempotency_key
    ))
}

fn validate_non_empty_fields(grant: &ServiceApiPersistedAgentGrantRecord) -> Result<(), String> {
    let fields = [
        grant.did.as_str(),
        grant.resource.as_str(),
        grant.role.as_str(),
        grant.action.as_str(),
        grant.status.as_str(),
        grant.idempotency_key.as_str(),
    ];
    if fields.iter().any(|field| field.trim().is_empty()) {
        return Err("agent grant fields must not be empty".to_owned());
    }
    Ok(())
}

fn validate_action_role_resource(
    grant: &ServiceApiPersistedAgentGrantRecord,
) -> Result<(), String> {
    let valid = match grant.action.as_str() {
        "task:create" => grant.role == "initiator" && grant.resource == "transaction:new",
        "task:read" => grant.role == "participant" && task_resource_is_valid(&grant.resource),
        "task:accept" | "task:complete" => {
            grant.role == "provider" && task_resource_is_valid(&grant.resource)
        }
        "escrow:fund" => grant.role == "initiator" && task_resource_is_valid(&grant.resource),
        "escrow:release" => {
            grant.role == "initiator" && resource_id_is_valid(&grant.resource, "escrow:")
        }
        _ => false,
    };
    if valid {
        return Ok(());
    }
    Err(format!(
        "agent grant action, role, and resource are incompatible: {}|{}|{}",
        grant.action, grant.role, grant.resource
    ))
}

fn task_resource_is_valid(resource: &str) -> bool {
    resource_id_is_valid(resource, "task:")
}

fn resource_id_is_valid(resource: &str, prefix: &str) -> bool {
    resource
        .strip_prefix(prefix)
        .is_some_and(|id| !id.is_empty() && !id.contains('/') && id != "*")
}
