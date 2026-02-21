use std::time::Instant;

const FIXTURE: &str =
    include_str!("../../../fixtures/ci/dependency_ci_smoke_advisory_fixture_matrix.txt");

#[derive(Debug, Clone, PartialEq, Eq)]
struct DependencyCiSmokeFixtureMetadata {
    schema_version: String,
    reason_taxonomy_version: String,
    reason_codes_csv: String,
    threshold_max_severity: String,
    columns: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DependencyCiSmokeFixtureCase {
    case_id: String,
    package: String,
    advisory_id: String,
    severity: String,
    expected_status: String,
    expected_reason_code: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AdvisorySeverity {
    Low,
    Moderate,
    High,
    Critical,
}

fn parse_severity(value: &str) -> Option<AdvisorySeverity> {
    match value {
        "low" => Some(AdvisorySeverity::Low),
        "moderate" => Some(AdvisorySeverity::Moderate),
        "high" => Some(AdvisorySeverity::High),
        "critical" => Some(AdvisorySeverity::Critical),
        _ => None,
    }
}

fn parse_fixture() -> Result<
    (
        DependencyCiSmokeFixtureMetadata,
        Vec<DependencyCiSmokeFixtureCase>,
    ),
    String,
> {
    let mut schema_version = None;
    let mut reason_taxonomy_version = None;
    let mut reason_codes_csv = None;
    let mut threshold_max_severity = None;
    let mut columns = None;
    let mut cases = Vec::new();

    for raw_line in FIXTURE.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.contains('=') {
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| format!("invalid metadata line: {line}"))?;
            let value = value.trim().to_owned();
            match key.trim() {
                "dependency_ci_smoke_advisory_fixture_schema_version" => {
                    schema_version = Some(value)
                }
                "dependency_ci_smoke_reason_taxonomy_version" => {
                    reason_taxonomy_version = Some(value)
                }
                "dependency_ci_smoke_reason_codes_csv" => reason_codes_csv = Some(value),
                "dependency_ci_smoke_threshold_max_severity" => {
                    threshold_max_severity = Some(value)
                }
                "columns" => columns = Some(value),
                unknown => return Err(format!("unknown metadata key: {unknown}")),
            }
            continue;
        }
        cases.push(parse_case_line(line)?);
    }

    let metadata = DependencyCiSmokeFixtureMetadata {
        schema_version: schema_version.ok_or("missing schema version metadata".to_owned())?,
        reason_taxonomy_version: reason_taxonomy_version
            .ok_or("missing reason taxonomy metadata".to_owned())?,
        reason_codes_csv: reason_codes_csv.ok_or("missing reason codes csv metadata".to_owned())?,
        threshold_max_severity: threshold_max_severity
            .ok_or("missing threshold max severity metadata".to_owned())?,
        columns: columns.ok_or("missing columns metadata".to_owned())?,
    };

    if parse_severity(&metadata.threshold_max_severity).is_none() {
        return Err(format!(
            "invalid threshold max severity: {}",
            metadata.threshold_max_severity
        ));
    }

    if cases.is_empty() {
        return Err("fixture matrix must contain at least one case".to_owned());
    }

    Ok((metadata, cases))
}

fn parse_case_line(line: &str) -> Result<DependencyCiSmokeFixtureCase, String> {
    let parts: Vec<&str> = line.split('|').map(str::trim).collect();
    if parts.len() != 6 {
        return Err(format!(
            "expected 6 columns, found {} in '{line}'",
            parts.len()
        ));
    }

    Ok(DependencyCiSmokeFixtureCase {
        case_id: parts[0].to_owned(),
        package: parts[1].to_owned(),
        advisory_id: parts[2].to_owned(),
        severity: parts[3].to_owned(),
        expected_status: parts[4].to_owned(),
        expected_reason_code: parts[5].to_owned(),
    })
}

fn evaluate_case(
    case: &DependencyCiSmokeFixtureCase,
    threshold_max_severity: AdvisorySeverity,
) -> (&'static str, &'static str) {
    let Some(case_severity) = parse_severity(&case.severity) else {
        return ("fail", "dependency_advisory_severity_unknown");
    };

    if case_severity > threshold_max_severity {
        return ("fail", "dependency_advisory_threshold_exceeded");
    }

    ("pass", "none")
}

