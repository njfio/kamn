use kamn_core::{
    DashboardAttentionLevel, DashboardPage, MessageStatus, OperatorActionAuditRecord,
    OperatorActionOutcome, OperatorBindingAction, OperatorDashboardSnapshot, OperatorDashboardUi,
    OperatorDashboardUiError, OperatorEscrowView, OperatorMessageView, OperatorReputationView,
    OperatorTaskView, TaskState,
};

fn page<T>(items: Vec<T>) -> DashboardPage<T> {
    DashboardPage {
        items,
        next_cursor: None,
    }
}

#[test]
fn operator_dashboard_ui_rejects_empty_message_recipients() {
    let snapshot = OperatorDashboardSnapshot {
        agents: page(vec![]),
        tasks: page(vec![]),
        messages: page(vec![OperatorMessageView {
            message_id: "urn:uuid:msg-empty-recipient".to_owned(),
            sender: "kamn:did:agent:operator-1".to_owned(),
            recipients: vec![],
            status: MessageStatus::Rejected,
        }]),
        escrows: page(vec![]),
        reputation: page(vec![]),
    };

    let ui = OperatorDashboardUi::new();
    assert_eq!(
        ui.compose(&snapshot, &[]),
        Err(OperatorDashboardUiError::EmptyMessageRecipients(
            "urn:uuid:msg-empty-recipient".to_owned()
        ))
    );
}

#[test]
fn operator_dashboard_ui_projects_all_mvp_sections() {
    let snapshot = OperatorDashboardSnapshot {
        agents: page(vec![
            kamn_core::OperatorAgentView {
                agent_did: "kamn:did:agent:beta-2".to_owned(),
                identity_key_id: "id-beta".to_owned(),
                signing_key_id: "sig-beta".to_owned(),
                agreement_key_id: "agr-beta".to_owned(),
            },
            kamn_core::OperatorAgentView {
                agent_did: "kamn:did:agent:alpha-1".to_owned(),
                identity_key_id: "id-alpha".to_owned(),
                signing_key_id: "sig-alpha".to_owned(),
                agreement_key_id: "agr-alpha".to_owned(),
            },
        ]),
        tasks: page(vec![
            OperatorTaskView {
                task_id: "task-b".to_owned(),
                requester: "kamn:did:agent:alpha-1".to_owned(),
                assignee: Some("kamn:did:agent:beta-2".to_owned()),
                state: TaskState::Blocked,
            },
            OperatorTaskView {
                task_id: "task-a".to_owned(),
                requester: "kamn:did:agent:alpha-1".to_owned(),
                assignee: Some("kamn:did:agent:beta-2".to_owned()),
                state: TaskState::Completed,
            },
        ]),
        messages: page(vec![
            OperatorMessageView {
                message_id: "urn:uuid:msg-2".to_owned(),
                sender: "kamn:did:agent:alpha-1".to_owned(),
                recipients: vec!["kamn:did:agent:beta-2".to_owned()],
                status: MessageStatus::Validated,
            },
            OperatorMessageView {
                message_id: "urn:uuid:msg-1".to_owned(),
                sender: "kamn:did:agent:alpha-1".to_owned(),
                recipients: vec!["kamn:did:agent:beta-2".to_owned()],
                status: MessageStatus::Rejected,
            },
        ]),
        escrows: page(vec![OperatorEscrowView {
            escrow_id: "escrow-1".to_owned(),
            payer: "kamn:did:agent:payer-1".to_owned(),
            payee: "kamn:did:agent:payee-1".to_owned(),
            status: kamn_core::EscrowStatus::Disputed,
            remaining_amount: 35,
        }]),
        reputation: page(vec![OperatorReputationView {
            agent_did: "kamn:did:agent:beta-2".to_owned(),
            trust_score: 220,
            delivery_rate: 0.74,
            dispute_rate: 0.31,
        }]),
    };
    let audit = vec![OperatorActionAuditRecord {
        agent_did: "kamn:did:agent:alpha-1".to_owned(),
        operator_did: "kamn:did:human:ops-1".to_owned(),
        action: OperatorBindingAction::Configure,
        target: "maintenance_mode".to_owned(),
        value: Some("enabled".to_owned()),
        requested_at_unix: 1_716_200_100,
        outcome: OperatorActionOutcome::Allowed,
    }];

    let ui = OperatorDashboardUi::new();
    let model = ui
        .compose(&snapshot, &audit)
        .expect("ui composition should succeed");

    assert_eq!(model.agent_list.len(), 2);
    assert_eq!(model.task_timeline.len(), 2);
    assert_eq!(model.message_traces.len(), 2);
    assert_eq!(model.escrow_status.len(), 1);
    assert_eq!(model.reputation_overview.len(), 1);
    assert_eq!(model.audit_traces.len(), 1);
    assert_eq!(model.summary.total_agents, 2);
    assert_eq!(model.summary.blocked_tasks, 1);
    assert_eq!(model.summary.failed_messages, 1);
    assert_eq!(model.summary.disputed_escrows, 1);
    assert_eq!(
        model.message_traces[0].attention,
        DashboardAttentionLevel::Critical
    );
}

