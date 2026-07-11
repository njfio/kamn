use super::message_store::ServiceApiPersistedAgentGrantRecord;
use super::*;

pub(crate) fn provision_test_transaction_grant(
    state_file: String,
    actor_did: &str,
    method: &str,
    path: &str,
    body: &str,
) -> Result<(), String> {
    let target = resolve_target(actor_did, method, path, body)?;
    let mut store = ServiceApiMessageStore::from_optional_state_file(Some(state_file))?;
    ensure_registered(&mut store, actor_did)?;
    if active_grant_exists(&store, &target) {
        return Ok(());
    }
    store.provision_agent_grant(grant_record(&target))
}

fn active_grant_exists(
    store: &ServiceApiMessageStore,
    target: &auth::TransactionAuthorizationTarget,
) -> bool {
    store.snapshot.agent_grants.values().any(|grant| {
        grant.did == target.actor_did
            && grant.resource == target.resource
            && grant.action == target.action
            && grant.role == target.role
            && grant.status == "active"
    })
}

fn resolve_target(
    actor_did: &str,
    method: &str,
    path: &str,
    body: &str,
) -> Result<auth::TransactionAuthorizationTarget, String> {
    let parsed = ParsedRequest {
        method: method.to_owned(),
        path: path.to_owned(),
        body: body.to_owned(),
        headers: BTreeMap::from([(
            REQUEST_AUTH_SENDER_DID_HEADER.to_owned(),
            actor_did.to_owned(),
        )]),
    };
    auth::resolve_transaction_authorization_target(&parsed)
        .map_err(|error| error.message)?
        .ok_or_else(|| format!("test authorization target is not a transaction route: {path}"))
}

fn ensure_registered(store: &mut ServiceApiMessageStore, actor_did: &str) -> Result<(), String> {
    if store
        .snapshot
        .agents
        .get(actor_did)
        .is_some_and(|agent| agent.registered)
    {
        return Ok(());
    }
    let registration = ServiceApiAgentRegisterRequestBody {
        agent_type: "test-agent".to_owned(),
        model_family: "test-runtime".to_owned(),
        capabilities: vec!["transaction-test".to_owned()],
    };
    store
        .register_agent_profile(actor_did, &registration)
        .map(|_| ())
        .map_err(|error| format!("test agent registration failed: {error:?}"))
}

fn grant_record(
    target: &auth::TransactionAuthorizationTarget,
) -> ServiceApiPersistedAgentGrantRecord {
    let idempotency_key = format!(
        "test-grant:{}:{}:{}:{}",
        target.actor_did, target.resource, target.action, target.role
    );
    ServiceApiPersistedAgentGrantRecord {
        did: target.actor_did.clone(),
        resource: target.resource.clone(),
        role: target.role.to_owned(),
        action: target.action.to_owned(),
        status: "active".to_owned(),
        idempotency_key,
    }
}
