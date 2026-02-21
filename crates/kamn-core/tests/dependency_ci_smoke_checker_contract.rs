use std::time::Instant;

use kamn_core::{
    dependency_ci_smoke_reason_codes_csv, dependency_ci_smoke_reason_taxonomy_version,
    evaluate_dependency_ci_smoke_policy, DependencyAdvisoryRecord, DependencyCiSmokeDecision,
    DependencyCiSmokePolicyInput, DependencyCiSmokeViolationReason,
};

const FIXTURE: &str =
    include_str!("../../../fixtures/ci/dependency_ci_smoke_advisory_fixture_matrix.txt");

#[derive(Debug, Clone, PartialEq, Eq)]
struct FixtureCase {
    severity: String,
    expected_status: String,
    expected_reason_code: String,
}

fn fixture_reason_codes_csv() -> String {
    for raw_line in FIXTURE.lines() {
        let line = raw_line.trim();
        if line.starts_with("dependency_ci_smoke_reason_codes_csv=") {
            let (_, value) = line
                .split_once('=')
                .expect("fixture reason codes marker should be key=value");
            return value.to_owned();
        }
    }
    panic!("fixture matrix must declare dependency_ci_smoke_reason_codes_csv marker");
}

fn fixture_threshold_max_severity() -> String {
    for raw_line in FIXTURE.lines() {
        let line = raw_line.trim();
        if line.starts_with("dependency_ci_smoke_threshold_max_severity=") {
            let (_, value) = line
                .split_once('=')
                .expect("fixture threshold marker should be key=value");
            return value.to_owned();
        }
    }
    panic!("fixture matrix must declare dependency_ci_smoke_threshold_max_severity marker");
}

fn fixture_cases() -> Vec<FixtureCase> {
    FIXTURE
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter(|line| !line.contains('='))
        .map(|line| {
            let parts: Vec<&str> = line.split('|').map(str::trim).collect();
            assert_eq!(
                parts.len(),
                6,
                "fixture case line must contain 6 columns: {line}"
            );
            FixtureCase {
                severity: parts[3].to_owned(),
                expected_status: parts[4].to_owned(),
                expected_reason_code: parts[5].to_owned(),
            }
        })
        .collect()
}

#[test]
fn unit_dependency_ci_smoke_checker_reason_taxonomy_markers_remain_deterministic() {
    assert_eq!(
        dependency_ci_smoke_reason_taxonomy_version(),
        "kamn.ci.dependency-ci-smoke-reason-taxonomy.v1"
    );
    assert_eq!(
        dependency_ci_smoke_reason_codes_csv(),
        "dependency_advisory_input_empty,dependency_advisory_severity_unknown,dependency_advisory_threshold_exceeded"
    );
}

#[test]
fn regression_dependency_ci_smoke_checker_taxonomy_remains_fixture_superset() {
    let fixture_codes = fixture_reason_codes_csv();
    let checker_codes = dependency_ci_smoke_reason_codes_csv();
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
fn functional_dependency_ci_smoke_checker_fails_closed_for_empty_input_and_unknown_severity() {
    let empty = DependencyCiSmokePolicyInput {
        threshold_max_severity: "moderate".to_owned(),
        advisories: Vec::new(),
    };
    assert_eq!(
        evaluate_dependency_ci_smoke_policy(&empty),
        DependencyCiSmokeDecision::Reject {
            reason: DependencyCiSmokeViolationReason::AdvisoryInputEmpty
        }
    );

    let unknown_severity = DependencyCiSmokePolicyInput {
        threshold_max_severity: "moderate".to_owned(),
        advisories: vec![DependencyAdvisoryRecord {
            package: "custom-crate".to_owned(),
            severity: "unknown".to_owned(),
        }],
    };
    assert_eq!(
        evaluate_dependency_ci_smoke_policy(&unknown_severity),
        DependencyCiSmokeDecision::Reject {
            reason: DependencyCiSmokeViolationReason::AdvisorySeverityUnknown
        }
    );
}

#[test]
fn integration_dependency_ci_smoke_checker_applies_fixture_threshold_deterministically() {
    let threshold_max_severity = fixture_threshold_max_severity();
    let cases = fixture_cases();

    for case in cases {
        let input = DependencyCiSmokePolicyInput {
            threshold_max_severity: threshold_max_severity.clone(),
            advisories: vec![DependencyAdvisoryRecord {
                package: "fixture-package".to_owned(),
                severity: case.severity,
            }],
        };

        match evaluate_dependency_ci_smoke_policy(&input) {
            DependencyCiSmokeDecision::Allow => {
                assert_eq!(case.expected_status, "pass");
                assert_eq!(case.expected_reason_code, "none");
            }
            DependencyCiSmokeDecision::Reject { reason } => {
                assert_eq!(case.expected_status, "fail");
                assert_eq!(case.expected_reason_code, reason.as_str());
            }
        }
    }
}

#[test]
fn performance_dependency_ci_smoke_checker_evaluation_remains_bounded() {
    let advisories: Vec<DependencyAdvisoryRecord> = (0..1024)
        .map(|index| DependencyAdvisoryRecord {
            package: format!("crate-{index}"),
            severity: "moderate".to_owned(),
        })
        .collect();

    let input = DependencyCiSmokePolicyInput {
        threshold_max_severity: "moderate".to_owned(),
        advisories,
    };

    let start = Instant::now();
    let decision = evaluate_dependency_ci_smoke_policy(&input);
    let elapsed_ms = start.elapsed().as_millis();

    assert_eq!(decision, DependencyCiSmokeDecision::Allow);
    assert!(
        elapsed_ms <= 250,
        "dependency ci smoke checker evaluation exceeded budget: {elapsed_ms}ms"
    );
}
