use std::fs;

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

#[test]
fn main_module_extraction_contract_declares_signer_and_wire_modules() {
    let main_rs = read_repo_file("src/main.rs");
    assert!(
        main_rs.contains("mod signer;"),
        "main.rs should declare signer module"
    );
    assert!(
        main_rs.contains("mod wire_payload;"),
        "main.rs should declare wire_payload module"
    );
    assert!(
        main_rs.contains("mod report_render;"),
        "main.rs should declare report_render module"
    );
    assert!(
        main_rs.contains("mod report_builder;"),
        "main.rs should declare report_builder module"
    );
    assert!(
        main_rs.contains("mod runtime_kolme_live;"),
        "main.rs should declare runtime_kolme_live module"
    );
    assert!(
        main_rs.contains("mod main_tests;"),
        "main.rs should declare sidecar test module for maintainability"
    );
    assert!(
        !main_rs.contains("mod tests {"),
        "main.rs should not keep inline monolithic tests module"
    );
}

#[test]
fn main_module_extraction_contract_removes_inline_report_rendering_impls() {
    let main_rs = read_repo_file("src/main.rs");
    assert!(
        !main_rs.contains("fn render_bootstrap_report("),
        "main.rs should not keep inline bootstrap report renderer"
    );
    assert!(
        !main_rs.contains("fn render_text_report("),
        "main.rs should not keep inline text report renderer"
    );
    assert!(
        !main_rs.contains("fn render_json_report("),
        "main.rs should not keep inline json report renderer"
    );
    assert!(
        !main_rs.contains("fn json_escape("),
        "main.rs should not keep inline json escape helper"
    );
    assert!(
        !main_rs.contains("fn build_bootstrap_report("),
        "main.rs should not keep inline bootstrap report assembly"
    );
    assert!(
        !main_rs.contains("fn build_kolme_live_request("),
        "main.rs should not keep inline Kolme live request builder"
    );
    assert!(
        !main_rs.contains("fn ensure_kolme_live_provider_marker("),
        "main.rs should not keep inline Kolme provider marker guard"
    );
    assert!(
        !main_rs.contains("fn map_kolme_live_submit_outcome("),
        "main.rs should not keep inline Kolme submit outcome mapper"
    );
    assert!(
        !main_rs.contains("fn kolme_live_finality_label("),
        "main.rs should not keep inline Kolme finality label helper"
    );
}

#[test]
fn main_module_extraction_contract_removes_inline_signer_payload_impls() {
    let main_rs = read_repo_file("src/main.rs");
    assert!(
        !main_rs.contains("fn build_kolme_live_direct_signed_wire_payload("),
        "main.rs should not keep inline direct signed payload builder"
    );
    assert!(
        !main_rs.contains("fn resolve_kolme_live_nonce("),
        "main.rs should not keep inline nonce resolver"
    );
    assert!(
        !main_rs.contains("fn render_kolme_live_native_direct_message("),
        "main.rs should not keep inline native direct message renderer"
    );
    assert!(
        !main_rs.contains("fn normalize_kolme_live_signer_profile_selector("),
        "main.rs should not keep inline signer profile normalization helper"
    );
    assert!(
        !main_rs.contains("fn normalize_kolme_live_signer_key_source("),
        "main.rs should not keep inline signer key-source normalization helper"
    );
}

#[test]
fn main_module_extraction_contract_removes_inline_kolme_live_branch_execution_impls() {
    let main_rs = read_repo_file("src/main.rs");
    assert!(
        !main_rs.contains("KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile("),
        "main.rs should not keep inline Kolme live provider constructor path"
    );
    assert!(
        !main_rs.contains(
            "submit_runtime_commit(signed_wire_payload.as_str(), request.idempotency_key())"
        ),
        "main.rs should not keep inline Kolme live submit invocation"
    );
    assert!(
        !main_rs.contains("KolmeRuntimeCommitFinalityChecker::new("),
        "main.rs should not keep inline Kolme live finality checker orchestration"
    );
    assert!(
        main_rs.contains("execute_kolme_live_runtime("),
        "main.rs should delegate Kolme live runtime branch to extracted module function"
    );
}

#[test]
fn main_module_extraction_contract_keeps_impls_in_new_modules() {
    let signer_rs = read_repo_file("src/signer.rs");
    let report_builder_rs = read_repo_file("src/report_builder.rs");
    let report_render_rs = read_repo_file("src/report_render.rs");
    let runtime_kolme_live_rs = read_repo_file("src/runtime_kolme_live.rs");
    let wire_payload_rs = read_repo_file("src/wire_payload.rs");
    assert!(
        signer_rs.contains("pub(crate) fn build_kolme_live_direct_signed_wire_payload("),
        "signer module should own direct signed payload builder"
    );
    assert!(
        signer_rs.contains("pub(crate) fn resolve_kolme_live_nonce("),
        "signer module should own nonce resolver"
    );
    assert!(
        signer_rs.contains("pub(crate) fn normalize_kolme_live_signer_profile_selector("),
        "signer module should own signer profile normalization helper"
    );
    assert!(
        signer_rs.contains("pub(crate) fn normalize_kolme_live_signer_key_source("),
        "signer module should own signer key-source normalization helper"
    );
    assert!(
        wire_payload_rs.contains("pub(crate) fn render_kolme_live_native_direct_message("),
        "wire_payload module should own native direct message renderer"
    );
    assert!(
        report_render_rs.contains("pub(crate) fn render_bootstrap_report("),
        "report_render module should own bootstrap report rendering"
    );
    assert!(
        report_builder_rs.contains("pub(crate) fn build_bootstrap_report("),
        "report_builder module should own bootstrap report assembly"
    );
    assert!(
        runtime_kolme_live_rs.contains("pub(crate) fn build_kolme_live_request("),
        "runtime_kolme_live module should own request builder"
    );
    assert!(
        runtime_kolme_live_rs.contains("pub(crate) fn ensure_kolme_live_provider_marker("),
        "runtime_kolme_live module should own provider marker guard"
    );
    assert!(
        runtime_kolme_live_rs.contains("pub(crate) fn map_kolme_live_submit_outcome("),
        "runtime_kolme_live module should own submit outcome mapper"
    );
    assert!(
        runtime_kolme_live_rs.contains("pub(crate) fn execute_kolme_live_runtime("),
        "runtime_kolme_live module should own Kolme live runtime branch execution"
    );
}
