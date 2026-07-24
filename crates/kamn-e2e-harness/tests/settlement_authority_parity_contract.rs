use kamn_e2e_harness::{
    verify_settlement_authority_parity, SettlementAuthorityAttempt, SettlementAuthorityDriver,
};
use serde_json::{json, Value};

const ESCROW: &str = "escrow-1";
const ACTOR: &str = "did:kamn:actor";
const IDEMPOTENCY: &str = "operation-1";

#[test]
fn three_drivers_prove_one_authoritative_settlement() {
    let report = verify_settlement_authority_parity(
        ESCROW,
        ACTOR,
        IDEMPOTENCY,
        attempts(fixture()),
        1,
    )
    .expect("complete shared authority should pass");

    assert_eq!(report.escrow_id, ESCROW);
    assert_eq!(report.idempotency_key, IDEMPOTENCY);
    assert_eq!(report.settlement_submissions, 1);
    assert_eq!(report.canonical_authority, canonical_fixture());
}

#[test]
fn missing_tampered_and_cross_resource_authority_fail_closed() {
    let mut missing = fixture();
    missing
        .as_object_mut()
        .expect("fixture object")
        .remove("transaction_signature");
    assert_error(missing, "PI_SERVICE_AUTHORITY_MISMATCH");

    let mut tampered = fixture();
    tampered["receipt_chain_commitment"] = json!("sha256:bad");
    assert_error(tampered, "RECEIPT_CHAIN_INVALID");

    let mut cross_resource = fixture();
    cross_resource["resource_id"] = json!("escrow-2");
    assert_error(cross_resource, "PI_SERVICE_AUTHORITY_MISMATCH");
}

#[test]
fn conflicting_retry_and_duplicate_submission_fail_closed() {
    let mut conflicting = attempts(fixture());
    conflicting[2].response["finalized_slot"] = json!(43);
    let error =
        verify_settlement_authority_parity(ESCROW, ACTOR, IDEMPOTENCY, conflicting, 1)
            .expect_err("same-key different authority must fail");
    assert_eq!(error.code, "SERVICE_AUTHORITY_REPLAY");
    assert_eq!(error.driver, Some(SettlementAuthorityDriver::Mcp));

    let error =
        verify_settlement_authority_parity(ESCROW, ACTOR, IDEMPOTENCY, attempts(fixture()), 2)
            .expect_err("duplicate settlement must fail");
    assert_eq!(error.code, "SERVICE_AUTHORITY_REPLAY");
    assert_eq!(error.field, "settlement_submissions");
}

#[test]
fn missing_driver_and_changed_identity_fail_closed() {
    let mut missing = attempts(fixture());
    missing.pop();
    let error = verify_settlement_authority_parity(ESCROW, ACTOR, IDEMPOTENCY, missing, 1)
        .expect_err("all three drivers are required");
    assert_eq!(error.code, "PI_SERVICE_AUTHORITY_MISMATCH");
    assert_eq!(error.field, "driver_set");

    let error = verify_settlement_authority_parity(
        ESCROW,
        ACTOR,
        "operation-other",
        attempts(fixture()),
        1,
    )
    .expect_err("shared idempotency identity is required");
    assert_eq!(error.code, "PI_SERVICE_AUTHORITY_MISMATCH");
    assert_eq!(error.field, "idempotency_key");
}

fn assert_error(value: Value, expected: &str) {
    let error = verify_settlement_authority_parity(
        ESCROW,
        ACTOR,
        IDEMPOTENCY,
        attempts(value),
        1,
    )
    .expect_err("invalid authority must fail");
    assert_eq!(error.code, expected);
}

fn attempts(response: Value) -> Vec<SettlementAuthorityAttempt> {
    [
        SettlementAuthorityDriver::Sdk,
        SettlementAuthorityDriver::Cli,
        SettlementAuthorityDriver::Mcp,
    ]
    .into_iter()
    .map(|driver| SettlementAuthorityAttempt {
        driver,
        escrow_id: ESCROW.to_owned(),
        idempotency_key: IDEMPOTENCY.to_owned(),
        response: response.clone(),
    })
    .collect()
}

fn canonical_fixture() -> String {
    serde_json::to_string(&fixture()).expect("canonical fixture")
}

fn fixture() -> Value {
    json!({
        "bridge_id": "bridge-1",
        "bridge_receipt_id": "bridge-receipt-1",
        "bridge_receipt_digest": digest('a'),
        "settlement_receipt_id": "settlement-1",
        "settlement_receipt_digest": digest('b'),
        "action": "settlement:confirmed",
        "resource_id": ESCROW,
        "actor_did": ACTOR,
        "resulting_state": "confirmed",
        "task_id": "task-1",
        "escrow_id": ESCROW,
        "recipient": "recipient-1",
        "amount_lamports": 31,
        "asset": "lamports",
        "network": "solana:devnet",
        "transaction_signature": "signature-1",
        "commitment": "finalized",
        "finalized_slot": 42,
        "receipt_chain_commitment": digest('c'),
        "terms_digest": digest('d'),
        "idempotency_key": IDEMPOTENCY,
    })
}

fn digest(value: char) -> String {
    format!("sha256:{}", value.to_string().repeat(64))
}
