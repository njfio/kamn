use super::super::*;

const GRANT_STATUS_ACTIVE: &str = "active";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceApiAuthorizationRequest<'a> {
    pub(crate) correlation_id: &'a str,
    pub(crate) actor_did: &'a str,
    pub(crate) resource: &'a str,
    pub(crate) action: &'a str,
    pub(crate) role: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceApiAuthorizationDecision {
    pub(crate) allowed: bool,
    pub(crate) reason_code: &'static str,
}

impl ServiceApiMessageStore {
    pub(crate) fn authorize_transaction_action(
        &mut self,
        request: ServiceApiAuthorizationRequest<'_>,
    ) -> Result<ServiceApiAuthorizationDecision, String> {
        self.refresh_from_disk()?;
        let decision = evaluate_authorization(&self.snapshot, &request);
        append_receipt(&mut self.snapshot, &request, &decision);
        self.persist()?;
        Ok(decision)
    }

    pub(crate) fn provision_agent_grant(
        &mut self,
        grant: ServiceApiPersistedAgentGrantRecord,
    ) -> Result<(), String> {
        self.refresh_from_disk()?;
        validate_grant(&grant)?;
        if let Some(existing) = self.snapshot.agent_grants.get(&grant.idempotency_key) {
            return identical_or_conflict(existing, &grant);
        }
        self.snapshot
            .agent_grants
            .insert(grant.idempotency_key.clone(), grant);
        self.persist()
    }

    pub(crate) fn revoke_agent_grant(&mut self, idempotency_key: &str) -> Result<(), String> {
        self.refresh_from_disk()?;
        let grant = self
            .snapshot
            .agent_grants
            .get_mut(idempotency_key)
            .ok_or_else(|| format!("agent grant not found: {idempotency_key}"))?;
        if grant.status == "revoked" {
            return Ok(());
        }
        grant.status = "revoked".to_owned();
        self.persist()
    }
}

fn evaluate_authorization(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
    request: &ServiceApiAuthorizationRequest<'_>,
) -> ServiceApiAuthorizationDecision {
    if !actor_is_registered(snapshot, request.actor_did) {
        return denied(REASON_CODE_AGENT_NOT_REGISTERED);
    }
    if let Some(grant) = exact_grant(snapshot, request) {
        if grant.status == GRANT_STATUS_ACTIVE {
            return ServiceApiAuthorizationDecision {
                allowed: true,
                reason_code: REASON_CODE_ACTION_AUTHORIZED,
            };
        }
        return denied(REASON_CODE_ACTION_NOT_GRANTED);
    }
    if actor_has_action_grant(snapshot, request) {
        return denied(REASON_CODE_RESOURCE_ROLE_MISMATCH);
    }
    denied(REASON_CODE_ACTION_NOT_GRANTED)
}

fn actor_is_registered(snapshot: &ServiceApiPersistedMessageStoreSnapshot, did: &str) -> bool {
    snapshot
        .agents
        .get(did)
        .is_some_and(|agent| agent.registered)
}

fn exact_grant<'a>(
    snapshot: &'a ServiceApiPersistedMessageStoreSnapshot,
    request: &ServiceApiAuthorizationRequest<'_>,
) -> Option<&'a ServiceApiPersistedAgentGrantRecord> {
    snapshot.agent_grants.values().find(|grant| {
        grant.did == request.actor_did
            && grant.action == request.action
            && grant.resource == request.resource
            && grant.role == request.role
    })
}

fn actor_has_action_grant(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
    request: &ServiceApiAuthorizationRequest<'_>,
) -> bool {
    snapshot
        .agent_grants
        .values()
        .any(|grant| grant.did == request.actor_did && grant.action == request.action)
}

fn denied(reason_code: &'static str) -> ServiceApiAuthorizationDecision {
    ServiceApiAuthorizationDecision {
        allowed: false,
        reason_code,
    }
}

fn append_receipt(
    snapshot: &mut ServiceApiPersistedMessageStoreSnapshot,
    request: &ServiceApiAuthorizationRequest<'_>,
    decision: &ServiceApiAuthorizationDecision,
) {
    let sequence = snapshot.authorization_receipts.len() + 1;
    snapshot
        .authorization_receipts
        .push(ServiceApiAuthorizationReceiptRecord {
            receipt_id: format!("authorization-receipt-{sequence:08}"),
            correlation_id: request.correlation_id.to_owned(),
            actor_did: request.actor_did.to_owned(),
            resource: request.resource.to_owned(),
            action: request.action.to_owned(),
            role: request.role.to_owned(),
            decision: if decision.allowed { "allow" } else { "deny" }.to_owned(),
            reason_code: decision.reason_code.to_owned(),
        });
}

fn validate_grant(grant: &ServiceApiPersistedAgentGrantRecord) -> Result<(), String> {
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
    if grant.resource == "*" {
        return Err("agent grant generic wildcard resource is forbidden".to_owned());
    }
    if !matches!(grant.status.as_str(), "active" | "revoked") {
        return Err(format!("agent grant status is invalid: {}", grant.status));
    }
    Ok(())
}

fn identical_or_conflict(
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
