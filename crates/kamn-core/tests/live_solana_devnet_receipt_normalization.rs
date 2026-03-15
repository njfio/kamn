use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use kamn_core::{
    normalize_cross_chain_receipt, CrossChainReceiptFinality, CrossChainReceiptNetwork,
    CrossChainReceiptProof, CrossChainReceiptStatus,
};
use serde_json::{json, Value};

const REPORT_ENV: &str = "KAMN_SOLANA_DEVNET_REPORT_FILE";
const OUTPUT_ENV: &str = "KAMN_SOLANA_DEVNET_NORMALIZATION_REPORT";
const SCHEMA_VERSION: &str = "kamn.solana.devnet.live-normalization-report.v1";

#[test]
fn live_report_normalizes_observed_labels() {
    let Some(report_path) = env::var_os(REPORT_ENV) else {
        eprintln!("skipping live Solana devnet normalization test; {REPORT_ENV} is not set");
        return;
    };
    let report_path = PathBuf::from(report_path);
    let proofs = live_proofs(&report_path);
    let normalized = normalize_finalities(proofs);
    assert_expected_finalities(&normalized);
    write_artifact_if_requested(&report_path, &normalized);
}

fn required_str<'a>(value: &'a Value, field: &str) -> &'a str {
    value[field]
        .as_str()
        .unwrap_or_else(|| panic!("missing string field: {field}"))
}

fn live_proofs(report_path: &PathBuf) -> Vec<Value> {
    let payload = fs::read_to_string(report_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", report_path.display()));
    let report: Value = serde_json::from_str(&payload)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", report_path.display()));
    report["receipt_proofs"]
        .as_array()
        .unwrap_or_else(|| panic!("{} missing receipt_proofs array", report_path.display()))
        .clone()
}

fn normalize_finalities(proofs: Vec<Value>) -> BTreeMap<String, CrossChainReceiptFinality> {
    let mut normalized = BTreeMap::new();
    for proof in proofs {
        let label = required_str(&proof, "finality_label");
        let receipt = live_receipt(&proof, label);
        let view = normalize_cross_chain_receipt(&receipt)
            .unwrap_or_else(|error| panic!("failed to normalize {label}: {error:?}"));
        normalized.insert(label.to_owned(), view.finality);
    }
    normalized
}

fn live_receipt(proof: &Value, label: &str) -> CrossChainReceiptProof {
    CrossChainReceiptProof {
        network: CrossChainReceiptNetwork::Solana,
        receipt_id: required_str(proof, "receipt_id").to_owned(),
        block_reference: required_str(proof, "block_reference").to_owned(),
        finality_label: label.to_owned(),
        confirmation_count: 0,
        status: parse_status(required_str(proof, "status")),
    }
}

fn assert_expected_finalities(normalized: &BTreeMap<String, CrossChainReceiptFinality>) {
    assert_eq!(
        normalized.get("processed"),
        Some(&CrossChainReceiptFinality::Pending)
    );
    assert_eq!(
        normalized.get("confirmed"),
        Some(&CrossChainReceiptFinality::Pending)
    );
    assert_eq!(
        normalized.get("finalized"),
        Some(&CrossChainReceiptFinality::Final)
    );
}

fn write_artifact_if_requested(
    report_path: &PathBuf,
    normalized: &BTreeMap<String, CrossChainReceiptFinality>,
) {
    let Some(output_path) = env::var_os(OUTPUT_ENV) else {
        return;
    };
    let output_path = PathBuf::from(output_path);
    let artifact = normalization_artifact(report_path, normalized);
    write_artifact(&output_path, artifact);
}

fn normalization_artifact(
    report_path: &PathBuf,
    normalized: &BTreeMap<String, CrossChainReceiptFinality>,
) -> Value {
    let normalized_finalities = normalized_finality_strings(normalized);
    json!({
        "schema_version": SCHEMA_VERSION,
        "status": "ok",
        "assertions_passed": true,
        "source_report_file": report_path,
        "normalized_finalities": normalized_finalities,
    })
}

fn normalized_finality_strings(
    normalized: &BTreeMap<String, CrossChainReceiptFinality>,
) -> BTreeMap<String, String> {
    normalized
        .iter()
        .map(|(label, finality)| (label.clone(), format!("{finality:?}")))
        .collect::<BTreeMap<_, _>>()
}

fn write_artifact(output_path: &PathBuf, artifact: Value) {
    fs::write(
        output_path,
        serde_json::to_string_pretty(&artifact).expect("normalization artifact should serialize")
            + "\n",
    )
    .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display()));
}

fn parse_status(status: &str) -> CrossChainReceiptStatus {
    match status {
        "success" => CrossChainReceiptStatus::Success,
        "pending" => CrossChainReceiptStatus::Pending,
        "failed" => CrossChainReceiptStatus::Failed,
        _ => panic!("unsupported live status: {status}"),
    }
}

#[test]
fn live_report_contract_keeps_expected_labels() {
    assert_eq!(format!("{:?}", CrossChainReceiptFinality::Final), "Final");
}
