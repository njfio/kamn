use std::collections::BTreeMap;

use kamn_runtime_guards::retention_engine::{
    evaluate_retention_policy, retention_policy_reason_codes_csv,
    retention_policy_reason_taxonomy_version, RetentionClass, RetentionDomain,
    RetentionEnginePolicy, RetentionPolicyCheckerInput, RetentionPolicyDecision,
    RetentionPolicyEngine, RetentionPolicyError, RetentionPolicyViolationReason,
    RetentionRecord,
};

fn base_policy() -> RetentionEnginePolicy {
    RetentionEnginePolicy {
        default_class: RetentionClass::MaxAgeSeconds(300),
        overrides: BTreeMap::from([(RetentionDomain::Messages, RetentionClass::MaxAgeSeconds(60))]),
    }
}

fn checker_input(domain: &str, window_seconds: u64, record_age_seconds: u64) -> RetentionPolicyCheckerInput {
    RetentionPolicyCheckerInput {
        domain: domain.to_owned(),
        window_seconds,
        record_age_seconds,
    }
}

fn record(domain: RetentionDomain, record_id: &str, created_at_secs: u64) -> RetentionRecord {
    RetentionRecord {
        domain,
        record_id: record_id.to_owned(),
        created_at_secs,
    }
}

fn assert_checker_reject(input: RetentionPolicyCheckerInput, reason: RetentionPolicyViolationReason) {
    assert_eq!(
        evaluate_retention_policy(&input),
        RetentionPolicyDecision::Reject { reason }
    );
}

#[test]
fn integration_runtime_guard_retention_checker_rejects_invalid_inputs_and_allows_boundary() {
    assert_checker_reject(
        checker_input("unknown", 60, 1),
        RetentionPolicyViolationReason::DomainUnknown,
    );
    assert_checker_reject(
        checker_input("messages", 0, 1),
        RetentionPolicyViolationReason::WindowNonPositive,
    );
    assert_checker_reject(
        checker_input("messages", 60, 61),
        RetentionPolicyViolationReason::RecordExpired,
    );
    assert_eq!(
        evaluate_retention_policy(&checker_input("messages", 60, 60)),
        RetentionPolicyDecision::Allow
    );
    assert_eq!(
        retention_policy_reason_taxonomy_version(),
        "kamn.runtime.retention-policy-reason-taxonomy.v1"
    );
    assert_eq!(
        retention_policy_reason_codes_csv(),
        "retention_domain_unknown,retention_window_non_positive,retention_record_expired"
    );
}

#[test]
fn integration_runtime_guard_retention_engine_status_uses_default_and_override_classes() {
    let engine = RetentionPolicyEngine::new(base_policy()).expect("engine should construct");

    let message_status = engine
        .status_for(&record(RetentionDomain::Messages, "msg-1", 100))
        .expect("message status should succeed");
    assert_eq!(message_status.class, RetentionClass::MaxAgeSeconds(60));
    assert_eq!(message_status.expires_at_secs, 160);

    let task_status = engine
        .status_for(&record(RetentionDomain::Tasks, "task-1", 100))
        .expect("task status should succeed");
    assert_eq!(task_status.class, RetentionClass::MaxAgeSeconds(300));
    assert_eq!(task_status.expires_at_secs, 400);
}

#[test]
fn integration_runtime_guard_retention_engine_evaluate_returns_deterministic_expired_ids() {
    let mut engine = RetentionPolicyEngine::new(base_policy()).expect("engine should construct");

    let evaluation = engine
        .evaluate(
            500,
            vec![
                record(RetentionDomain::Tasks, "task-2", 250),
                record(RetentionDomain::Messages, "msg-1", 100),
                record(RetentionDomain::Tasks, "task-1", 150),
                record(RetentionDomain::Messages, "msg-2", 460),
            ],
        )
        .expect("evaluation should succeed");

    assert_eq!(evaluation.expired_ids, vec!["msg-1", "task-1"]);
}

#[test]
fn integration_runtime_guard_retention_engine_rejects_resurfaced_expired_record() {
    let mut engine = RetentionPolicyEngine::new(base_policy()).expect("engine should construct");

    let first = engine
        .evaluate(500, vec![record(RetentionDomain::Tasks, "task-1", 100)])
        .expect("first evaluation should succeed");
    assert_eq!(first.expired_ids, vec!["task-1"]);

    assert_eq!(
        engine.evaluate(501, vec![record(RetentionDomain::Tasks, "task-1", 499)]),
        Err(RetentionPolicyError::ResurfacedExpiredRecord(
            "task-1".to_owned()
        ))
    );
}

#[test]
fn integration_runtime_guard_retention_engine_invalid_policy_and_record_fail_closed() {
    assert_eq!(
        RetentionPolicyEngine::new(RetentionEnginePolicy {
            default_class: RetentionClass::MaxAgeSeconds(0),
            overrides: BTreeMap::new(),
        }),
        Err(RetentionPolicyError::InvalidRetentionClass(0))
    );

    let engine = RetentionPolicyEngine::new(base_policy()).expect("engine should construct");
    assert_eq!(
        engine.status_for(&record(RetentionDomain::Tasks, " ", 1)),
        Err(RetentionPolicyError::EmptyField("record_id"))
    );
}
