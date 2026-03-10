use super::super::constants::*;
use super::super::gate_support::*;
use super::super::models::*;
use crate::main_tests::extract_json_string_field;
use crate::{execute, render_bootstrap_report, OutputMode};
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
    let parsed = parse_args_with_clean_daemon_env(args).expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    LivePostgresPhase6Projection {
        reason_code: extract_json_string_field(
            rendered.as_str(),
            "daemon_phase6_runtime_reason_code",
        )
        .expect("daemon report should expose phase6 reason code"),
        reason_taxonomy_version: extract_json_string_field(
            rendered.as_str(),
            "daemon_phase6_runtime_reason_taxonomy_version",
        )
        .expect("daemon report should expose phase6 reason taxonomy version"),
    }
}
