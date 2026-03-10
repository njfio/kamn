use super::super::support_helpers::*;

#[test]
fn spec_c93_verify_command_rejects_evidence_artifact_missing_verification_block() {
    let paths = setup_verify_case_with_artifact(
        "missing_verification_block",
        VALID_MANIFEST,
        r#"{"data":{"agent":"alice"}}"#,
        valid_chain_dump_json(),
    );
    let err = execute_verify_contract(&verify_config(&paths))
        .expect_err("verify should fail for missing marker");
    assert!(err.contains("evidence artifact missing _verification block"));
    cleanup_verify_case(&paths);
}

#[test]
fn spec_c94_verify_command_rejects_evidence_artifact_missing_kolme_anchor_tx_hash() {
    let paths = setup_verify_case_with_artifact(
        "missing_verification_tx_hash",
        VALID_MANIFEST,
        r#"{"data":{"agent":"alice"},"_verification":{"evidence_hash":"sha256:abc123","captured_at":"2026-02-21T14:31:05Z","source_node":"kamn-processor-1","agent":"alice","kolme_anchor":{"block_height":42,"finality":"FINAL"}}}"#,
        valid_chain_dump_json(),
    );
    let err = execute_verify_contract(&verify_config(&paths))
        .expect_err("verify should fail for missing marker");
    assert!(err.contains("evidence artifact missing _verification.kolme_anchor.tx_hash"));
    cleanup_verify_case(&paths);
}

#[test]
fn spec_c95_verify_command_accepts_evidence_artifact_with_complete_verification_block() {
    let paths = setup_verify_case_with_artifact(
        "complete_verification_block",
        VALID_MANIFEST,
        r#"{"data":{"agent":"alice"},"_verification":{"evidence_hash":"sha256:abc123","captured_at":"2026-02-21T14:31:05Z","source_node":"kamn-processor-1","agent":"alice","kolme_anchor":{"tx_hash":"sha256:def456","block_height":42,"finality":"FINAL"}}}"#,
        valid_chain_dump_json(),
    );
    let report = execute_verify_contract(&verify_config(&paths)).expect("verify should succeed");
    assert!(report.contains("\"schema_check\""));
    assert!(report.contains("\"proof_check\""));
    assert!(report.contains("\"chain_check\""));
    assert!(report.contains("\"content_check\""));
    cleanup_verify_case(&paths);
}
