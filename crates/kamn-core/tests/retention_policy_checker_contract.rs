use kamn_core::{
    evaluate_retention_policy, retention_policy_reason_codes_csv,
    retention_policy_reason_taxonomy_version, RetentionPolicyCheckerInput, RetentionPolicyDecision,
    RetentionPolicyViolationReason,
};

const FIXTURE: &str = include_str!("../../../fixtures/runtime/retention_policy_fixture_matrix.txt");

fn fixture_reason_codes_csv() -> String {
    for raw_line in FIXTURE.lines() {
        let line = raw_line.trim();
        if line.starts_with("retention_policy_reason_codes_csv=") {
            let (_, value) = line
                .split_once('=')
                .expect("fixture reason codes marker should be key=value");
            return value.to_owned();
        }
    }
    panic!("retention fixture matrix must declare retention_policy_reason_codes_csv marker");
}

#[test]
fn unit_retention_checker_reason_taxonomy_markers_remain_deterministic() {
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
fn regression_retention_checker_taxonomy_remains_fixture_superset() {
    let fixture_codes = fixture_reason_codes_csv();
    let checker_codes = retention_policy_reason_codes_csv();
    for code in fixture_codes.split(',') {
        assert!(
            checker_codes
                .split(',')
                .any(|checker_code| checker_code == code),
            "checker taxonomy missing fixture reason code: {code}"
        );
    }
}

#[test]
fn functional_retention_checker_fails_closed_for_unknown_domain_and_invalid_window() {
    let unknown_domain = RetentionPolicyCheckerInput {
        domain: "unknown_domain".to_owned(),
        window_seconds: 300,
        record_age_seconds: 10,
    };
    assert_eq!(
        evaluate_retention_policy(&unknown_domain),
        RetentionPolicyDecision::Reject {
            reason: RetentionPolicyViolationReason::DomainUnknown
        }
    );

    let invalid_window = RetentionPolicyCheckerInput {
        domain: "messages".to_owned(),
        window_seconds: 0,
        record_age_seconds: 10,
    };
    assert_eq!(
        evaluate_retention_policy(&invalid_window),
        RetentionPolicyDecision::Reject {
            reason: RetentionPolicyViolationReason::WindowNonPositive
        }
    );
}

#[test]
fn integration_retention_checker_handles_expiration_deterministically() {
    let within_window = RetentionPolicyCheckerInput {
        domain: "tasks".to_owned(),
        window_seconds: 600,
        record_age_seconds: 600,
    };
    assert_eq!(
        evaluate_retention_policy(&within_window),
        RetentionPolicyDecision::Allow
    );

    let expired = RetentionPolicyCheckerInput {
        domain: "tasks".to_owned(),
        window_seconds: 600,
        record_age_seconds: 601,
    };
    assert_eq!(
        evaluate_retention_policy(&expired),
        RetentionPolicyDecision::Reject {
            reason: RetentionPolicyViolationReason::RecordExpired
        }
    );
}
