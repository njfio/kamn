use crate::support::{assert_contains_all, assert_not_contains_all, read_repo_file};

pub(super) fn read_runtime_file(path: &str) -> String {
    read_repo_file(&format!("src/{path}"))
}

pub(super) fn assert_runtime_orchestration_root_contracts(runtime_orchestration_rs: &str) {
    assert_contains_all(
        runtime_orchestration_rs,
        &[
            (
                "mod daemon_phase;",
                "runtime_orchestration.rs should declare daemon phase submodule",
            ),
            (
                "mod full_supervisor;",
                "runtime_orchestration.rs should declare full_supervisor submodule",
            ),
            (
                "mod runtime_policy_contracts;",
                "runtime_orchestration.rs should declare runtime_policy_contracts submodule",
            ),
            (
                "mod runtime_mode_handlers;",
                "runtime_orchestration.rs should declare runtime_mode_handlers submodule",
            ),
        ],
    );
    assert_runtime_orchestration_no_inline_helpers(runtime_orchestration_rs);
    assert_runtime_orchestration_no_inline_branches(runtime_orchestration_rs);
}

pub(super) fn assert_runtime_entrypoint_boundary(main_rs: &str, runtime_orchestration_rs: &str) {
    assert!(
        main_rs.contains("use runtime_orchestration::{build_runtime_execution_id, execute};"),
        "main.rs should dispatch runtime execution through runtime_orchestration boundary"
    );
    assert!(
        runtime_orchestration_rs.contains("use daemon_phase::execute_daemon_runtime;"),
        "runtime_orchestration should delegate daemon execution to daemon_phase module"
    );
}

pub(super) fn assert_runtime_mode_delegation(
    runtime_orchestration_rs: &str,
    runtime_mode_handlers_rs: &str,
    daemon_phase_rs: &str,
) {
    assert!(
        runtime_orchestration_rs.contains("execute_kolme_live_runtime(")
            || runtime_mode_handlers_rs.contains("execute_kolme_live_runtime("),
        "runtime orchestration boundary should delegate single-cycle Kolme execution to runtime_kolme_live"
    );
    assert!(
        daemon_phase_rs.contains("pub(super) fn execute_daemon_runtime("),
        "daemon_phase should own daemon runtime execution helper"
    );
}

pub(super) fn assert_kolme_live_boundary(
    runtime_orchestration_rs: &str,
    runtime_mode_handlers_rs: &str,
) {
    assert!(
        runtime_orchestration_rs.contains("execute_kolme_live_runtime_continuous(")
            || runtime_mode_handlers_rs.contains("execute_kolme_live_runtime_continuous("),
        "runtime orchestration boundary should delegate continuous Kolme execution to runtime_kolme_live"
    );
    assert!(
        !runtime_orchestration_rs.contains("fn daemon_shutdown_drain_status("),
        "runtime_orchestration should not re-inline daemon shutdown status helper from daemon_phase"
    );
}

pub(super) fn assert_kolme_live_impl_boundary(
    runtime_orchestration_rs: &str,
    runtime_kolme_live_rs: &str,
) {
    assert!(
        runtime_kolme_live_rs.contains("pub(crate) fn execute_kolme_live_runtime("),
        "runtime_kolme_live should own single-cycle Kolme execution helper"
    );
    assert!(
        !runtime_orchestration_rs.contains("fn build_kolme_live_request("),
        "runtime_orchestration should not re-inline Kolme request construction helper"
    );
}

pub(super) fn assert_runtime_submodules(
    daemon_phase_rs: &str,
    full_supervisor_rs: &str,
    runtime_policy_contracts_rs: &str,
    runtime_mode_handlers_rs: &str,
) {
    assert_daemon_phase_exports(daemon_phase_rs);
    assert_full_supervisor_exports(full_supervisor_rs);
    assert_runtime_policy_exports(runtime_policy_contracts_rs);
    assert_runtime_mode_handler_exports(runtime_mode_handlers_rs);
}

fn assert_runtime_orchestration_no_inline_helpers(runtime_orchestration_rs: &str) {
    assert_not_contains_all(
        runtime_orchestration_rs,
        &[
            ("fn execute_daemon_runtime(", "runtime_orchestration.rs should not keep inline daemon runtime executor"),
            ("fn run_full_supervisor_http_probe(", "runtime_orchestration.rs should not keep inline full supervisor probe helper"),
            ("fn start_full_supervisor_service_api_lane(", "runtime_orchestration.rs should not keep inline full supervisor service-api lane start helper"),
            ("fn finish_full_supervisor_observability_lane(", "runtime_orchestration.rs should not keep inline full supervisor observability lane finish helper"),
            ("fn production_transport_profile_remediation(", "runtime_orchestration.rs should not keep inline runtime transport profile remediation helper"),
            ("fn classify_full_supervisor_stop_contract_violation(", "runtime_orchestration.rs should not keep inline full-supervisor stop contract classifier"),
            ("fn classify_shutdown_checkpoint_reconciliation_violation(", "runtime_orchestration.rs should not keep inline shutdown checkpoint reconciliation classifier"),
            ("fn classify_kolme_live_signer_fallback_secret_policy_violation(", "runtime_orchestration.rs should not keep inline Kolme signer fallback-secret policy classifier"),
        ],
    );
}

