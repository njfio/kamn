use super::*;

#[test]
fn integration_fixture_validator_classifies_commit_request_schema_cases() {
    let cases = parse_fixture_cases();
    assert_eq!(cases.len(), 5);
    cases.iter().for_each(assert_fixture_case);
}

fn assert_fixture_case(case: &FixtureCase) {
    let result = fixture_request(case);
    if case.expected_status == "pass" {
        assert_fixture_pass(case, &result);
        return;
    }
    if case.expected_status == "fail" {
        assert_fixture_failure(case, result);
        return;
    }
    panic!(
        "unexpected fixture expected_status value: {}",
        case.expected_status
    );
}

fn fixture_request(
    case: &FixtureCase,
) -> Result<KolmeRuntimeCommitRequest, KolmeRuntimeCommitError> {
    KolmeRuntimeCommitRequest::deterministic(
        case.operation_id.as_str(),
        case.state_root.as_str(),
        case.actor_did.as_str(),
        case.nonce,
        case.payload_hash.as_str(),
    )
}

fn assert_fixture_pass(
    case: &FixtureCase,
    result: &Result<KolmeRuntimeCommitRequest, KolmeRuntimeCommitError>,
) {
    assert!(
        result.is_ok(),
        "expected case '{}' to pass, got {result:?}",
        case.case_id
    );
}

fn assert_fixture_failure(
    case: &FixtureCase,
    result: Result<KolmeRuntimeCommitRequest, KolmeRuntimeCommitError>,
) {
    match result {
        Err(KolmeRuntimeCommitError::InvalidRequest { reason, .. }) => {
            assert_eq!(
                reason,
                case.expected_reason.as_str(),
                "fixture reason mismatch for case {}",
                case.case_id
            );
        }
        other => panic!(
            "expected invalid request error for case '{}' but got {other:?}",
            case.case_id
        ),
    }
}