#[test]
fn operator_dashboard_ui_integration_surfaces_denied_audit_traces() {
    let snapshot = OperatorDashboardSnapshot {
        agents: page(vec![]),
        tasks: page(vec![]),
        messages: page(vec![]),
        escrows: page(vec![]),
        reputation: page(vec![]),
    };
    let audit = vec![
        OperatorActionAuditRecord {
            agent_did: "kamn:did:agent:ops-1".to_owned(),
            operator_did: "kamn:did:human:ops-1".to_owned(),
            action: OperatorBindingAction::ReadHistory,
            target: "audit_log".to_owned(),
            value: None,
            requested_at_unix: 1_716_200_200,
            outcome: OperatorActionOutcome::Allowed,
        },
        OperatorActionAuditRecord {
            agent_did: "kamn:did:agent:ops-1".to_owned(),
            operator_did: "kamn:did:human:ops-2".to_owned(),
            action: OperatorBindingAction::Configure,
            target: "maintenance_mode".to_owned(),
            value: Some("enabled".to_owned()),
            requested_at_unix: 1_716_200_300,
            outcome: OperatorActionOutcome::Denied,
        },
    ];

    let ui = OperatorDashboardUi::new();
    let model = ui
        .compose(&snapshot, &audit)
        .expect("ui composition should succeed");
    assert_eq!(model.audit_traces.len(), 2);
    assert_eq!(model.summary.denied_operator_actions, 1);
    assert_eq!(model.audit_traces[0].operator_did, "kamn:did:human:ops-2");
    assert_eq!(
        model.audit_traces[0].attention,
        DashboardAttentionLevel::Critical
    );
}

#[test]
fn operator_dashboard_ui_regression_orders_audit_traces_newest_first() {
    // Regression: #201
    let snapshot = OperatorDashboardSnapshot {
        agents: page(vec![]),
        tasks: page(vec![]),
        messages: page(vec![]),
        escrows: page(vec![]),
        reputation: page(vec![]),
    };
    let audit = vec![
        OperatorActionAuditRecord {
            agent_did: "kamn:did:agent:ops-1".to_owned(),
            operator_did: "kamn:did:human:ops-1".to_owned(),
            action: OperatorBindingAction::ReadHistory,
            target: "audit_log".to_owned(),
            value: None,
            requested_at_unix: 3,
            outcome: OperatorActionOutcome::Allowed,
        },
        OperatorActionAuditRecord {
            agent_did: "kamn:did:agent:ops-1".to_owned(),
            operator_did: "kamn:did:human:ops-2".to_owned(),
            action: OperatorBindingAction::Configure,
            target: "feature_x".to_owned(),
            value: Some("on".to_owned()),
            requested_at_unix: 9,
            outcome: OperatorActionOutcome::Denied,
        },
        OperatorActionAuditRecord {
            agent_did: "kamn:did:agent:ops-1".to_owned(),
            operator_did: "kamn:did:human:ops-3".to_owned(),
            action: OperatorBindingAction::Configure,
            target: "feature_y".to_owned(),
            value: Some("on".to_owned()),
            requested_at_unix: 6,
            outcome: OperatorActionOutcome::Allowed,
        },
    ];

    let ui = OperatorDashboardUi::new();
    let model = ui
        .compose(&snapshot, &audit)
        .expect("ui composition should succeed");
    let ordered: Vec<u64> = model
        .audit_traces
        .iter()
        .map(|entry| entry.requested_at_unix)
        .collect();
    assert_eq!(ordered, vec![9, 6, 3]);
}
