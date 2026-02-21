use kamn_core::RetentionDomain;

const FIXTURE: &str = include_str!("../../../fixtures/runtime/retention_policy_fixture_matrix.txt");

#[derive(Debug, Clone, PartialEq, Eq)]
struct RetentionPolicyFixtureMetadata {
    schema_version: String,
    reason_taxonomy_version: String,
    reason_codes_csv: String,
    columns: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RetentionPolicyFixtureCase {
    case_id: String,
    domain: String,
    max_age_seconds: u64,
    expected_status: String,
    expected_reason_code: String,
}

fn parse_fixture() -> Result<
    (
        RetentionPolicyFixtureMetadata,
        Vec<RetentionPolicyFixtureCase>,
    ),
    String,
> {
    let mut schema_version = None;
    let mut reason_taxonomy_version = None;
    let mut reason_codes_csv = None;
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
                "retention_policy_fixture_matrix_schema_version" => schema_version = Some(value),
                "retention_policy_reason_taxonomy_version" => reason_taxonomy_version = Some(value),
                "retention_policy_reason_codes_csv" => reason_codes_csv = Some(value),
                "columns" => columns = Some(value),
                unknown => return Err(format!("unknown metadata key: {unknown}")),
            }
            continue;
        }
        cases.push(parse_case_line(line)?);
    }

    let metadata = RetentionPolicyFixtureMetadata {
        schema_version: schema_version.ok_or("missing schema version metadata".to_owned())?,
        reason_taxonomy_version: reason_taxonomy_version
            .ok_or("missing reason taxonomy metadata".to_owned())?,
        reason_codes_csv: reason_codes_csv.ok_or("missing reason codes csv metadata".to_owned())?,
        columns: columns.ok_or("missing columns metadata".to_owned())?,
    };

    if cases.is_empty() {
        return Err("fixture matrix must contain at least one case".to_owned());
    }
    Ok((metadata, cases))
}

fn parse_case_line(line: &str) -> Result<RetentionPolicyFixtureCase, String> {
    let parts: Vec<&str> = line.split('|').map(str::trim).collect();
    if parts.len() != 5 {
        return Err(format!(
            "expected 5 columns, found {} in '{line}'",
            parts.len()
        ));
    }
    let max_age_seconds = parts[2]
        .parse::<u64>()
        .map_err(|_| format!("max_age_seconds must be an unsigned integer in '{line}'"))?;

    Ok(RetentionPolicyFixtureCase {
        case_id: parts[0].to_owned(),
        domain: parts[1].to_owned(),
        max_age_seconds,
        expected_status: parts[3].to_owned(),
        expected_reason_code: parts[4].to_owned(),
    })
}

fn parse_domain(value: &str) -> Option<RetentionDomain> {
    match value {
        "messages" => Some(RetentionDomain::Messages),
        "tasks" => Some(RetentionDomain::Tasks),
        "escrows" => Some(RetentionDomain::Escrows),
        "reputation" => Some(RetentionDomain::Reputation),
        _ => None,
    }
}

fn evaluate_case(case: &RetentionPolicyFixtureCase) -> (&'static str, &'static str) {
    if parse_domain(&case.domain).is_none() {
        return ("fail", "retention_domain_unknown");
    }
    if case.max_age_seconds == 0 {
        return ("fail", "retention_window_non_positive");
    }
    ("pass", "none")
}

#[test]
fn unit_retention_fixture_parser_rejects_malformed_case_columns() {
    let error = parse_case_line("broken|line")
        .expect_err("malformed fixture case line should fail parsing deterministically");
    assert!(
        error.contains("expected 5 columns"),
        "unexpected malformed-line parser error: {error}"
    );
}

#[test]
fn functional_retention_fixture_matrix_covers_valid_and_invalid_windows() {
    let (_, cases) = parse_fixture().expect("fixture matrix should parse");
    assert!(
        cases.iter().any(|case| case.expected_status == "pass"),
        "fixture matrix must include at least one valid pass case"
    );
    assert!(
        cases
            .iter()
            .any(|case| case.expected_reason_code == "retention_domain_unknown"),
        "fixture matrix must include unknown-domain fail-closed coverage"
    );
    assert!(
        cases
            .iter()
            .any(|case| case.expected_reason_code == "retention_window_non_positive"),
        "fixture matrix must include non-positive window fail-closed coverage"
    );
}

#[test]
fn integration_retention_fixture_parser_contract_matches_expected_policy_outcomes() {
    let (_, cases) = parse_fixture().expect("fixture matrix should parse");
    for case in cases {
        let (status, reason) = evaluate_case(&case);
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
fn regression_retention_fixture_reason_taxonomy_markers_remain_deterministic() {
    let (metadata, cases) = parse_fixture().expect("fixture matrix should parse");
    assert_eq!(
        metadata.schema_version,
        "kamn.runtime.retention-policy-fixture-matrix.v1"
    );
    assert_eq!(
        metadata.reason_taxonomy_version,
        "kamn.runtime.retention-policy-fixture-reason-taxonomy.v1"
    );
    assert_eq!(
        metadata.reason_codes_csv,
        "retention_domain_unknown,retention_window_non_positive"
    );
    assert_eq!(
        metadata.columns,
        "case_id|domain|max_age_seconds|expected_status|expected_reason_code"
    );
    let observed_case_ids: Vec<String> = cases.into_iter().map(|case| case.case_id).collect();
    assert_eq!(
        observed_case_ids,
        vec![
            "valid_messages_window",
            "valid_tasks_window",
            "valid_escrows_window",
            "invalid_domain",
            "invalid_window",
        ]
    );
}
