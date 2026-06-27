use super::super::constants::*;
use super::super::models::*;
use crate::main_tests::{extract_json_string_field, lock_signer_env_guard, EnvVarGuard};
use crate::{daemon_test_env_lock, execute, parse_args, render_bootstrap_report, OutputMode};

pub(crate) fn project_live_postgres_matrix_rows() -> Vec<LivePostgresMatrixRow> {
    vec![
        LivePostgresMatrixRow {
            scenario_id: "env_unset",
            gate_reason_code: LIVE_POSTGRES_ENV_UNSET_REASON_CODE,
            daemon_phase6_reason_code: None,
        },
        LivePostgresMatrixRow {
            scenario_id: "env_set_no_shutdown",
            gate_reason_code: LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE,
            daemon_phase6_reason_code: Some(LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE),
        },
        LivePostgresMatrixRow {
            scenario_id: "env_set_shutdown",
            gate_reason_code: LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE,
            daemon_phase6_reason_code: Some(LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE),
        },
    ]
}

pub(crate) fn run_daemon_for_phase6_projection(
    mut args: Vec<String>,
) -> LivePostgresPhase6Projection {
    args.push("--output".to_owned());
    args.push("json".to_owned());
    let _service_api_env_lock = lock_signer_env_guard();
    let _daemon_env_lock = daemon_test_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _env_guards = isolated_daemon_projection_env_guards();
    let parsed = parse_args(args).expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    phase6_projection_from_rendered_report(rendered.as_str())
}

fn phase6_projection_from_rendered_report(rendered: &str) -> LivePostgresPhase6Projection {
    LivePostgresPhase6Projection {
        reason_code: extract_json_string_field(rendered, "daemon_phase6_runtime_reason_code")
            .expect("daemon report should expose phase6 reason code"),
        reason_taxonomy_version: extract_json_string_field(
            rendered,
            "daemon_phase6_runtime_reason_taxonomy_version",
        )
        .expect("daemon report should expose phase6 reason taxonomy version"),
    }
}

fn isolated_daemon_projection_env_guards() -> Vec<EnvVarGuard> {
    let (state_file, relay_spool_file) = unique_daemon_projection_paths();
    [
        ("KAMN_NODE_DAEMON_MAX_TICKS", None),
        ("KAMN_NODE_DAEMON_TICK_INTERVAL_MS", None),
        ("KAMN_SERVICE_API_STATE_FILE", Some(state_file.as_str())),
        (
            "KAMN_SERVICE_API_RELAY_SPOOL_FILE",
            Some(relay_spool_file.as_str()),
        ),
    ]
    .into_iter()
    .map(|(key, value)| EnvVarGuard::set(key, value))
    .collect()
}

fn unique_daemon_projection_paths() -> (String, String) {
    let suffix = format!(
        "{}-{}-{:?}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos(),
        std::thread::current().id()
    );
    let base = std::env::temp_dir();
    (
        base.join(format!("kamn-node-phase6-projection-state-{suffix}.json"))
            .to_string_lossy()
            .to_string(),
        base.join(format!("kamn-node-phase6-projection-spool-{suffix}.ndjson"))
            .to_string_lossy()
            .to_string(),
    )
}
