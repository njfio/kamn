use super::super::*;

const ACTOR_DID: &str = "kamn:did:agent:grant-contract";
const GRANT_ID: &str = "grant-contract-create";

#[test]
fn unit_agent_grant_provisioning_is_idempotent_and_rejects_conflicts() {
    let mut store = registered_store();
    let grant = task_create_grant();

    store
        .provision_agent_grant(grant.clone())
        .expect("first grant should persist");
    store
        .provision_agent_grant(grant)
        .expect("identical grant should be idempotent");
    let error = store
        .provision_agent_grant(conflicting_grant())
        .expect_err("conflicting idempotency key must fail");

    assert_eq!(store.snapshot.agent_grants.len(), 1);
    assert!(error.contains("idempotency key conflict"));
}

#[test]
fn unit_agent_grant_provisioning_rejects_duplicate_semantic_authority() {
    let mut store = registered_store();
    store
        .provision_agent_grant(task_create_grant())
        .expect("first grant should persist");
    let mut duplicate = task_create_grant();
    duplicate.idempotency_key = "grant-contract-create-duplicate".to_owned();

    let error = store
        .provision_agent_grant(duplicate)
        .expect_err("duplicate semantic grant must fail");

    assert_eq!(store.snapshot.agent_grants.len(), 1);
    assert!(error.contains("duplicate semantic grant"));
}

#[test]
fn unit_revoked_grant_revalidation_fails_closed_and_is_idempotent() {
    let mut store = registered_store();
    store
        .provision_agent_grant(task_create_grant())
        .expect("grant should persist");
    assert!(authorize(&mut store).allowed);

    store
        .revoke_agent_grant(GRANT_ID)
        .expect("grant should revoke");
    store
        .revoke_agent_grant(GRANT_ID)
        .expect("repeated revoke should be idempotent");

    let decision = revalidate(&mut store);
    assert!(!decision.allowed);
    assert_eq!(decision.reason_code, REASON_CODE_ACTION_NOT_GRANTED);
}

#[test]
fn unit_agent_grant_validation_rejects_generic_or_incompatible_authority() {
    let mut store = registered_store();
    let mut wildcard = task_create_grant();
    wildcard.resource = "*".to_owned();
    let mut wrong_role = task_create_grant();
    wrong_role.idempotency_key = "grant-contract-wrong-role".to_owned();
    wrong_role.role = "verifier".to_owned();

    assert!(store.provision_agent_grant(wildcard).is_err());
    assert!(store.provision_agent_grant(wrong_role).is_err());
    assert!(store.snapshot.agent_grants.is_empty());
}

fn registered_store() -> ServiceApiMessageStore {
    let mut store = ServiceApiMessageStore {
        state_file: None,
        audit_export_file: None,
        snapshot: ServiceApiPersistedMessageStoreSnapshot::default(),
    };
    let registration = ServiceApiAgentRegisterRequestBody {
        agent_type: "test-agent".to_owned(),
        model_family: "test-runtime".to_owned(),
        capabilities: vec!["task".to_owned()],
    };
    store
        .register_agent_profile(ACTOR_DID, &registration)
        .expect("actor should register");
    store
}

fn task_create_grant() -> ServiceApiPersistedAgentGrantRecord {
    ServiceApiPersistedAgentGrantRecord {
        did: ACTOR_DID.to_owned(),
        resource: "transaction:new".to_owned(),
        role: "initiator".to_owned(),
        action: "task:create".to_owned(),
        status: "active".to_owned(),
        idempotency_key: GRANT_ID.to_owned(),
    }
}

fn conflicting_grant() -> ServiceApiPersistedAgentGrantRecord {
    ServiceApiPersistedAgentGrantRecord {
        action: "task:complete".to_owned(),
        role: "provider".to_owned(),
        resource: "task:other".to_owned(),
        ..task_create_grant()
    }
}

fn authorize(store: &mut ServiceApiMessageStore) -> ServiceApiAuthorizationDecision {
    store
        .authorize_transaction_action(request())
        .expect("authorization should evaluate")
}

fn revalidate(store: &mut ServiceApiMessageStore) -> ServiceApiAuthorizationDecision {
    store
        .revalidate_transaction_action(request())
        .expect("revalidation should evaluate")
}

fn request() -> ServiceApiAuthorizationRequest<'static> {
    ServiceApiAuthorizationRequest {
        correlation_id: "authz-test",
        actor_did: ACTOR_DID,
        resource: "transaction:new",
        action: "task:create",
        role: "initiator",
    }
}
