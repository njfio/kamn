use super::super::support_helpers::*;

#[test]
fn spec_c96_verify_command_rejects_chain_dump_missing_chain_name_marker() {
    assert_chain_dump_failure(
        "missing_chain_name",
        r#"{"chain_version":1,"blocks":[]}"#,
        "chain dump missing chain_name marker",
    );
}

#[test]
fn spec_c97_verify_command_rejects_chain_dump_missing_blocks_marker() {
    assert_chain_dump_failure(
        "missing_chain_blocks",
        r#"{"chain_name":"kamn-e2e-devnet","chain_version":1}"#,
        "chain dump missing blocks marker",
    );
}

#[test]
fn spec_c98_verify_command_rejects_chain_dump_block_missing_block_hash_marker() {
    assert_chain_dump_failure(
        "missing_block_hash_marker",
        r#"{"chain_name":"kamn-e2e-devnet","chain_version":1,"blocks":[{"height":0,"previous_block_hash":"GENESIS"}]}"#,
        "chain dump block missing block_hash marker",
    );
}

#[test]
fn spec_c99_verify_command_rejects_chain_dump_block_missing_previous_block_hash_marker() {
    assert_chain_dump_failure(
        "missing_previous_block_hash_marker",
        r#"{"chain_name":"kamn-e2e-devnet","chain_version":1,"blocks":[{"height":0,"block_hash":"sha256:block-0"}]}"#,
        "chain dump block missing previous_block_hash marker",
    );
}

fn assert_chain_dump_failure(case: &str, chain_dump: &str, expected: &str) {
    let paths = setup_verify_case(case, VALID_MANIFEST, chain_dump);
    let err = execute_verify_contract(&verify_config(&paths))
        .expect_err("verify should fail for chain dump marker regression");
    assert!(err.contains(expected));
    cleanup_verify_case(&paths);
}
