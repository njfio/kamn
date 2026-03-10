use std::fs;
use std::path::PathBuf;

const DAEMON_PHASE_ROOT_SOURCE: &str = include_str!("../src/runtime_orchestration/daemon_phase.rs");
const RUNTIME_EXECUTION_FILE: &str = "src/runtime_orchestration/daemon_phase/runtime_execution.rs";
const PROJECTIONS_FILE: &str = "src/runtime_orchestration/daemon_phase/projections.rs";
const LIVE_POSTGRES_BUNDLE_FILE: &str =
    "src/runtime_orchestration/daemon_phase/live_postgres_bundle.rs";
const SERVICE_API_RELAY_P2P_FILE: &str =
    "src/runtime_orchestration/daemon_phase/service_api_relay_p2p.rs";
const SERVICE_API_RELAY_TICK_LOOP_FILE: &str =
    "src/runtime_orchestration/daemon_phase/service_api_relay_tick_loop.rs";
const TEST_ROOT_FILE: &str = "src/runtime_orchestration/daemon_phase/tests.rs";
const ROOT_MAX_LINES: usize = 200;
const EXTRACTED_MAX_LINES: usize = 200;

#[test]
fn regression_daemon_phase_root_declares_extracted_modules() {
    for marker in [
        "mod runtime_execution;",
        "mod projections;",
        "mod live_postgres_bundle;",
        "mod service_api_relay_p2p;",
        "mod service_api_relay_tick_loop;",
        "#[cfg(test)]\nmod tests;",
    ] {
        assert!(
            DAEMON_PHASE_ROOT_SOURCE.contains(marker),
            "daemon_phase.rs must declare extracted module marker: {marker}"
        );
    }
}

#[test]
fn regression_daemon_phase_root_removes_residual_projection_and_relay_definitions() {
    for marker in [
        "fn execute_daemon_phase6_runtime_projection(",
        "fn execute_daemon_convergence_projection(",
        "fn project_live_postgres_multi_host_execution_bundle_selector_rows(",
        "fn validate_live_postgres_selector_bundle(",
        "fn normalize_daemon_service_api_relay_p2p_config(",
        "fn resolve_daemon_service_api_relay_p2p_context(",
        "fn resolve_daemon_service_api_relay_p2p_context_from_json(",
        "fn forward_service_api_relay_entry_via_p2p(",
        "fn drain_daemon_service_api_relay_p2p_inbox(",
        "fn execute_daemon_service_api_relay_tick_loop(",
        "fn daemon_tick_remaining_sleep_duration(",
        "mod tests {",
    ] {
        assert!(
            !DAEMON_PHASE_ROOT_SOURCE.contains(marker),
            "daemon_phase.rs must not keep residual extracted marker: {marker}"
        );
    }
}

#[test]
fn regression_daemon_phase_extracted_module_files_exist() {
    for relative_path in [
        RUNTIME_EXECUTION_FILE,
        PROJECTIONS_FILE,
        LIVE_POSTGRES_BUNDLE_FILE,
        SERVICE_API_RELAY_P2P_FILE,
        SERVICE_API_RELAY_TICK_LOOP_FILE,
        TEST_ROOT_FILE,
    ] {
        let full_path = manifest_dir().join(relative_path);
        assert!(
            full_path.exists(),
            "expected daemon_phase extracted module missing: {}",
            full_path.display()
        );
    }
}

#[test]
fn regression_daemon_phase_root_respects_file_budget() {
    let line_count = DAEMON_PHASE_ROOT_SOURCE.lines().count();
    assert!(
        line_count <= ROOT_MAX_LINES,
        "daemon_phase.rs should stay within the root file budget: {line_count} > {ROOT_MAX_LINES}"
    );
}

#[test]
fn regression_daemon_phase_extracted_files_stay_within_line_budget() {
    let offenders = [
        RUNTIME_EXECUTION_FILE,
        PROJECTIONS_FILE,
        LIVE_POSTGRES_BUNDLE_FILE,
        SERVICE_API_RELAY_P2P_FILE,
        SERVICE_API_RELAY_TICK_LOOP_FILE,
        TEST_ROOT_FILE,
    ]
    .into_iter()
    .filter_map(|relative_path| {
        let full_path = manifest_dir().join(relative_path);
        let line_count = fs::read_to_string(&full_path).ok()?.lines().count();
        (line_count > EXTRACTED_MAX_LINES)
            .then(|| format!("{} ({line_count})", full_path.display()))
    })
    .collect::<Vec<String>>();

    assert!(
        offenders.is_empty(),
        "extracted daemon_phase files exceed {EXTRACTED_MAX_LINES} LOC: {}",
        offenders.join(", ")
    );
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
