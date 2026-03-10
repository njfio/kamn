use crate::support::read_repo_file;

#[test]
fn main_module_extraction_contract_keeps_impls_in_new_modules() {
    let signer_rs = read_repo_file("src/signer.rs");
    let report_builder_rs = read_repo_file("src/report_builder.rs");
    let report_render_rs = read_repo_file("src/report_render.rs");
    let runtime_kolme_live_rs = read_repo_file("src/runtime_kolme_live.rs");
    let runtime_orchestration_rs = read_repo_file("src/runtime_orchestration.rs");
    let wire_payload_rs = read_repo_file("src/wire_payload.rs");

    assert_signer_impls(&signer_rs);
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
    assert_runtime_impls(&runtime_kolme_live_rs, &runtime_orchestration_rs);
}

fn assert_signer_impls(signer_rs: &str) {
    assert!(
        signer_rs.contains("pub(crate) fn build_kolme_live_direct_signed_wire_payload("),
        "signer module should own direct signed payload builder"
    );
    assert!(
        signer_rs.contains("pub(crate) fn resolve_kolme_live_nonce(")
            || signer_rs.contains("pub(crate) use nonce::resolve_kolme_live_nonce;"),
        "signer module should own nonce resolver"
    );
    assert!(
        signer_rs.contains("pub(crate) fn normalize_kolme_live_signer_profile_selector(")
            || signer_rs.contains("pub(crate) use signer_policy::")
                && signer_rs.contains("normalize_kolme_live_signer_profile_selector"),
        "signer module should own signer profile normalization helper"
    );
    assert!(
        signer_rs.contains("pub(crate) fn normalize_kolme_live_signer_key_source(")
            || signer_rs.contains("pub(crate) use signer_policy::")
                && signer_rs.contains("normalize_kolme_live_signer_key_source"),
        "signer module should own signer key-source normalization helper"
    );
}

fn assert_runtime_impls(runtime_kolme_live_rs: &str, runtime_orchestration_rs: &str) {
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
    assert!(
        runtime_orchestration_rs.contains("pub(crate) fn execute(cli: NodeCli)"),
        "runtime_orchestration module should own runtime mode execution dispatch"
    );
    assert!(
        runtime_orchestration_rs.contains("pub(crate) fn validate_full_supervisor_stop_contract(")
            || runtime_orchestration_rs.contains(
                "pub(crate) use runtime_policy_contracts::validate_full_supervisor_stop_contract;",
            ),
        "runtime_orchestration module should surface full supervisor stop contract validation"
    );
    assert!(
        runtime_orchestration_rs.contains("pub(crate) fn enforce_kolme_live_signer_key_source_policy(")
            || runtime_orchestration_rs.contains(
                "pub(crate) use runtime_policy_contracts::enforce_kolme_live_signer_key_source_policy;",
            ),
        "runtime_orchestration module should surface signer key-source policy enforcement"
    );
}
