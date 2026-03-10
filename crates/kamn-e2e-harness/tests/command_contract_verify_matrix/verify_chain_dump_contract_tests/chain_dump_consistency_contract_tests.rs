use super::super::support_helpers::*;

#[test]
fn spec_c100_verify_command_rejects_chain_dump_hash_continuity_mismatch() {
    let paths = setup_verify_case(
        "chain_hash_continuity_mismatch",
        VALID_MANIFEST,
        r#"{"chain_name":"kamn-e2e-devnet","chain_version":1,"blocks":[{"height":0,"block_hash":"sha256:block-0","previous_block_hash":"GENESIS"},{"height":1,"block_hash":"sha256:block-1","previous_block_hash":"sha256:wrong-prior-block"}]}"#,
    );
    expect_verify_failure(
        &paths,
        "verify should fail for chain continuity mismatch",
        "chain dump hash continuity mismatch at block index 1",
    );
    cleanup_verify_case(&paths);
}

#[test]
fn spec_c101_verify_command_rejects_chain_dump_genesis_anchor_mismatch() {
    let paths = setup_verify_case(
        "chain_genesis_anchor_mismatch",
        VALID_MANIFEST,
        r#"{"chain_name":"kamn-e2e-devnet","chain_version":1,"blocks":[{"height":0,"block_hash":"sha256:block-0","previous_block_hash":"sha256:not-genesis"},{"height":1,"block_hash":"sha256:block-1","previous_block_hash":"sha256:block-0"}]}"#,
    );
    expect_verify_failure(
        &paths,
        "verify should fail for chain genesis anchor mismatch",
        "chain dump genesis anchor mismatch at block index 0",
    );
    cleanup_verify_case(&paths);
}
