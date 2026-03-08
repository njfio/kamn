use kamn_runtime_guards::fairness_policy::{
    evaluate_fairness_policy, fairness_policy_reason_codes_csv,
    fairness_policy_reason_taxonomy_version, FairnessPolicyDecision, FairnessPolicyInput,
    FairnessPolicyViolationReason,
};

fn valid_input(scope: &str) -> FairnessPolicyInput {
    FairnessPolicyInput {
        scope: scope.to_owned(),
        window_seconds: 60,
        active_weighted_share: 3,
        max_weighted_share_gap: 5,
    }
}

#[test]
fn integration_runtime_guard_fairness_policy_allows_all_supported_scope_classes() {
    for scope in ["control_plane", "tenant_interactive", "bulk_replication"] {
        assert_eq!(
            evaluate_fairness_policy(&valid_input(scope)),
            FairnessPolicyDecision::Allow
        );
    }
}

#[test]
fn integration_runtime_guard_fairness_policy_rejects_invalid_inputs_with_deterministic_reasons() {
    let cases = [
        (
            FairnessPolicyInput {
                scope: "unknown".to_owned(),
                ..valid_input("control_plane")
            },
            FairnessPolicyViolationReason::ScopeUnknown,
        ),
        (
            FairnessPolicyInput {
                window_seconds: 0,
                ..valid_input("control_plane")
            },
            FairnessPolicyViolationReason::WindowNonPositive,
        ),
        (
            FairnessPolicyInput {
                max_weighted_share_gap: 0,
                ..valid_input("control_plane")
            },
            FairnessPolicyViolationReason::MaxGapNonPositive,
        ),
        (
            FairnessPolicyInput {
                active_weighted_share: 6,
                ..valid_input("control_plane")
            },
            FairnessPolicyViolationReason::WeightedShareExceedsGap,
        ),
    ];

    for (input, reason) in cases {
        assert_eq!(
            evaluate_fairness_policy(&input),
            FairnessPolicyDecision::Reject { reason }
        );
    }
}

#[test]
fn integration_runtime_guard_fairness_policy_allows_gap_boundary_without_mutating_input() {
    let input = FairnessPolicyInput {
        active_weighted_share: 5,
        ..valid_input("control_plane")
    };
    let original = input.clone();

    assert_eq!(evaluate_fairness_policy(&input), FairnessPolicyDecision::Allow);
    assert_eq!(input, original);
}

#[test]
fn integration_runtime_guard_fairness_policy_reason_helpers_expose_deterministic_markers() {
    assert_eq!(
        fairness_policy_reason_taxonomy_version(),
        "kamn.runtime.fairness-policy-reason-taxonomy.v1"
    );
    assert_eq!(
        fairness_policy_reason_codes_csv(),
        "fairness_scope_unknown,fairness_window_non_positive,fairness_max_gap_non_positive,fairness_weighted_share_exceeds_gap"
    );
}
