use kamn_core::{
    AuditDomain, AuditEventRecord, AuditExportEngine, AuditExportError, AuditExportFilter,
    AuditExportFormat, AuditExportRequest,
};
use std::collections::BTreeSet;

fn authorized_exporter() -> &'static str {
    "kamn:did:agent:compliance-operator-1"
}

fn event(
    domain: AuditDomain,
    event_id: &str,
    occurred_at: &str,
    actor: &str,
    action: &str,
    payload_digest: &str,
) -> AuditEventRecord {
    AuditEventRecord {
        domain,
        event_id: event_id.to_owned(),
        occurred_at: occurred_at.to_owned(),
        actor: actor.to_owned(),
        action: action.to_owned(),
        payload_digest: payload_digest.to_owned(),
    }
}

#[test]
fn export_is_deterministic_and_integrity_stable() {
    let mut engine =
        AuditExportEngine::new(vec![authorized_exporter().to_owned()]).expect("engine constructs");

    engine
        .ingest_event(event(
            AuditDomain::Tasks,
            "task-event-2",
            "2026-02-08T14:02:00Z",
            "kamn:did:agent:task-worker-1",
            "TaskCompleted",
            "sha256:task-2",
        ))
        .expect("event ingests");
    engine
        .ingest_event(event(
            AuditDomain::Messages,
            "msg-event-1",
            "2026-02-08T14:01:00Z",
            "kamn:did:agent:messenger-1",
            "MessageSent",
            "sha256:msg-1",
        ))
        .expect("event ingests");

    let request = AuditExportRequest {
        request_id: "export-1".to_owned(),
        requested_by: authorized_exporter().to_owned(),
        requested_at: "2026-02-08T14:03:00Z".to_owned(),
        format: AuditExportFormat::JsonLines,
        filter: AuditExportFilter::default(),
    };

    let first = engine.export(&request).expect("export should succeed");
    let second = engine.export(&request).expect("export should succeed");

    assert_eq!(first.records.len(), 2);
    assert_eq!(first.records[0].event_id, "msg-event-1");
    assert_eq!(first.records[1].event_id, "task-event-2");
    assert_eq!(
        first.manifest.integrity_hash,
        second.manifest.integrity_hash
    );
}

#[test]
fn export_filter_is_respected_across_domains_and_actor_allowlist() {
    let mut engine =
        AuditExportEngine::new(vec![authorized_exporter().to_owned()]).expect("engine constructs");

    engine
        .ingest_event(event(
            AuditDomain::Messages,
            "msg-keep",
            "2026-02-08T15:01:00Z",
            "kamn:did:agent:operator-a",
            "MessageRedacted",
            "sha256:keep",
        ))
        .expect("event ingests");
    engine
        .ingest_event(event(
            AuditDomain::Escrows,
            "escrow-drop",
            "2026-02-08T15:02:00Z",
            "kamn:did:agent:operator-b",
            "EscrowReleased",
            "sha256:drop",
        ))
        .expect("event ingests");

    let request = AuditExportRequest {
        request_id: "export-2".to_owned(),
        requested_by: authorized_exporter().to_owned(),
        requested_at: "2026-02-08T15:03:00Z".to_owned(),
        format: AuditExportFormat::Json,
        filter: AuditExportFilter {
            domains: [AuditDomain::Messages].into_iter().collect(),
            actor_allowlist: ["kamn:did:agent:operator-a".to_owned()]
                .into_iter()
                .collect(),
            from_inclusive: Some("2026-02-08T15:00:00Z".to_owned()),
            to_inclusive: Some("2026-02-08T15:10:00Z".to_owned()),
        },
    };

    let exported = engine.export(&request).expect("export should succeed");
    assert_eq!(exported.records.len(), 1);
    assert_eq!(exported.records[0].event_id, "msg-keep");
}

#[test]
fn integration_multi_domain_pipeline_exports_all_domains() {
    let mut engine =
        AuditExportEngine::new(vec![authorized_exporter().to_owned()]).expect("engine constructs");

    let domains = [
        AuditDomain::Messages,
        AuditDomain::Tasks,
        AuditDomain::Escrows,
        AuditDomain::Reputation,
    ];

    for (index, domain) in domains.iter().enumerate() {
        engine
            .ingest_event(event(
                domain.clone(),
                &format!("event-{index}"),
                &format!("2026-02-08T16:0{}:00Z", index + 1),
                "kamn:did:agent:operator-a",
                "DomainEvent",
                &format!("sha256:{index}"),
            ))
            .expect("event ingests");
    }

    let mut domain_filter = BTreeSet::new();
    domain_filter.extend(domains);

    let request = AuditExportRequest {
        request_id: "export-3".to_owned(),
        requested_by: authorized_exporter().to_owned(),
        requested_at: "2026-02-08T16:10:00Z".to_owned(),
        format: AuditExportFormat::JsonLines,
        filter: AuditExportFilter {
            domains: domain_filter,
            actor_allowlist: BTreeSet::new(),
            from_inclusive: None,
            to_inclusive: None,
        },
    };

    let exported = engine.export(&request).expect("export should succeed");
    assert_eq!(exported.records.len(), 4);
    assert_eq!(exported.manifest.record_count, 4);
}

#[test]
fn regression_blocks_unauthorized_export_request() {
    let mut engine =
        AuditExportEngine::new(vec![authorized_exporter().to_owned()]).expect("engine constructs");
    engine
        .ingest_event(event(
            AuditDomain::Messages,
            "msg-unauth",
            "2026-02-08T17:00:00Z",
            "kamn:did:agent:operator-a",
            "MessageSent",
            "sha256:unauth",
        ))
        .expect("event ingests");

    // Regression: #153
    assert_eq!(
        engine.export(&AuditExportRequest {
            request_id: "export-4".to_owned(),
            requested_by: "kamn:did:agent:unauthorized-1".to_owned(),
            requested_at: "2026-02-08T17:01:00Z".to_owned(),
            format: AuditExportFormat::Json,
            filter: AuditExportFilter::default(),
        }),
        Err(AuditExportError::UnauthorizedExporter(
            "kamn:did:agent:unauthorized-1".to_owned()
        ))
    );
}
