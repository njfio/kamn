use super::support::*;

#[test]
fn unit_finality_checker_rejects_empty_endpoint_or_status_path() {
    assert_invalid_checker("", "/commit/finality", "provider_base_url");
    assert_invalid_checker("http://127.0.0.1:3030", "", "provider_status_path");
}

#[test]
fn functional_finality_checker_maps_confirmed_alias_to_final_receipt() {
    let (mut checker, calls) = checker_with_responses(
        "http://127.0.0.1:3030",
        "/commit/finality",
        vec![finality_response(
            "kolme-fork-local",
            "kolme-commit:ab12cd34:h42",
            "confirmed",
        )],
    );

    let receipt = checker
        .check_commit_finality("kolme-commit:ab12cd34:h42")
        .expect("checker should parse finality response");

    assert_eq!(receipt, expected_receipt(KolmeCommitReceiptFinality::Final));
    assert_single_call(&calls, "http://127.0.0.1:3030", "/commit/finality");
}

#[test]
fn regression_issue_1918_finality_checker_trims_endpoint_inputs() {
    // Regression: #1918
    let (mut checker, calls) = checker_with_responses(
        "  http://127.0.0.1:3030  ",
        "  /commit/finality  ",
        vec![finality_response(
            "kolme-fork-local",
            "kolme-commit:ab12cd34:h42",
            "pending",
        )],
    );

    let receipt = checker
        .check_commit_finality("kolme-commit:ab12cd34:h42")
        .expect("checker should parse finality response");

    assert_eq!(receipt.provider, "kolme-fork-local");
    assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Pending);
    assert_single_call(&calls, "http://127.0.0.1:3030", "/commit/finality");
}

#[test]
fn functional_finality_checker_polls_pending_then_final() {
    let (mut checker, _calls) = checker_with_responses(
        "http://127.0.0.1:3030",
        "/commit/finality",
        vec![
            finality_response("kolme-fork-local", "kolme-commit:ab12cd34:h42", "pending"),
            finality_response("kolme-fork-local", "kolme-commit:ab12cd34:h42", "confirmed"),
        ],
    );

    let receipt = checker
        .poll_finality("kolme-commit:ab12cd34:h42", 2)
        .expect("checker should return first non-pending finality");

    assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Final);
}

#[test]
fn regression_finality_checker_fails_closed_for_commit_id_mismatch() {
    // Regression: #1413
    let (mut checker, _calls) = checker_with_responses(
        "http://127.0.0.1:3030",
        "/commit/finality",
        vec![finality_response(
            "kolme-fork-local",
            "kolme-commit:other:h42",
            "final",
        )],
    );

    assert!(
        matches!(
            checker.check_commit_finality("kolme-commit:ab12cd34:h42"),
            Err(KolmeRuntimeCommitProviderError::MalformedResponse { .. })
        ),
        "checker must fail closed when response commit id mismatches requested commit id"
    );
}

#[test]
fn regression_finality_checker_times_out_when_pending_budget_exhausted() {
    // Regression: #1413
    let pending = finality_response("kolme-fork-local", "kolme-commit:ab12cd34:h42", "pending");
    let (mut checker, _calls) = checker_with_responses(
        "http://127.0.0.1:3030",
        "/commit/finality",
        vec![pending.clone(), pending],
    );

    assert_eq!(
        checker.poll_finality("kolme-commit:ab12cd34:h42", 2),
        Err(KolmeRuntimeCommitProviderError::Timeout)
    );
}

fn assert_invalid_checker(base_url: &str, status_path: &str, field: &'static str) {
    let (transport, _calls) = RecordingFinalityTransport::with_responses(Vec::new());
    assert!(
        matches!(
            KolmeRuntimeCommitFinalityChecker::new(base_url, status_path, transport),
            Err(KolmeRuntimeCommitError::InvalidRequest {
                field: observed_field,
                reason: "must not be empty",
            }) if observed_field == field
        ),
        "finality checker should reject empty {field}"
    );
}

fn checker_with_responses(
    base_url: &str,
    status_path: &str,
    responses: Vec<Result<String, KolmeRuntimeCommitProviderError>>,
) -> (
    KolmeRuntimeCommitFinalityChecker<RecordingFinalityTransport>,
    FinalityTransportCalls,
) {
    let (transport, calls) = RecordingFinalityTransport::with_responses(responses);
    let checker = KolmeRuntimeCommitFinalityChecker::new(base_url, status_path, transport)
        .expect("checker should build");
    (checker, calls)
}

fn finality_response(
    provider: &str,
    commit_id: &str,
    finality: &str,
) -> Result<String, KolmeRuntimeCommitProviderError> {
    Ok(format!(
        r#"{{"provider":"{provider}","commit_id":"{commit_id}","finality":"{finality}"}}"#
    ))
}

fn expected_receipt(finality: KolmeCommitReceiptFinality) -> KolmeRuntimeCommitProviderReceipt {
    KolmeRuntimeCommitProviderReceipt {
        provider: "kolme-fork-local".to_owned(),
        commit_id: "kolme-commit:ab12cd34:h42".to_owned(),
        finality,
    }
}

fn assert_single_call(calls: &FinalityTransportCalls, base_url: &str, status_path: &str) {
    let calls = calls.borrow();
    assert_eq!(calls.len(), 1, "finality transport should be called once");
    assert_eq!(calls[0].0, base_url);
    assert_eq!(calls[0].1, status_path);
    assert_eq!(calls[0].2, "kolme-commit:ab12cd34:h42");
}
