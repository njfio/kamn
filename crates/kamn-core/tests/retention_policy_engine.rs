use kamn_core::{
    RetentionClass, RetentionDomain, RetentionEnginePolicy, RetentionPolicyEngine,
    RetentionPolicyError, RetentionRecord, RetentionStatus,
};
use std::collections::BTreeMap;

fn policy() -> RetentionEnginePolicy {
    let mut overrides = BTreeMap::new();
    overrides.insert(
        RetentionDomain::Messages,
        RetentionClass::MaxAgeSeconds(300),
    );
    overrides.insert(
        RetentionDomain::Escrows,
        RetentionClass::MaxAgeSeconds(1200),
    );
    RetentionEnginePolicy {
        default_class: RetentionClass::MaxAgeSeconds(600),
        overrides,
    }
}

#[test]
fn domain_override_controls_expiration_window() {
    let mut engine = RetentionPolicyEngine::new(policy()).expect("engine should construct");

    let expired = engine
        .evaluate(
            1_000,
            vec![
                RetentionRecord {
                    domain: RetentionDomain::Messages,
                    record_id: "msg-old".to_owned(),
                    created_at_secs: 200,
                },
                RetentionRecord {
                    domain: RetentionDomain::Tasks,
                    record_id: "task-fresh".to_owned(),
                    created_at_secs: 600,
                },
            ],
        )
        .expect("evaluation should succeed");

    assert_eq!(expired.expired_ids, vec!["msg-old".to_owned()]);
}

#[test]
fn expiration_order_is_deterministic() {
    let mut engine = RetentionPolicyEngine::new(policy()).expect("engine should construct");

    let expired = engine
        .evaluate(
            10_000,
            vec![
                RetentionRecord {
                    domain: RetentionDomain::Tasks,
                    record_id: "task-b".to_owned(),
                    created_at_secs: 100,
                },
                RetentionRecord {
                    domain: RetentionDomain::Tasks,
                    record_id: "task-a".to_owned(),
                    created_at_secs: 100,
                },
                RetentionRecord {
                    domain: RetentionDomain::Messages,
                    record_id: "msg-a".to_owned(),
                    created_at_secs: 100,
                },
            ],
        )
        .expect("evaluation should succeed");

    assert_eq!(
        expired.expired_ids,
        vec!["msg-a".to_owned(), "task-a".to_owned(), "task-b".to_owned(),]
    );
}

#[test]
fn integration_status_surface_reports_domain_class_and_expiry() {
    let engine = RetentionPolicyEngine::new(policy()).expect("engine should construct");

    let status = engine
        .status_for(&RetentionRecord {
            domain: RetentionDomain::Escrows,
            record_id: "escrow-1".to_owned(),
            created_at_secs: 100,
        })
        .expect("status should resolve");

    assert_eq!(
        status,
        RetentionStatus {
            domain: RetentionDomain::Escrows,
            record_id: "escrow-1".to_owned(),
            class: RetentionClass::MaxAgeSeconds(1200),
            expires_at_secs: 1300,
        }
    );
}

#[test]
fn regression_expired_record_cannot_resurface() {
    let mut engine = RetentionPolicyEngine::new(policy()).expect("engine should construct");

    let first = engine
        .evaluate(
            2_000,
            vec![RetentionRecord {
                domain: RetentionDomain::Messages,
                record_id: "msg-resurface".to_owned(),
                created_at_secs: 100,
            }],
        )
        .expect("evaluation should succeed");
    assert_eq!(first.expired_ids, vec!["msg-resurface".to_owned()]);

    // Regression: #155
    assert_eq!(
        engine.evaluate(
            2_100,
            vec![RetentionRecord {
                domain: RetentionDomain::Messages,
                record_id: "msg-resurface".to_owned(),
                created_at_secs: 2_050,
            }],
        ),
        Err(RetentionPolicyError::ResurfacedExpiredRecord(
            "msg-resurface".to_owned()
        ))
    );
}
