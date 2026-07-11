use super::*;

#[test]
fn false_confirmation_is_an_ambiguous_outcome() {
    let error = require_confirmation(false, "finalized").expect_err("ambiguous result");

    assert!(error.starts_with("SETTLEMENT_OUTCOME_AMBIGUOUS"));
}
