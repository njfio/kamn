const FIXTURE: &str = include_str!("../../../fixtures/runtime/quota_policy_fixture_matrix.txt");

#[derive(Debug, Clone, PartialEq, Eq)]
struct QuotaPolicyFixtureMetadata {
    schema_version: String,
    reason_taxonomy_version: String,
    reason_codes_csv: String,
    columns: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QuotaPolicyFixtureCase {
    case_id: String,
    scope: String,
    window_seconds: u64,
    limit: u64,
    expected_status: String,
    expected_reason_code: String,
}

fn parse_fixture() -> Result<(QuotaPolicyFixtureMetadata, Vec<QuotaPolicyFixtureCase>), String> {
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
                "quota_policy_fixture_matrix_schema_version" => schema_version = Some(value),
                "quota_policy_reason_taxonomy_version" => reason_taxonomy_version = Some(value),
                "quota_policy_reason_codes_csv" => reason_codes_csv = Some(value),
                "columns" => columns = Some(value),
                unknown => return Err(format!("unknown metadata key: {unknown}")),
            }
            continue;
        }
        cases.push(parse_case_line(line)?);
    }

    let metadata = QuotaPolicyFixtureMetadata {
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

fn parse_case_line(line: &str) -> Result<QuotaPolicyFixtureCase, String> {
    let parts: Vec<&str> = line.split('|').map(str::trim).collect();
    if parts.len() != 6 {
        return Err(format!(
            "expected 6 columns, found {} in '{line}'",
            parts.len()
        ));
    }
    let window_seconds = parts[2]
        .parse::<u64>()
        .map_err(|_| format!("window_seconds must be an unsigned integer in '{line}'"))?;
    let limit = parts[3]
        .parse::<u64>()
        .map_err(|_| format!("limit must be an unsigned integer in '{line}'"))?;

    Ok(QuotaPolicyFixtureCase {
        case_id: parts[0].to_owned(),
        scope: parts[1].to_owned(),
        window_seconds,
        limit,
        expected_status: parts[4].to_owned(),
        expected_reason_code: parts[5].to_owned(),
    })
}

fn evaluate_case(case: &QuotaPolicyFixtureCase) -> (&'static str, &'static str) {
    const ALLOWED_SCOPES: &[&str] = &["processor_ingress", "peer_sync", "channel_broadcast"];

    if !ALLOWED_SCOPES.contains(&case.scope.as_str()) {
        return ("fail", "quota_scope_unknown");
    }
    if case.window_seconds == 0 {
        return ("fail", "quota_window_non_positive");
    }
    if case.limit == 0 {
        return ("fail", "quota_limit_non_positive");
    }
    ("pass", "none")
}

#[test]
fn unit_quota_fixture_parser_rejects_malformed_case_columns() {
    let error = parse_case_line("broken|line")
        .expect_err("malformed fixture case line should fail parsing deterministically");
    assert!(
        error.contains("expected 6 columns"),
        "unexpected malformed-line parser error: {error}"
    );
}

#[test]
fn functional_quota_fixture_matrix_covers_valid_and_invalid_quota_windows() {
    let (_, cases) = parse_fixture().expect("fixture matrix should parse");
    assert!(
        cases.iter().any(|case| case.expected_status == "pass"),
        "fixture matrix must include at least one valid pass case"
    );
    assert!(
        cases
            .iter()
            .any(|case| case.expected_reason_code == "quota_window_non_positive"),
        "fixture matrix must include invalid quota-window coverage"
    );
}

#[test]
fn integration_quota_fixture_parser_contract_matches_expected_policy_outcomes() {
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
fn regression_quota_fixture_reason_taxonomy_markers_remain_deterministic() {
    let (metadata, cases) = parse_fixture().expect("fixture matrix should parse");
    assert_eq!(
        metadata.schema_version,
        "kamn.runtime.quota-policy-fixture-matrix.v1"
    );
    assert_eq!(
        metadata.reason_taxonomy_version,
        "kamn.runtime.quota-policy-reason-taxonomy.v1"
    );
    assert_eq!(
        metadata.reason_codes_csv,
        "quota_scope_unknown,quota_window_non_positive,quota_limit_non_positive"
    );
    assert_eq!(
        metadata.columns,
        "case_id|scope|window_seconds|limit|expected_status|expected_reason_code"
    );
    let observed_case_ids: Vec<String> = cases.into_iter().map(|case| case.case_id).collect();
    assert_eq!(
        observed_case_ids,
        vec![
            "valid_processor_ingress",
            "valid_peer_sync",
            "invalid_scope",
            "invalid_window",
            "invalid_limit",
        ]
    );
}
