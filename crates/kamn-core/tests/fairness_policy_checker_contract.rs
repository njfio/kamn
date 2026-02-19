use kamn_core::{
    evaluate_fairness_policy, fairness_policy_reason_codes_csv,
    fairness_policy_reason_taxonomy_version, FairnessPolicyDecision, FairnessPolicyInput,
    FairnessPolicyViolationReason,
};

const FIXTURE: &str =
    include_str!("../../../fixtures/runtime/starvation_fairness_fixture_matrix.txt");

#[derive(Debug, Clone)]
struct FairnessFixtureCase {
    scope: String,
    window_seconds: u64,
    active_weighted_share: u64,
    max_weighted_share_gap: u64,
    expected_status: String,
    expected_reason_code: String,
}

fn fixture_reason_codes_csv() -> String {
    for raw_line in FIXTURE.lines() {
        let line = raw_line.trim();
        if line.starts_with("fairness_reason_codes_csv=") {
            let (_, value) = line
                .split_once('=')
                .expect("fixture reason codes marker should be key=value");
            return value.to_owned();
        }
    }
    panic!("fairness fixture matrix must declare fairness_reason_codes_csv marker");
}

fn parse_cases() -> Vec<FairnessFixtureCase> {
    let mut cases = Vec::new();
    for raw_line in FIXTURE.lines() {
        let line = raw_line.trim();
        if line.is_empty()
            || line.starts_with('#')
            || line.contains('=')
            || line.starts_with("case_id|")
        {
            continue;
        }
        let columns: Vec<&str> = line.split('|').collect();
        if columns.len() != 7 {
            panic!("fixture case must have 7 columns: {line}");
        }
        cases.push(FairnessFixtureCase {
            scope: columns[1].to_owned(),
            window_seconds: columns[2]
                .parse::<u64>()
                .expect("window_seconds must parse as u64"),
            active_weighted_share: columns[3]
                .parse::<u64>()
                .expect("active_weighted_share must parse as u64"),
            max_weighted_share_gap: columns[4]
                .parse::<u64>()
                .expect("max_weighted_share_gap must parse as u64"),
            expected_status: columns[5].to_owned(),
            expected_reason_code: columns[6].to_owned(),
        });
    }
    cases
}

#[test]
fn unit_fairness_checker_reason_taxonomy_markers_remain_deterministic() {
    assert_eq!(
        fairness_policy_reason_taxonomy_version(),
        "kamn.runtime.fairness-policy-reason-taxonomy.v1"
    );
    assert_eq!(
        fairness_policy_reason_codes_csv(),
        "fairness_scope_unknown,fairness_window_non_positive,fairness_max_gap_non_positive,fairness_weighted_share_exceeds_gap"
    );
}

#[test]
fn regression_fairness_checker_taxonomy_remains_fixture_superset() {
    let fixture_codes = fixture_reason_codes_csv();
    let checker_codes = fairness_policy_reason_codes_csv();
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
fn functional_fairness_checker_fails_closed_for_invalid_scope_and_bounds() {
    let unknown_scope = FairnessPolicyInput {
        scope: "unknown_scope".to_owned(),
        window_seconds: 60,
        active_weighted_share: 10,
        max_weighted_share_gap: 25,
    };
    assert_eq!(
        evaluate_fairness_policy(&unknown_scope),
        FairnessPolicyDecision::Reject {
            reason: FairnessPolicyViolationReason::ScopeUnknown
        }
    );

    let invalid_window = FairnessPolicyInput {
        scope: "control_plane".to_owned(),
        window_seconds: 0,
        active_weighted_share: 10,
        max_weighted_share_gap: 25,
    };
    assert_eq!(
        evaluate_fairness_policy(&invalid_window),
        FairnessPolicyDecision::Reject {
            reason: FairnessPolicyViolationReason::WindowNonPositive
        }
    );

    let invalid_gap = FairnessPolicyInput {
        scope: "control_plane".to_owned(),
        window_seconds: 60,
        active_weighted_share: 10,
        max_weighted_share_gap: 0,
    };
    assert_eq!(
        evaluate_fairness_policy(&invalid_gap),
        FairnessPolicyDecision::Reject {
            reason: FairnessPolicyViolationReason::MaxGapNonPositive
        }
    );
}

#[test]
fn integration_fairness_checker_matches_fixture_expectations() {
    let cases = parse_cases();
    assert!(
        !cases.is_empty(),
        "fairness fixture matrix must include at least one case"
    );
    for case in cases {
        let input = FairnessPolicyInput {
            scope: case.scope.clone(),
            window_seconds: case.window_seconds,
            active_weighted_share: case.active_weighted_share,
            max_weighted_share_gap: case.max_weighted_share_gap,
        };
        let decision = evaluate_fairness_policy(&input);
        match (case.expected_status.as_str(), decision) {
            ("pass", FairnessPolicyDecision::Allow) => {
                assert_eq!(case.expected_reason_code, "none");
            }
            ("fail", FairnessPolicyDecision::Reject { reason }) => {
                assert_eq!(case.expected_reason_code, reason.as_str());
            }
            ("pass", FairnessPolicyDecision::Reject { reason }) => {
                panic!(
                    "expected pass but checker rejected with {}",
                    reason.as_str()
                );
            }
            ("fail", FairnessPolicyDecision::Allow) => {
                panic!(
                    "expected failure with reason {} but checker allowed",
                    case.expected_reason_code
                );
            }
            (status, _) => {
                panic!("unsupported expected_status in fixture: {status}");
            }
        }
    }
}
