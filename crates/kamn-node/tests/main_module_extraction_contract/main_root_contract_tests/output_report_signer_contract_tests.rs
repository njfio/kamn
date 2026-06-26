use crate::support::{assert_not_contains_all, read_repo_file};

#[test]
fn main_module_extraction_contract_removes_inline_output_io_impls() {
    let main_rs = read_repo_file("src/main.rs");
    assert_not_contains_all(
        &main_rs,
        &[
            (
                "fn emit_bootstrap_report_output(",
                "main.rs should not keep inline bootstrap report output helper",
            ),
            (
                "fn write_stdout_line(",
                "main.rs should not keep inline stdout writer helper",
            ),
            (
                "fn write_stderr_line(",
                "main.rs should not keep inline stderr writer helper",
            ),
            (
                "fn write_line_to_stream(",
                "main.rs should not keep inline stream writer helper",
            ),
        ],
    );
}

#[test]
fn main_module_extraction_contract_removes_inline_report_rendering_impls() {
    let main_rs = read_repo_file("src/main.rs");
    assert_not_contains_all(
        &main_rs,
        &[
            (
                "fn render_bootstrap_report(",
                "main.rs should not keep inline bootstrap report renderer",
            ),
            (
                "fn render_text_report(",
                "main.rs should not keep inline text report renderer",
            ),
            (
                "fn render_json_report(",
                "main.rs should not keep inline json report renderer",
            ),
            (
                "fn json_escape(",
                "main.rs should not keep inline json escape helper",
            ),
            (
                "fn build_bootstrap_report(",
                "main.rs should not keep inline bootstrap report assembly",
            ),
            (
                "fn build_kolme_live_request(",
                "main.rs should not keep inline Kolme live request builder",
            ),
            (
                "fn ensure_kolme_live_provider_marker(",
                "main.rs should not keep inline Kolme provider marker guard",
            ),
            (
                "fn map_kolme_live_submit_outcome(",
                "main.rs should not keep inline Kolme submit outcome mapper",
            ),
            (
                "fn kolme_live_finality_label(",
                "main.rs should not keep inline Kolme finality label helper",
            ),
        ],
    );
}

#[test]
fn main_module_extraction_contract_removes_inline_signer_payload_impls() {
    let main_rs = read_repo_file("src/main.rs");
    assert_not_contains_all(
        &main_rs,
        &[
            (
                "fn build_kolme_live_direct_signed_wire_payload(",
                "main.rs should not keep inline direct signed payload builder",
            ),
            (
                "fn resolve_kolme_live_nonce(",
                "main.rs should not keep inline nonce resolver",
            ),
            (
                "fn render_kolme_live_native_direct_message(",
                "main.rs should not keep inline native direct message renderer",
            ),
            (
                "fn normalize_kolme_live_signer_profile_selector(",
                "main.rs should not keep inline signer profile normalization helper",
            ),
            (
                "fn normalize_kolme_live_signer_key_source(",
                "main.rs should not keep inline signer key-source normalization helper",
            ),
        ],
    );
}
