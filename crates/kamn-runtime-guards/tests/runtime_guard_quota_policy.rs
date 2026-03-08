use kamn_runtime_guards::quota_policy::{
    evaluate_quota_policy, quota_policy_reason_codes_csv, quota_policy_reason_taxonomy_version,
    QuotaPolicyDecision, QuotaPolicyInput, QuotaPolicyViolationReason,
};

fn valid_input(scope: &str) -> QuotaPolicyInput {
    QuotaPolicyInput {
        scope: scope.to_owned(),
        window_seconds: 60,
        limit: 5,
        observed_count: 3,
    }
}

#[test]
fn integration_runtime_guard_quota_policy_allows_all_supported_scope_classes() {
    for scope in ["processor_ingress", "peer_sync", "channel_broadcast"] {
        assert_eq!(evaluate_quota_policy(&valid_input(scope)), QuotaPolicyDecision::Allow);
    }
}

#[test]
fn integration_runtime_guard_quota_policy_rejects_invalid_inputs_with_deterministic_reasons() {
    let cases = [
        (
            QuotaPolicyInput {
                scope: "unknown".to_owned(),
                ..valid_input("processor_ingress")
            },
            QuotaPolicyViolationReason::ScopeUnknown,
        ),
        (
            QuotaPolicyInput {
                window_seconds: 0,
                ..valid_input("processor_ingress")
            },
            QuotaPolicyViolationReason::QuotaWindowNonPositive,
        ),
        (
            QuotaPolicyInput {
                limit: 0,
                ..valid_input("processor_ingress")
            },
            QuotaPolicyViolationReason::QuotaLimitNonPositive,
        ),
        (
            QuotaPolicyInput {
                observed_count: 6,
                ..valid_input("processor_ingress")
            },
            QuotaPolicyViolationReason::QuotaLimitExceeded,
        ),
    ];

    for (input, reason) in cases {
        assert_eq!(
            evaluate_quota_policy(&input),
            QuotaPolicyDecision::Reject { reason }
        );
    }
}

#[test]
fn integration_runtime_guard_quota_policy_allows_limit_boundary_without_mutating_input() {
    let input = QuotaPolicyInput {
        observed_count: 5,
        ..valid_input("processor_ingress")
    };
    let original = input.clone();

    assert_eq!(evaluate_quota_policy(&input), QuotaPolicyDecision::Allow);
    assert_eq!(input, original);
}

#[test]
fn integration_runtime_guard_quota_policy_reason_helpers_expose_deterministic_markers() {
    assert_eq!(
        quota_policy_reason_taxonomy_version(),
        "kamn.runtime.quota-policy-reason-taxonomy.v1"
    );
    assert_eq!(
        quota_policy_reason_codes_csv(),
        "quota_scope_unknown,quota_window_non_positive,quota_limit_non_positive,quota_limit_exceeded"
    );
}
