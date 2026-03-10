use super::super::support_helpers::*;

#[test]
fn spec_c102_verify_command_rejects_evidence_artifact_non_final_kolme_anchor_finality() {
    assert_evidence_format_failure(
        "invalid_finality_value",
        r#"{"data":{"agent":"alice"},"_verification":{"evidence_hash":"sha256:abc123","captured_at":"2026-02-21T14:31:05Z","source_node":"kamn-processor-1","agent":"alice","kolme_anchor":{"tx_hash":"sha256:def456","block_height":42,"finality":"PENDING"}}}"#,
        "evidence artifact invalid _verification.kolme_anchor.finality value",
    );
}

#[test]
fn spec_c103_verify_command_rejects_evidence_artifact_invalid_evidence_hash_format() {
    assert_evidence_format_failure(
        "invalid_evidence_hash_format",
        r#"{"data":{"agent":"alice"},"_verification":{"evidence_hash":"abc123","captured_at":"2026-02-21T14:31:05Z","source_node":"kamn-processor-1","agent":"alice","kolme_anchor":{"tx_hash":"sha256:def456","block_height":42,"finality":"FINAL"}}}"#,
        "evidence artifact invalid _verification.evidence_hash format",
    );
}

#[test]
fn spec_c104_verify_command_rejects_evidence_artifact_invalid_anchor_tx_hash_format() {
    assert_evidence_format_failure(
        "invalid_anchor_tx_hash_format",
        r#"{"data":{"agent":"alice"},"_verification":{"evidence_hash":"sha256:abc123","captured_at":"2026-02-21T14:31:05Z","source_node":"kamn-processor-1","agent":"alice","kolme_anchor":{"tx_hash":"def456","block_height":42,"finality":"FINAL"}}}"#,
        "evidence artifact invalid _verification.kolme_anchor.tx_hash format",
    );
}

#[test]
fn spec_c105_verify_command_rejects_evidence_artifact_invalid_anchor_block_height_format() {
    assert_evidence_format_failure(
        "invalid_anchor_block_height_format",
        r#"{"data":{"agent":"alice"},"_verification":{"evidence_hash":"sha256:abc123","captured_at":"2026-02-21T14:31:05Z","source_node":"kamn-processor-1","agent":"alice","kolme_anchor":{"tx_hash":"sha256:def456","block_height":"forty-two","finality":"FINAL"}}}"#,
        "evidence artifact invalid _verification.kolme_anchor.block_height format",
    );
}

#[test]
fn spec_c106_verify_command_rejects_evidence_artifact_invalid_captured_at_format() {
    assert_evidence_format_failure(
        "invalid_captured_at_format",
        r#"{"data":{"agent":"alice"},"_verification":{"evidence_hash":"sha256:abc123","captured_at":"2026/02/21 14:31:05","source_node":"kamn-processor-1","agent":"alice","kolme_anchor":{"tx_hash":"sha256:def456","block_height":42,"finality":"FINAL"}}}"#,
        "evidence artifact invalid _verification.captured_at format",
    );
}

fn assert_evidence_format_failure(case: &str, artifact: &str, expected: &str) {
    let paths =
        setup_verify_case_with_artifact(case, VALID_MANIFEST, artifact, valid_chain_dump_json());
    let err = execute_verify_contract(&verify_config(&paths))
        .expect_err("verify should fail for invalid evidence format");
    assert!(err.contains(expected));
    cleanup_verify_case(&paths);
}
