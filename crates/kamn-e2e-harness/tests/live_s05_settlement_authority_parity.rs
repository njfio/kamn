use kamn_e2e_harness::{
    verify_settlement_authority_parity, SettlementAuthorityAttempt, SettlementAuthorityDriver,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const INPUT_ENV: &str = "KAMN_E2E_S05_AUTHORITY_PARITY_CAPTURE";
const OUTPUT_ENV: &str = "KAMN_E2E_S05_AUTHORITY_PARITY_REPORT";

#[test]
#[ignore = "requires an explicitly captured funded S-05 parity execution"]
fn integration_live_s05_authority_parity_capture_is_complete() {
    let capture_path = required_path(INPUT_ENV);
    let capture = read_json(capture_path.as_path());
    let escrow = required_string(&capture, "escrow_id");
    let actor = required_string(&capture, "actor_did");
    let idempotency = required_string(&capture, "idempotency_key");
    validate_balance_movement(&capture);
    validate_rpc_artifact(&capture);

    let submissions = required_u64(&capture, "settlement_submissions");
    let report = verify_settlement_authority_parity(
        escrow,
        actor,
        idempotency,
        attempts(&capture, escrow, idempotency),
        submissions,
    )
    .expect("live authority parity capture must verify");
    write_report(&capture, &report.canonical_authority);
}

fn attempts(capture: &Value, escrow: &str, idempotency: &str) -> Vec<SettlementAuthorityAttempt> {
    [
        (SettlementAuthorityDriver::Sdk, "sdk"),
        (SettlementAuthorityDriver::Cli, "cli"),
        (SettlementAuthorityDriver::Mcp, "mcp"),
    ]
    .into_iter()
    .map(|(driver, key)| SettlementAuthorityAttempt {
        driver,
        escrow_id: escrow.to_owned(),
        idempotency_key: idempotency.to_owned(),
        response: capture["attempts"][key].clone(),
    })
    .collect()
}

fn validate_balance_movement(capture: &Value) {
    let payer_before = required_u64(capture, "payer_balance_before");
    let payer_after = required_u64(capture, "payer_balance_after");
    let recipient_before = required_u64(capture, "recipient_balance_before");
    let recipient_after = required_u64(capture, "recipient_balance_after");
    assert!(payer_after < payer_before, "payer balance must decrease");
    assert!(
        recipient_after > recipient_before,
        "recipient balance must increase"
    );
}

fn validate_rpc_artifact(capture: &Value) {
    let path = PathBuf::from(required_string(capture, "authoritative_rpc_artifact"));
    let artifact = read_json(path.as_path());
    assert_eq!(artifact["confirmationStatus"], "finalized");
    assert!(artifact["meta"]["err"].is_null());
    let signature = required_string(capture, "transaction_signature");
    assert!(
        artifact.to_string().contains(signature),
        "RPC artifact must bind the settlement signature"
    );
}

fn write_report(capture: &Value, canonical_authority: &str) {
    let output = std::env::var(OUTPUT_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(".kamn/e2e/live-s05-authority-parity/report.json"));
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).expect("create live parity report directory");
    }
    let report = json!({
        "status": "PASS",
        "proof_kind": "executed-funded-live",
        "escrow_id": capture["escrow_id"],
        "idempotency_key": capture["idempotency_key"],
        "settlement_submissions": capture["settlement_submissions"],
        "transaction_signature": capture["transaction_signature"],
        "authoritative_rpc_artifact": capture["authoritative_rpc_artifact"],
        "canonical_authority": canonical_authority,
    });
    std::fs::write(
        output,
        serde_json::to_vec_pretty(&report).expect("report json"),
    )
    .expect("write live parity report");
}

fn required_path(name: &str) -> PathBuf {
    let value = std::env::var(name).unwrap_or_else(|_| panic!("required env missing: {name}"));
    assert!(!value.trim().is_empty(), "required env empty: {name}");
    PathBuf::from(value)
}

fn read_json(path: &Path) -> Value {
    let bytes = std::fs::read(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    serde_json::from_slice(bytes.as_slice())
        .unwrap_or_else(|error| panic!("invalid JSON in {}: {error}", path.display()))
}

fn required_string<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key]
        .as_str()
        .filter(|item| !item.trim().is_empty())
        .unwrap_or_else(|| panic!("capture missing non-empty {key}"))
}

fn required_u64(value: &Value, key: &str) -> u64 {
    value[key]
        .as_u64()
        .unwrap_or_else(|| panic!("capture missing integer {key}"))
}
