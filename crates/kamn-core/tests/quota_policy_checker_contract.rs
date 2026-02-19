use kamn_core::{
    evaluate_quota_policy, quota_policy_reason_codes_csv, quota_policy_reason_taxonomy_version,
    QuotaPolicyDecision, QuotaPolicyInput, QuotaPolicyViolationReason,
};

const FIXTURE: &str = include_str!("../../../fixtures/runtime/quota_policy_fixture_matrix.txt");

fn fixture_reason_codes_csv() -> String {
    for raw_line in FIXTURE.lines() {
        let line = raw_line.trim();
        if line.starts_with("quota_policy_reason_codes_csv=") {
            let (_, value) = line
                .split_once('=')
                .expect("fixture reason codes marker should be key=value");
            return value.to_owned();
        }
    }
    panic!("quota fixture matrix must declare quota_policy_reason_codes_csv marker");
}

#[test]
fn unit_quota_checker_reason_taxonomy_markers_remain_deterministic() {
    assert_eq!(
        quota_policy_reason_taxonomy_version(),
        "kamn.runtime.quota-policy-reason-taxonomy.v1"
    );
    assert_eq!(
        quota_policy_reason_codes_csv(),
        "quota_scope_unknown,quota_window_non_positive,quota_limit_non_positive,quota_limit_exceeded"
    );
}

#[test]
fn regression_quota_checker_taxonomy_remains_fixture_superset() {
    let fixture_codes = fixture_reason_codes_csv();
    let checker_codes = quota_policy_reason_codes_csv();
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
fn functional_quota_checker_fails_closed_for_invalid_scope_and_bounds() {
    let unknown_scope = QuotaPolicyInput {
        scope: "unknown_scope".to_owned(),
        window_seconds: 60,
        limit: 100,
        observed_count: 1,
    };
    assert_eq!(
        evaluate_quota_policy(&unknown_scope),
        QuotaPolicyDecision::Reject {
            reason: QuotaPolicyViolationReason::ScopeUnknown
        }
    );

    let invalid_window = QuotaPolicyInput {
        scope: "processor_ingress".to_owned(),
        window_seconds: 0,
        limit: 100,
        observed_count: 1,
    };
    assert_eq!(
        evaluate_quota_policy(&invalid_window),
        QuotaPolicyDecision::Reject {
            reason: QuotaPolicyViolationReason::QuotaWindowNonPositive
        }
    );

    let invalid_limit = QuotaPolicyInput {
        scope: "processor_ingress".to_owned(),
        window_seconds: 60,
        limit: 0,
        observed_count: 1,
    };
    assert_eq!(
        evaluate_quota_policy(&invalid_limit),
        QuotaPolicyDecision::Reject {
            reason: QuotaPolicyViolationReason::QuotaLimitNonPositive
        }
    );
}

#[test]
fn integration_quota_checker_handles_limit_exhaustion_deterministically() {
    let allowed_input = QuotaPolicyInput {
        scope: "peer_sync".to_owned(),
        window_seconds: 120,
        limit: 50,
        observed_count: 50,
    };
    assert_eq!(
        evaluate_quota_policy(&allowed_input),
        QuotaPolicyDecision::Allow
    );

    let exceeded_input = QuotaPolicyInput {
        scope: "peer_sync".to_owned(),
        window_seconds: 120,
        limit: 50,
        observed_count: 51,
    };
    assert_eq!(
        evaluate_quota_policy(&exceeded_input),
        QuotaPolicyDecision::Reject {
            reason: QuotaPolicyViolationReason::QuotaLimitExceeded
        }
    );
}
