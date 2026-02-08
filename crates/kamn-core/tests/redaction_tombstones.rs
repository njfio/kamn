use kamn_core::{
    canonical_state_key, RedactionAction, RedactionComplianceEngine, RedactionComplianceError,
    RedactionRequestStatus, RedactionVisibility,
};

fn requester() -> &'static str {
    "kamn:did:agent:operator-1"
}

#[test]
fn redaction_requires_quorum_before_application() {
    let mut engine = RedactionComplianceEngine::new(2).expect("engine should construct");

    engine
        .submit_request(
            "redact-1",
            "kamn.messages",
            "msg-42",
            requester(),
            RedactionAction::Redact,
            "contains regulated data",
            "2026-02-08T12:00:00Z",
        )
        .expect("request should be accepted");

    engine
        .approve(
            "redact-1",
            "kamn:did:agent:approver-1",
            "2026-02-08T12:01:00Z",
            "first approval",
        )
        .expect("first approval should succeed");

    assert_eq!(
        engine
            .request_status("redact-1")
            .expect("status should resolve"),
        RedactionRequestStatus::PendingApproval {
            approvals_collected: 1,
            approvals_required: 2,
        }
    );
    assert_eq!(
        engine
            .retrieve_visibility("kamn.messages", "msg-42")
            .expect("visibility should resolve"),
        RedactionVisibility::Available
    );

    engine
        .approve(
            "redact-1",
            "kamn:did:agent:approver-2",
            "2026-02-08T12:02:00Z",
            "second approval",
        )
        .expect("second approval should apply redaction");

    assert_eq!(
        engine
            .request_status("redact-1")
            .expect("status should resolve"),
        RedactionRequestStatus::Applied
    );
    assert_eq!(
        engine
            .retrieve_visibility("kamn.messages", "msg-42")
            .expect("visibility should resolve"),
        RedactionVisibility::Redacted {
            request_id: "redact-1".to_owned(),
        }
    );
}

#[test]
fn tombstone_returns_explicit_retrieval_notice() {
    let mut engine = RedactionComplianceEngine::new(1).expect("engine should construct");

    engine
        .submit_request(
            "tombstone-1",
            "kamn.tasks",
            "task-9",
            requester(),
            RedactionAction::Tombstone,
            "legal erasure request",
            "2026-02-08T12:10:00Z",
        )
        .expect("request should be accepted");
    engine
        .approve(
            "tombstone-1",
            "kamn:did:agent:approver-1",
            "2026-02-08T12:11:00Z",
            "approved",
        )
        .expect("approval should apply tombstone");

    assert_eq!(
        engine
            .retrieve_visibility("kamn.tasks", "task-9")
            .expect("visibility should resolve"),
        RedactionVisibility::Tombstoned {
            request_id: "tombstone-1".to_owned(),
        }
    );
}

#[test]
fn integration_uses_canonical_state_key_for_target_index() {
    let mut engine = RedactionComplianceEngine::new(1).expect("engine should construct");
    engine
        .submit_request(
            "redact-2",
            "kamn.messages",
            "msg-77",
            requester(),
            RedactionAction::Redact,
            "compliance action",
            "2026-02-08T12:20:00Z",
        )
        .expect("request should be accepted");
    engine
        .approve(
            "redact-2",
            "kamn:did:agent:approver-3",
            "2026-02-08T12:21:00Z",
            "approved",
        )
        .expect("approval should apply redaction");

    assert_eq!(
        engine
            .target_storage_key("kamn.messages", "msg-77")
            .expect("key should resolve"),
        canonical_state_key("kamn.messages", "record", "msg-77")
            .expect("canonical key should compute")
    );
}

#[test]
fn regression_prevents_silent_restore_after_tombstone() {
    let mut engine = RedactionComplianceEngine::new(1).expect("engine should construct");
    engine
        .submit_request(
            "tombstone-2",
            "kamn.messages",
            "msg-99",
            requester(),
            RedactionAction::Tombstone,
            "high-severity compliance event",
            "2026-02-08T12:30:00Z",
        )
        .expect("request should be accepted");
    engine
        .approve(
            "tombstone-2",
            "kamn:did:agent:approver-5",
            "2026-02-08T12:31:00Z",
            "approved",
        )
        .expect("approval should apply tombstone");

    // Regression: #151
    assert_eq!(
        engine.submit_request(
            "restore-attempt",
            "kamn.messages",
            "msg-99",
            requester(),
            RedactionAction::Redact,
            "attempt to override tombstone",
            "2026-02-08T12:32:00Z",
        ),
        Err(RedactionComplianceError::TargetAlreadyProtected {
            namespace: "kamn.messages".to_owned(),
            entity_id: "msg-99".to_owned(),
        })
    );
}