fn assert_runtime_orchestration_no_inline_branches(runtime_orchestration_rs: &str) {
    assert_not_contains_all(
        runtime_orchestration_rs,
        &[
            (
                "RuntimeModeKind::Full => {",
                "runtime_orchestration.rs should not keep inline full runtime branch body",
            ),
            (
                "RuntimeModeKind::KolmeLive => {",
                "runtime_orchestration.rs should not keep inline kolme-live runtime branch body",
            ),
            (
                "node.runtime.full.bootstrap.start",
                "runtime_orchestration.rs should not keep inline full runtime bootstrap markers",
            ),
        ],
    );
}

fn assert_daemon_phase_exports(daemon_phase_rs: &str) {
    assert!(
        daemon_phase_rs.contains("pub(super) fn execute_daemon_runtime("),
        "daemon phase module should own daemon runtime execution"
    );
    assert!(
        daemon_phase_rs.contains("pub(super) fn daemon_shutdown_drain_status("),
        "daemon phase module should own shutdown drain status derivation"
    );
}

fn assert_full_supervisor_exports(full_supervisor_rs: &str) {
    assert!(
        full_supervisor_rs.contains("pub(super) fn execute_full_supervisor_daemon_runtime("),
        "full_supervisor module should own full supervisor daemon runtime execution helper"
    );
    assert!(
        full_supervisor_rs.contains("pub(super) fn run_full_supervisor_http_probe("),
        "full_supervisor module should own full supervisor HTTP probe helper"
    );
}

fn assert_runtime_mode_handler_exports(runtime_mode_handlers_rs: &str) {
    assert!(
        runtime_mode_handlers_rs.contains("pub(super) fn execute_full_runtime_mode("),
        "runtime_mode_handlers module should own full runtime-mode execution handler"
    );
    assert!(
        runtime_mode_handlers_rs.contains("pub(super) fn execute_kolme_live_runtime_mode("),
        "runtime_mode_handlers module should own kolme-live runtime-mode execution handler"
    );
}

fn assert_runtime_policy_exports(runtime_policy_contracts_rs: &str) {
    assert_runtime_policy_shutdown_exports(runtime_policy_contracts_rs);
    assert_runtime_policy_validation_exports(runtime_policy_contracts_rs);
}

fn assert_runtime_policy_shutdown_exports(runtime_policy_contracts_rs: &str) {
    assert!(
        runtime_policy_contracts_rs.contains("pub(super) fn should_use_os_signal_shutdown(")
            || runtime_policy_contracts_rs.contains("pub(crate) fn should_use_os_signal_shutdown("),
        "runtime_policy_contracts module should own os-signal shutdown selection policy helper"
    );
    assert!(
        runtime_policy_contracts_rs
            .contains("pub(super) fn classify_production_transport_profile_violation(")
            || runtime_policy_contracts_rs
                .contains("pub(crate) fn classify_production_transport_profile_violation("),
        "runtime_policy_contracts module should own transport profile violation classifier"
    );
}

fn assert_runtime_policy_validation_exports(runtime_policy_contracts_rs: &str) {
    assert!(
        runtime_policy_contracts_rs
            .contains("pub(super) fn validate_full_supervisor_stop_contract(")
            || runtime_policy_contracts_rs
                .contains("pub(crate) fn validate_full_supervisor_stop_contract("),
        "runtime_policy_contracts module should own full-supervisor stop contract validation"
    );
    assert!(
        runtime_policy_contracts_rs
            .contains("pub(super) fn validate_shutdown_checkpoint_reconciliation(")
            || runtime_policy_contracts_rs
                .contains("pub(crate) fn validate_shutdown_checkpoint_reconciliation("),
        "runtime_policy_contracts module should own shutdown checkpoint reconciliation validation"
    );
    assert!(
        runtime_policy_contracts_rs
            .contains("pub(super) fn enforce_kolme_live_signer_key_source_policy(")
            || runtime_policy_contracts_rs
                .contains("pub(crate) fn enforce_kolme_live_signer_key_source_policy("),
        "runtime_policy_contracts module should own Kolme signer key-source policy enforcement"
    );
}
