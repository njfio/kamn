use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const EVIDENCE: &str = "docs/validation/evidence/7162-external-a2a-x402-observation.json";
const RUNBOOK: &str = "docs/validation/external-a2a-x402-receipt-authority-probe.md";

#[test]
fn observed_request_digest_is_recomputable() {
    let evidence = read_json();
    let body = text(&evidence, "/request/canonical_body");
    let expected = format!("sha256:{:x}", Sha256::digest(body.as_bytes()));
    assert_eq!(text(&evidence, "/request/body_sha256"), expected);
}

#[test]
fn discovery_and_challenge_payment_terms_are_consistent() {
    let evidence = read_json();
    for field in ["network", "asset", "pay_to"] {
        assert_equal_paths(
            &evidence,
            &format!("/discovery/{field}"),
            &format!("/challenge/{field}"),
        );
    }
    assert_equal_paths(&evidence, "/discovery/price_usd", "/challenge/amount_usd");
}

#[test]
fn blocked_verdict_is_supported_by_observed_stages() {
    let evidence = read_json();
    assert_eq!(text(&evidence, "/verdict"), "BLOCKED");
    assert_eq!(boolean(&evidence, "/challenge/observed"), true);
    assert_eq!(boolean(&evidence, "/approval/observed"), false);
    assert_eq!(boolean(&evidence, "/settlement/observed"), false);
    assert_eq!(boolean(&evidence, "/service_result/observed"), false);
    assert!(text(&evidence, "/verdict_reason").contains("no-funds"));
}

#[test]
fn evidence_preserves_no_funds_and_secret_safe_boundary() {
    let evidence = read_json();
    assert_eq!(boolean(&evidence, "/safety/funds_spent"), false);
    assert_eq!(boolean(&evidence, "/safety/credentials_used"), false);
    assert_eq!(boolean(&evidence, "/safety/mark_paid_called"), false);
    let raw = serde_json::to_string(&evidence).expect("evidence should serialize");
    for forbidden in ["private_key", "authorization", "payment-signature"] {
        assert!(!raw.to_lowercase().contains(forbidden), "found {forbidden}");
    }
}

#[test]
fn runbook_reports_evidence_boundaries() {
    let runbook = read_file(RUNBOOK);
    for marker in [
        "# External A2A x402 Receipt-Authority Probe",
        "Verdict: BLOCKED",
        "unpaid x402 challenge",
        "request body digest",
        "no approval response was observed",
        "no settlement response was observed",
        "not KAMN service authority",
        "not production readiness",
    ] {
        assert!(runbook.contains(marker), "{RUNBOOK} missing marker: {marker}");
    }
}

fn assert_equal_paths(value: &Value, left: &str, right: &str) {
    assert_eq!(
        value.pointer(left),
        value.pointer(right),
        "mismatched evidence paths: {left} and {right}"
    );
}

fn text<'a>(value: &'a Value, path: &str) -> &'a str {
    value
        .pointer(path)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("expected string at {path}"))
}

fn boolean(value: &Value, path: &str) -> bool {
    value
        .pointer(path)
        .and_then(Value::as_bool)
        .unwrap_or_else(|| panic!("expected bool at {path}"))
}

fn read_json() -> Value {
    serde_json::from_str(&read_file(EVIDENCE))
        .unwrap_or_else(|error| panic!("{EVIDENCE} should contain JSON: {error}"))
}

fn read_file(relative: &str) -> String {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