#[test]
fn unit_dependency_ci_smoke_fixture_parser_rejects_malformed_case_columns() {
    let error = parse_case_line("broken|line")
        .expect_err("malformed fixture case line should fail parsing deterministically");
    assert!(
        error.contains("expected 6 columns"),
        "unexpected malformed-line parser error: {error}"
    );
}

#[test]
fn functional_dependency_ci_smoke_fixture_matrix_covers_expected_severity_rows() {
    let (_, cases) = parse_fixture().expect("fixture matrix should parse");

    assert!(
        cases.iter().any(|case| case.severity == "low"),
        "fixture matrix must include low severity coverage"
    );
    assert!(
        cases.iter().any(|case| case.severity == "moderate"),
        "fixture matrix must include moderate severity coverage"
    );
    assert!(
        cases.iter().any(|case| case.severity == "high"),
        "fixture matrix must include high severity coverage"
    );
    assert!(
        cases.iter().any(|case| case.severity == "critical"),
        "fixture matrix must include critical severity coverage"
    );
    assert!(
        cases
            .iter()
            .any(|case| case.expected_reason_code == "dependency_advisory_severity_unknown"),
        "fixture matrix must include unknown-severity fail-closed coverage"
    );
    assert!(
        cases
            .iter()
            .any(|case| case.expected_reason_code == "dependency_advisory_threshold_exceeded"),
        "fixture matrix must include threshold-exceeded fail-closed coverage"
    );
}

#[test]
fn integration_dependency_ci_smoke_threshold_mapping_matches_expected_outcomes() {
    let (metadata, cases) = parse_fixture().expect("fixture matrix should parse");
    let threshold_max_severity = parse_severity(&metadata.threshold_max_severity)
        .expect("threshold max severity should parse deterministically");

    for case in cases {
        let (status, reason) = evaluate_case(&case, threshold_max_severity);
        assert_eq!(
            status, case.expected_status,
            "status mismatch for case '{}'",
            case.case_id
        );
        assert_eq!(
            reason, case.expected_reason_code,
            "reason mismatch for case '{}'",
            case.case_id
        );
    }
}

#[test]
fn regression_dependency_ci_smoke_fixture_taxonomy_markers_remain_deterministic() {
    let (metadata, cases) = parse_fixture().expect("fixture matrix should parse");
    assert_eq!(
        metadata.schema_version,
        "kamn.ci.dependency-ci-smoke-advisory-fixture-matrix.v1"
    );
    assert_eq!(
        metadata.reason_taxonomy_version,
        "kamn.ci.dependency-ci-smoke-reason-taxonomy.v1"
    );
    assert_eq!(
        metadata.reason_codes_csv,
        "dependency_advisory_severity_unknown,dependency_advisory_threshold_exceeded"
    );
    assert_eq!(metadata.threshold_max_severity, "moderate");
    assert_eq!(
        metadata.columns,
        "case_id|package|advisory_id|severity|expected_status|expected_reason_code"
    );

    let observed_case_ids: Vec<String> = cases.into_iter().map(|case| case.case_id).collect();
    assert_eq!(
        observed_case_ids,
        vec![
            "advisory_low_pass",
            "advisory_moderate_pass",
            "advisory_high_fail",
            "advisory_critical_fail",
            "advisory_unknown_fail",
        ]
    );
}

#[test]
fn performance_dependency_ci_smoke_fixture_parse_and_evaluate_within_budget() {
    let start = Instant::now();
    let (metadata, cases) = parse_fixture().expect("fixture matrix should parse");
    let threshold_max_severity = parse_severity(&metadata.threshold_max_severity)
        .expect("threshold max severity should parse deterministically");

    for case in &cases {
        let _ = evaluate_case(case, threshold_max_severity);
    }

    let elapsed_ms = start.elapsed().as_millis();
    assert!(
        elapsed_ms <= 250,
        "dependency advisory fixture parse/evaluate exceeded budget: {elapsed_ms}ms"
    );
}
