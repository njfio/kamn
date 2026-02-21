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
        main_rs.contains("mod runtime_orchestration;"),
        "main.rs should declare runtime_orchestration module"
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
    let runtime_orchestration_rs = read_repo_file("src/runtime_orchestration.rs");
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
        runtime_orchestration_rs.contains("execute_kolme_live_runtime("),
        "runtime_orchestration.rs should own Kolme live runtime branch delegation"
    );
}

#[test]
fn main_module_extraction_contract_removes_inline_runtime_orchestration_impls() {
    let main_rs = read_repo_file("src/main.rs");
    assert!(
        !main_rs.contains("fn execute_daemon_runtime("),
        "main.rs should not keep inline daemon runtime executor"
    );
    assert!(
        !main_rs.contains("fn classify_full_supervisor_stop_contract_violation("),
        "main.rs should not keep inline full supervisor stop classifier"
    );
    assert!(
        !main_rs.contains("fn enforce_kolme_live_signer_contract_policy("),
        "main.rs should not keep inline signer policy enforcement helper"
    );
    assert!(
        !main_rs.contains("fn execute(cli: NodeCli)"),
        "main.rs should delegate runtime execution to runtime_orchestration module"
    );
}

#[test]
fn runtime_orchestration_module_extraction_contract_declares_daemon_phase_module() {
    let runtime_orchestration_rs = read_repo_file("src/runtime_orchestration.rs");
    let daemon_phase_rs = read_repo_file("src/runtime_orchestration/daemon_phase.rs");
    assert!(
        runtime_orchestration_rs.contains("mod daemon_phase;"),
        "runtime_orchestration.rs should declare daemon phase submodule"
    );
    assert!(
        !runtime_orchestration_rs.contains("fn execute_daemon_runtime("),
        "runtime_orchestration.rs should not keep inline daemon runtime executor"
    );
    assert!(
        daemon_phase_rs.contains("pub(super) fn execute_daemon_runtime("),
        "daemon phase module should own daemon runtime execution"
    );
    assert!(
        daemon_phase_rs.contains("pub(super) fn daemon_shutdown_drain_status("),
        "daemon phase module should own shutdown drain status derivation"
    );
}

#[test]
fn main_module_extraction_contract_keeps_impls_in_new_modules() {
    let signer_rs = read_repo_file("src/signer.rs");
    let report_builder_rs = read_repo_file("src/report_builder.rs");
    let report_render_rs = read_repo_file("src/report_render.rs");
    let runtime_kolme_live_rs = read_repo_file("src/runtime_kolme_live.rs");
    let runtime_orchestration_rs = read_repo_file("src/runtime_orchestration.rs");
    let wire_payload_rs = read_repo_file("src/wire_payload.rs");
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
    assert!(
        runtime_orchestration_rs.contains("pub(crate) fn execute(cli: NodeCli)"),
        "runtime_orchestration module should own runtime mode execution dispatch"
    );
    assert!(
        runtime_orchestration_rs.contains("pub(crate) fn validate_full_supervisor_stop_contract("),
        "runtime_orchestration module should own full supervisor stop contract validation"
    );
    assert!(
        runtime_orchestration_rs
            .contains("pub(crate) fn enforce_kolme_live_signer_key_source_policy("),
        "runtime_orchestration module should own signer key-source policy enforcement"
    );
}

#[test]
fn main_module_extraction_contract_runtime_module_boundary_parity_markers_remain_stable() {
    let main_rs = read_repo_file("src/main.rs");
    let runtime_orchestration_rs = read_repo_file("src/runtime_orchestration.rs");
    let daemon_phase_rs = read_repo_file("src/runtime_orchestration/daemon_phase.rs");
    let runtime_kolme_live_rs = read_repo_file("src/runtime_kolme_live.rs");

    assert!(
        main_rs.contains("use runtime_orchestration::{build_runtime_execution_id, execute};"),
        "main.rs should dispatch runtime execution through runtime_orchestration boundary"
    );
    assert!(
        runtime_orchestration_rs.contains("use daemon_phase::execute_daemon_runtime;"),
        "runtime_orchestration should delegate daemon execution to daemon_phase module"
    );
    assert!(
        runtime_orchestration_rs.contains("execute_kolme_live_runtime("),
        "runtime_orchestration should delegate single-cycle Kolme execution to runtime_kolme_live"
    );
    assert!(
        runtime_orchestration_rs.contains("execute_kolme_live_runtime_continuous("),
        "runtime_orchestration should delegate continuous Kolme execution to runtime_kolme_live"
    );
    assert!(
        daemon_phase_rs.contains("pub(super) fn execute_daemon_runtime("),
        "daemon_phase should own daemon runtime execution helper"
    );
    assert!(
        !runtime_orchestration_rs.contains("fn daemon_shutdown_drain_status("),
        "runtime_orchestration should not re-inline daemon shutdown status helper from daemon_phase"
    );
    assert!(
        runtime_kolme_live_rs.contains("pub(crate) fn execute_kolme_live_runtime("),
        "runtime_kolme_live should own single-cycle Kolme execution helper"
    );
    assert!(
        !runtime_orchestration_rs.contains("fn build_kolme_live_request("),
        "runtime_orchestration should not re-inline Kolme request construction helper"
    );
}

