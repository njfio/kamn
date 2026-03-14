use super::{
    generate_verification_report_json, validate_evidence_verification_blocks, verify_chain_dump,
    verify_manifest,
};
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_evidence_dir(label: &str) -> std::path::PathBuf {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be monotonic for tests")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "kamn-e2e-{label}-{}-{}",
        std::process::id(),
        unique_suffix
    ))
}

fn write_artifact(dir: &std::path::Path, filename: &str, body: &str) {
    std::fs::create_dir_all(dir.join("s01-agent-discovery"))
        .expect("evidence directory should be created");
    std::fs::write(dir.join("s01-agent-discovery").join(filename), body)
        .expect("artifact should be written");
}

fn cleanup(dir: &std::path::Path) {
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn unit_verify_manifest_rejects_missing_schema_marker() {
    let result = verify_manifest(r#"{"execution_mode":"sdk-direct","scenarios":[]}"#);
    assert!(result.is_err());
}

#[test]
fn unit_generate_verification_report_json_is_deterministic() {
    let manifest = r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","run_id":"e2e-run","started_at":"2026-02-21T14:30:52Z","completed_at":"2026-02-21T14:35:12Z","duration_seconds":260,"execution_mode":"sdk-direct","infrastructure":{"kolme_version":"0.x.y","kamn_version":"0.1.0","kamn_commit":"49efe252","kamn_agent_lib_version":"0.1.0","agent_runtime":"sdk-direct","node_count":3,"agent_count":3,"storage_backend":"sqlite+postgres"},"scenarios":[],"summary":{"total_scenarios":15,"passed":13,"failed":1,"skipped":1,"kolme_blocks_produced":47,"messages_exchanged":128,"proofs_anchored":47,"proofs_verified":47}}"#;
    let first = generate_verification_report_json(manifest).expect("report should build");
    let second = generate_verification_report_json(manifest).expect("report should build");
    assert_eq!(first, second);
    assert!(first.contains("\"schema_check\""));
    assert!(first.contains("\"proof_check\""));
    assert!(first.contains("\"chain_check\""));
    assert!(first.contains("\"content_check\""));
}

#[test]
fn unit_verify_chain_dump_rejects_missing_block_hash_marker() {
    let err = verify_chain_dump(
        r#"{"chain_name":"kamn-e2e-devnet","chain_version":1,"blocks":[{"height":0,"previous_block_hash":"GENESIS"}]}"#,
    )
    .expect_err("missing block hash marker should fail");
    assert!(err.contains("chain dump block missing block_hash marker"));
}

#[test]
fn unit_verify_chain_dump_rejects_hash_continuity_mismatch() {
    let err = verify_chain_dump(
        r#"{"chain_name":"kamn-e2e-devnet","chain_version":1,"blocks":[{"height":0,"block_hash":"sha256:block-0","previous_block_hash":"GENESIS"},{"height":1,"block_hash":"sha256:block-1","previous_block_hash":"sha256:wrong-prior"}]}"#,
    )
    .expect_err("chain continuity mismatch should fail");
    assert!(err.contains("chain dump hash continuity mismatch at block index 1"));
}

#[test]
fn unit_verify_chain_dump_rejects_genesis_anchor_mismatch() {
    let err = verify_chain_dump(
        r#"{"chain_name":"kamn-e2e-devnet","chain_version":1,"blocks":[{"height":0,"block_hash":"sha256:block-0","previous_block_hash":"sha256:not-genesis"},{"height":1,"block_hash":"sha256:block-1","previous_block_hash":"sha256:block-0"}]}"#,
    )
    .expect_err("genesis anchor mismatch should fail");
    assert!(err.contains("chain dump genesis anchor mismatch at block index 0"));
}

#[test]
fn unit_validate_evidence_verification_blocks_rejects_non_final_finality_value() {
    let evidence_dir = unique_evidence_dir("finality-value");
    write_artifact(
        evidence_dir.as_path(),
        "alice_registration.json",
        r#"{"data":{"agent":"alice"},"_verification":{"evidence_hash":"sha256:abc123","captured_at":"2026-02-21T14:31:05Z","source_node":"kamn-processor-1","agent":"alice","kolme_anchor":{"tx_hash":"sha256:def456","block_height":42,"finality":"SOFT"}}}"#,
    );
    let err = validate_evidence_verification_blocks(&evidence_dir, &[])
        .expect_err("non-final finality value should fail");
    assert!(err.contains("evidence artifact invalid _verification.kolme_anchor.finality value"));
    cleanup(evidence_dir.as_path());
}

#[test]
fn unit_validate_evidence_verification_blocks_rejects_invalid_evidence_hash_format() {
    let evidence_dir = unique_evidence_dir("evidence-hash-format");
    write_artifact(
        evidence_dir.as_path(),
        "alice_registration.json",
        r#"{"data":{"agent":"alice"},"_verification":{"evidence_hash":"abc123","captured_at":"2026-02-21T14:31:05Z","source_node":"kamn-processor-1","agent":"alice","kolme_anchor":{"tx_hash":"sha256:def456","block_height":42,"finality":"FINAL"}}}"#,
    );
    let err = validate_evidence_verification_blocks(&evidence_dir, &[])
        .expect_err("invalid evidence hash format should fail");
    assert!(err.contains("evidence artifact invalid _verification.evidence_hash format"));
    cleanup(evidence_dir.as_path());
}

#[test]
fn unit_validate_evidence_verification_blocks_rejects_invalid_anchor_tx_hash_format() {
    let evidence_dir = unique_evidence_dir("anchor-tx-hash-format");
    write_artifact(
        evidence_dir.as_path(),
        "alice_registration.json",
        r#"{"data":{"agent":"alice"},"_verification":{"evidence_hash":"sha256:abc123","captured_at":"2026-02-21T14:31:05Z","source_node":"kamn-processor-1","agent":"alice","kolme_anchor":{"tx_hash":"def456","block_height":42,"finality":"FINAL"}}}"#,
    );
    let err = validate_evidence_verification_blocks(&evidence_dir, &[])
        .expect_err("invalid anchor tx hash format should fail");
    assert!(err.contains("evidence artifact invalid _verification.kolme_anchor.tx_hash format"));
    cleanup(evidence_dir.as_path());
}

#[test]
fn unit_validate_evidence_verification_blocks_rejects_invalid_anchor_block_height_format() {
    let evidence_dir = unique_evidence_dir("anchor-height-format");
    write_artifact(
        evidence_dir.as_path(),
        "alice_registration.json",
        r#"{"data":{"agent":"alice"},"_verification":{"evidence_hash":"sha256:abc123","captured_at":"2026-02-21T14:31:05Z","source_node":"kamn-processor-1","agent":"alice","kolme_anchor":{"tx_hash":"sha256:def456","block_height":"forty-two","finality":"FINAL"}}}"#,
    );
    let err = validate_evidence_verification_blocks(&evidence_dir, &[])
        .expect_err("invalid block_height format should fail");
    assert!(
        err.contains("evidence artifact invalid _verification.kolme_anchor.block_height format")
    );
    cleanup(evidence_dir.as_path());
}

#[test]
fn unit_validate_evidence_verification_blocks_rejects_invalid_captured_at_format() {
    let evidence_dir = unique_evidence_dir("captured-at-format");
    write_artifact(
        evidence_dir.as_path(),
        "alice_registration.json",
        r#"{"data":{"agent":"alice"},"_verification":{"evidence_hash":"sha256:abc123","captured_at":"2026/02/21 14:31:05","source_node":"kamn-processor-1","agent":"alice","kolme_anchor":{"tx_hash":"sha256:def456","block_height":42,"finality":"FINAL"}}}"#,
    );
    let err = validate_evidence_verification_blocks(&evidence_dir, &[])
        .expect_err("invalid captured_at format should fail");
    assert!(err.contains("evidence artifact invalid _verification.captured_at format"));
    cleanup(evidence_dir.as_path());
}
