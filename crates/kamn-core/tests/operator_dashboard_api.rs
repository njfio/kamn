use kamn_core::{
    AgentKeyHierarchy, DashboardPageRequest, EscrowLifecycle, MessageLifecycleStore,
    OperatorDashboardApi, OperatorDashboardApiError, ReputationStore, TaskOperationEngine,
};

#[test]
fn operator_dashboard_api_rejects_zero_page_limit() {
    assert_eq!(
        DashboardPageRequest::new(0, None, None),
        Err(OperatorDashboardApiError::InvalidPageLimit(0))
    );
}

#[test]
fn operator_dashboard_api_lists_agents_with_deterministic_pagination_and_filtering() {
    let mut api = OperatorDashboardApi::new();
    let base_hierarchy =
        AgentKeyHierarchy::new("id-1", "sig-1", "agr-1").expect("hierarchy should initialize");
    api.upsert_agent_from_hierarchy("kamn:did:agent:alpha-1", &base_hierarchy)
        .expect("agent should upsert");
    api.upsert_agent_from_hierarchy("kamn:did:agent:alpha-2", &base_hierarchy)
        .expect("agent should upsert");
    api.upsert_agent_from_hierarchy("kamn:did:agent:beta-1", &base_hierarchy)
        .expect("agent should upsert");

    let first = api
        .list_agents(&DashboardPageRequest::new(2, None, None).expect("request should be valid"))
        .expect("list should succeed");
    assert_eq!(first.items.len(), 2);
    assert_eq!(first.items[0].agent_did, "kamn:did:agent:alpha-1");
    assert_eq!(first.items[1].agent_did, "kamn:did:agent:alpha-2");
    assert_eq!(first.next_cursor, Some("kamn:did:agent:alpha-2".to_owned()));

    let second = api
        .list_agents(
            &DashboardPageRequest::new(2, first.next_cursor.clone(), None)
                .expect("request should be valid"),
        )
        .expect("second page should succeed");
    assert_eq!(second.items.len(), 1);
    assert_eq!(second.items[0].agent_did, "kamn:did:agent:beta-1");

    let filtered = api
        .list_agents(
            &DashboardPageRequest::new(10, None, Some("kamn:did:agent:alpha".to_owned()))
                .expect("request should be valid"),
        )
        .expect("filtered list should succeed");
    assert_eq!(filtered.items.len(), 2);
}

#[test]
fn operator_dashboard_api_builds_cross_domain_snapshot_from_core_modules() {
    let mut api = OperatorDashboardApi::new();

    let mut hierarchy =
        AgentKeyHierarchy::new("id-2", "sig-2", "agr-2").expect("hierarchy should initialize");
    hierarchy
        .register_ephemeral("session-1", "ephemeral-1", 1_800_000_000)
        .expect("ephemeral should register");
    api.upsert_agent_from_hierarchy("kamn:did:agent:operator-1", &hierarchy)
        .expect("agent should upsert");

    let mut tasks = TaskOperationEngine::new();
    tasks
        .submit(
            "task-operator-1",
            "kamn:did:agent:operator-1",
            "investigate incident",
        )
        .expect("task should submit");
    api.upsert_task(tasks.task("task-operator-1").expect("task should exist"))
        .expect("task should upsert");

    let mut messages = MessageLifecycleStore::new();
    messages
        .register(
            "urn:uuid:msg-operator-1",
            "kamn:did:agent:operator-1",
            vec!["kamn:did:agent:operator-2".to_owned()],
            "2026-02-08T12:00:00Z",
            "2026-02-08T12:30:00Z",
        )
        .expect("message should register");
    api.upsert_message_from_store(&messages, "urn:uuid:msg-operator-1")
        .expect("message should upsert");

    let mut escrow = EscrowLifecycle::new(100).expect("escrow should initialize");
    escrow.release(40).expect("escrow release should succeed");
    api.upsert_escrow(
        "escrow-operator-1",
        "kamn:did:agent:payer-1",
        "kamn:did:agent:payee-1",
        &escrow,
    )
    .expect("escrow should upsert");

    let mut reputation = ReputationStore::default();
    reputation
        .register_agent("kamn:did:agent:operator-1", 10)
        .expect("agent should register");
    api.upsert_reputation(
        reputation
            .get_agent("kamn:did:agent:operator-1")
            .expect("reputation should exist"),
    )
    .expect("reputation should upsert");

    let snapshot = api
        .snapshot(&DashboardPageRequest::new(10, None, None).expect("request should be valid"))
        .expect("snapshot should build");
    assert_eq!(snapshot.agents.items.len(), 1);
    assert_eq!(snapshot.tasks.items.len(), 1);
    assert_eq!(snapshot.messages.items.len(), 1);
    assert_eq!(snapshot.escrows.items.len(), 1);
    assert_eq!(snapshot.reputation.items.len(), 1);
}

#[test]
fn operator_dashboard_api_regression_rejects_tampered_cursor_tokens() {
    // Regression: #203
    let mut api = OperatorDashboardApi::new();
    let hierarchy =
        AgentKeyHierarchy::new("id-3", "sig-3", "agr-3").expect("hierarchy should initialize");
    api.upsert_agent_from_hierarchy("kamn:did:agent:cursor-1", &hierarchy)
        .expect("agent should upsert");

    assert_eq!(
        api.list_agents(
            &DashboardPageRequest::new(10, Some("tampered-cursor".to_owned()), None)
                .expect("request should be valid"),
        ),
        Err(OperatorDashboardApiError::InvalidPaginationCursor(
            "tampered-cursor".to_owned()
        ))
    );
}