#[test]
fn main_module_extraction_contract_main_tests_decomposition_and_budget_markers_remain_stable() {
    let main_tests_rs = read_repo_file("src/main_tests.rs");
    let main_tests_lines = main_tests_rs.lines().count();
    let module_decl_count = main_tests_rs
        .lines()
        .filter(|line| line.trim_start().starts_with("mod "))
        .count();

    assert!(
        main_tests_rs.contains(
            "main_tests structural budget shell only; keep domain tests in src/main_tests/*.rs"
        ),
        "main_tests.rs should carry explicit decomposition drift guard marker"
    );
    assert!(
        !main_tests_rs.contains("#[test]"),
        "main_tests.rs should not re-inline individual test bodies"
    );
    assert!(
        main_tests_lines <= 260,
        "main_tests.rs should remain a bounded shell (<=260 lines)"
    );
    assert!(
        module_decl_count >= 9,
        "main_tests.rs should keep decomposed domain module boundaries"
    );
}

#[test]
fn main_module_extraction_contract_runtime_tests_decomposition_shell_markers_remain_stable() {
    let runtime_tests_rs = read_repo_file("src/main_tests/runtime_tests.rs");
    let runtime_tests_lines = runtime_tests_rs.lines().count();
    let include_decl_count = runtime_tests_rs
        .lines()
        .filter(|line| line.trim_start().starts_with("include!(\"runtime_tests/"))
        .count();

    assert!(
        runtime_tests_rs.contains("runtime_tests structural budget shell only"),
        "runtime_tests.rs should carry explicit decomposition drift guard marker"
    );
    assert!(
        !runtime_tests_rs.contains("#[test]"),
        "runtime_tests.rs should not keep inline test bodies"
    );
    assert!(
        runtime_tests_lines <= 120,
        "runtime_tests.rs should remain a bounded shell (<=120 lines)"
    );
    assert!(
        include_decl_count >= 6,
        "runtime_tests.rs should route tests through focused include fragments"
    );
}

#[test]
fn main_module_extraction_contract_daemon_tests_decomposition_shell_markers_remain_stable() {
    let daemon_tests_rs = read_repo_file("src/main_tests/daemon_tests.rs");
    let daemon_tests_lines = daemon_tests_rs.lines().count();
    let include_decl_count = daemon_tests_rs
        .lines()
        .filter(|line| line.trim_start().starts_with("include!(\"daemon_tests/"))
        .count();

    assert!(
        daemon_tests_rs.contains(
            "daemon_tests structural budget shell phase3; route runtime/matrix/topology contracts"
        ),
        "daemon_tests.rs should carry explicit phase3 decomposition drift guard marker"
    );
    assert!(
        daemon_tests_rs.contains("include!(\"daemon_tests/runtime_contract_tests.rs\");"),
        "daemon_tests.rs should route runtime contract tests through include fragment"
    );
    assert!(
        daemon_tests_rs
            .contains("include!(\"daemon_tests/live_postgres_matrix_contract_tests.rs\");"),
        "daemon_tests.rs should route live-postgres matrix contract tests through include fragment"
    );
    assert!(
        daemon_tests_rs
            .contains("include!(\"daemon_tests/live_postgres_topology_contract_tests.rs\");"),
        "daemon_tests.rs should route topology-heavy contract tests through include fragment"
    );
    assert!(
        daemon_tests_rs.contains(
            "include!(\"daemon_tests/live_postgres_distributed_execution_contract_tests.rs\");"
        ),
        "daemon_tests.rs should route distributed execution contract tests through include fragment"
    );
    assert!(
        !daemon_tests_rs.contains(
            "fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_scope_contract_is_canonical("
        ),
        "daemon_tests.rs should not keep inline topology contract test bodies after phase2 extraction"
    );
    assert!(
        !daemon_tests_rs.contains("fn functional_runtime_daemon_emits_structured_transition_markers("),
        "daemon_tests.rs should not keep inline runtime contract test bodies after phase3 extraction"
    );
    assert!(
        !daemon_tests_rs.contains(
            "fn functional_runtime_daemon_live_postgres_validation_slice_matrix_projection_contract_is_canonical("
        ),
        "daemon_tests.rs should not keep inline live-postgres matrix contract test bodies after phase3 extraction"
    );
    assert!(
        daemon_tests_lines <= 300,
        "daemon_tests.rs should remain within phase3 bounded shell target (<=300 lines)"
    );
    assert!(
        include_decl_count >= 4,
        "daemon_tests.rs should keep include-based decomposition entries"
    );
}
