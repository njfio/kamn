use super::support_helpers::*;

#[test]
fn spec_c91_verify_command_rejects_missing_infrastructure_kolme_version_marker() {
    let paths = setup_verify_case(
        "missing_infra",
        MISSING_INFRA_KOLME_VERSION_MANIFEST,
        valid_chain_dump_json(),
    );
    let err = execute_verify_contract(&verify_config(&paths))
        .expect_err("verify should fail for missing marker");
    assert!(err.contains("manifest missing infrastructure.kolme_version"));
    cleanup_verify_case(&paths);
}

#[test]
fn spec_c92_verify_command_rejects_missing_summary_proofs_verified_marker() {
    let paths = setup_verify_case(
        "missing_summary",
        MISSING_SUMMARY_PROOFS_VERIFIED_MANIFEST,
        valid_chain_dump_json(),
    );
    let err = execute_verify_contract(&verify_config(&paths))
        .expect_err("verify should fail for missing marker");
    assert!(err.contains("manifest missing summary.proofs_verified"));
    cleanup_verify_case(&paths);
}
