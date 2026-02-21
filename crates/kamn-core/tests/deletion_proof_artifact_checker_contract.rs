const FIXTURE: &str =
    include_str!("../../../fixtures/runtime/deletion_proof_artifact_fixture_matrix.txt");

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeletionProofFixtureMetadata {
    schema_version: String,
    reason_taxonomy_version: String,
    reason_codes_csv: String,
    columns: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeletionProofFixtureCase {
    case_id: String,
    subject_id: String,
    tombstone_hash: String,
    expected_hash: String,
    proof_status: String,
    expected_status: String,
    expected_reason_code: String,
}

fn parse_fixture() -> Result<(DeletionProofFixtureMetadata, Vec<DeletionProofFixtureCase>), String>
{
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
                "deletion_proof_fixture_matrix_schema_version" => schema_version = Some(value),
                "deletion_proof_reason_taxonomy_version" => reason_taxonomy_version = Some(value),
                "deletion_proof_reason_codes_csv" => reason_codes_csv = Some(value),
                "columns" => columns = Some(value),
                unknown => return Err(format!("unknown metadata key: {unknown}")),
            }
            continue;
        }
        cases.push(parse_case_line(line)?);
    }

    let metadata = DeletionProofFixtureMetadata {
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

fn parse_case_line(line: &str) -> Result<DeletionProofFixtureCase, String> {
    let parts: Vec<&str> = line.split('|').map(str::trim).collect();
    if parts.len() != 7 {
        return Err(format!(
            "expected 7 columns, found {} in '{line}'",
            parts.len()
        ));
    }

    Ok(DeletionProofFixtureCase {
        case_id: parts[0].to_owned(),
        subject_id: parts[1].to_owned(),
        tombstone_hash: parts[2].to_owned(),
        expected_hash: parts[3].to_owned(),
        proof_status: parts[4].to_owned(),
        expected_status: parts[5].to_owned(),
        expected_reason_code: parts[6].to_owned(),
    })
}

fn evaluate_case(case: &DeletionProofFixtureCase) -> (&'static str, &'static str) {
    if case.subject_id.trim().is_empty() {
        return ("fail", "deletion_proof_subject_missing");
    }
    if case.tombstone_hash.trim().is_empty() {
        return ("fail", "deletion_proof_tombstone_missing");
    }
    if case.proof_status != "deleted" {
        return ("fail", "deletion_proof_status_invalid");
    }
    if case.tombstone_hash != case.expected_hash {
        return ("fail", "deletion_proof_hash_mismatch");
    }
    ("pass", "none")
}

#[test]
fn unit_deletion_proof_fixture_parser_rejects_malformed_case_columns() {
    let error = parse_case_line("broken|line")
        .expect_err("malformed fixture case line should fail parsing deterministically");
    assert!(
        error.contains("expected 7 columns"),
        "unexpected malformed-line parser error: {error}"
    );
}

#[test]
fn functional_deletion_proof_fixture_matrix_covers_valid_and_invalid_proof_classes() {
    let (_, cases) = parse_fixture().expect("fixture matrix should parse");
    assert!(
        cases.iter().any(|case| case.expected_status == "pass"),
        "fixture matrix must include at least one valid pass case"
    );
    assert!(
        cases
            .iter()
            .any(|case| case.expected_reason_code == "deletion_proof_subject_missing"),
        "fixture matrix must include missing-subject fail-closed coverage"
    );
    assert!(
        cases
            .iter()
            .any(|case| case.expected_reason_code == "deletion_proof_tombstone_missing"),
        "fixture matrix must include missing-tombstone fail-closed coverage"
    );
    assert!(
        cases
            .iter()
            .any(|case| case.expected_reason_code == "deletion_proof_status_invalid"),
        "fixture matrix must include invalid-status fail-closed coverage"
    );
    assert!(
        cases
            .iter()
            .any(|case| case.expected_reason_code == "deletion_proof_hash_mismatch"),
        "fixture matrix must include hash-mismatch fail-closed coverage"
    );
}

#[test]
fn integration_deletion_proof_checker_contract_matches_expected_fixture_outcomes() {
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
fn regression_deletion_proof_fixture_markers_remain_deterministic() {
    let (metadata, cases) = parse_fixture().expect("fixture matrix should parse");
    assert_eq!(
        metadata.schema_version,
        "kamn.runtime.deletion-proof-fixture-matrix.v1"
    );
    assert_eq!(
        metadata.reason_taxonomy_version,
        "kamn.runtime.deletion-proof-checker-reason-taxonomy.v1"
    );
    assert_eq!(
        metadata.reason_codes_csv,
        "deletion_proof_subject_missing,deletion_proof_tombstone_missing,deletion_proof_status_invalid,deletion_proof_hash_mismatch"
    );
    assert_eq!(
        metadata.columns,
        "case_id|subject_id|tombstone_hash|expected_hash|proof_status|expected_status|expected_reason_code"
    );

    let observed_case_ids: Vec<String> = cases.into_iter().map(|case| case.case_id).collect();
    assert_eq!(
        observed_case_ids,
        vec![
            "valid_message_deletion_proof",
            "valid_task_deletion_proof",
            "invalid_subject_missing",
            "invalid_tombstone_missing",
            "invalid_status",
            "invalid_hash_mismatch",
        ]
    );
}
