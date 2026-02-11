use kamn_kolme::{
    deterministic_backend_commit_id, parse_live_provider_outcome, txhash_from_commit_id,
    KolmeProviderOutcome, KolmeProviderOutcomePolicyError, ReceiptFinality,
};

#[test]
fn functional_parse_live_provider_outcome_maps_submitted_status() {
    let response =
        "status=submitted\nprovider=kolme-fork\ncommit_id=kolme-commit:ab12cd34\nfinality=final\n";
    let outcome = parse_live_provider_outcome(response, None).expect("response should parse");
    assert_eq!(
        outcome,
        KolmeProviderOutcome::Submitted {
            provider: "kolme-fork".to_owned(),
            commit_id: "kolme-commit:ab12cd34".to_owned(),
            finality: ReceiptFinality::Final,
        }
    );
}

#[test]
fn functional_parse_live_provider_outcome_uses_provider_hint_for_txhash_only_shape() {
    let outcome = parse_live_provider_outcome("{\"txhash\":\"ab12cd34\"}", Some("kolme-fork"))
        .expect("txhash-only response should parse");
    assert_eq!(
        outcome,
        KolmeProviderOutcome::Submitted {
            provider: "kolme-fork".to_owned(),
            commit_id: "kolme-commit:ab12cd34".to_owned(),
            finality: ReceiptFinality::Pending,
        }
    );
}

#[test]
fn unit_deterministic_backend_commit_id_includes_optional_height() {
    assert_eq!(
        deterministic_backend_commit_id("ab12cd34", Some(72)),
        "kolme-commit:ab12cd34:h72".to_owned()
    );
    assert_eq!(
        deterministic_backend_commit_id("ab12cd34", None),
        "kolme-commit:ab12cd34".to_owned()
    );
}

#[test]
fn regression_issue_1749_parse_live_provider_outcome_rejects_missing_status_and_txhash() {
    // Regression: #1749
    let error = parse_live_provider_outcome("provider=kolme-fork\n", None)
        .expect_err("missing status/txhash should fail closed");
    assert_eq!(
        error,
        KolmeProviderOutcomePolicyError::MalformedResponse {
            reason: "missing required field: status".to_owned(),
        }
    );
}

#[test]
fn regression_issue_1749_txhash_from_commit_id_rejects_invalid_prefix() {
    // Regression: #1749
    let error =
        txhash_from_commit_id("commit:ab12cd34").expect_err("invalid commit_id prefix must fail");
    assert_eq!(
        error,
        KolmeProviderOutcomePolicyError::MalformedResponse {
            reason: "commit_id must start with 'kolme-commit:'".to_owned(),
        }
    );
}
