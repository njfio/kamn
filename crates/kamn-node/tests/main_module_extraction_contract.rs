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
}

#[test]
fn main_module_extraction_contract_keeps_impls_in_new_modules() {
    let signer_rs = read_repo_file("src/signer.rs");
    let report_render_rs = read_repo_file("src/report_render.rs");
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
        wire_payload_rs.contains("pub(crate) fn render_kolme_live_native_direct_message("),
        "wire_payload module should own native direct message renderer"
    );
    assert!(
        report_render_rs.contains("pub(crate) fn render_bootstrap_report("),
        "report_render module should own bootstrap report rendering"
    );
}
