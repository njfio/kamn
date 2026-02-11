use kamn_kolme::{
    parse_provider_finality_receipt, KolmeCommitReceiptFinality, KolmeProviderFinalityReceipt,
    KolmeProviderFinalityReceiptPolicyError,
};

#[test]
fn functional_parse_provider_finality_receipt_maps_response_to_receipt_contract() {
    let response = r#"{"provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34:h42","finality":"confirmed"}"#;
    let receipt = parse_provider_finality_receipt(response, "kolme-commit:ab12cd34:h42")
        .expect("finality response should parse");
    assert_eq!(
        receipt,
        KolmeProviderFinalityReceipt {
            provider: "kolme-fork-local".to_owned(),
            commit_id: "kolme-commit:ab12cd34:h42".to_owned(),
            finality: KolmeCommitReceiptFinality::Final,
        }
    );
}

#[test]
fn regression_issue_1826_parse_provider_finality_receipt_rejects_commit_id_mismatch() {
    // Regression: #1826
    let response = r#"{"provider":"kolme-fork-local","commit_id":"kolme-commit:other:h42","finality":"final"}"#;
    let error = parse_provider_finality_receipt(response, "kolme-commit:ab12cd34:h42")
        .expect_err("commit_id mismatch must fail closed");
    assert_eq!(
        error,
        KolmeProviderFinalityReceiptPolicyError::MalformedResponse {
            reason: "commit_id mismatch: expected 'kolme-commit:ab12cd34:h42', observed 'kolme-commit:other:h42'".to_owned(),
        }
    );
}

#[test]
fn regression_issue_1826_parse_provider_finality_receipt_rejects_missing_finality_field() {
    // Regression: #1826
    let response = r#"{"provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34:h42"}"#;
    let error = parse_provider_finality_receipt(response, "kolme-commit:ab12cd34:h42")
        .expect_err("missing finality field must fail closed");
    assert_eq!(
        error,
        KolmeProviderFinalityReceiptPolicyError::MalformedResponse {
            reason: "missing required field: finality".to_owned(),
        }
    );
}
