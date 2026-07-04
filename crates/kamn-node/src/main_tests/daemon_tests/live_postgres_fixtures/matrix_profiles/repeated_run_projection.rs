use super::super::constants::*;
use super::super::gate_support::*;
use super::super::models::*;
use super::super::*;
use super::daemon_args::*;

fn assert_live_postgres_gate_ready() -> Option<String> {
    let (gate_reason_code, maybe_database_url) = resolve_live_postgres_gate_decision();
    if maybe_database_url.is_none() {
        assert_eq!(gate_reason_code, LIVE_POSTGRES_ENV_UNSET_REASON_CODE);
        return None;
    }
    assert_eq!(
        gate_reason_code,
        LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE
    );
    maybe_database_url
}

fn apply_live_postgres_migrations(database_url: String) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime should be constructible for live postgres validation");
    runtime.block_on(async move {
        let adapter = kamn_core::DataLayerPgExecutionAdapter::connect(
            kamn_core::DataLayerPgExecutionAdapterConfig {
                database_url,
                max_connections: 4,
            },
        )
        .await
        .expect("live postgres connection should succeed when test URL is provided");
        adapter
            .apply_migrations()
            .await
            .expect("live postgres migrations should apply for validation slice");
    });
}

fn repeated_run_args(shutdown: Option<ShutdownArgsSpec>) -> Vec<String> {
    daemon_args_for_live_postgres_profile("processor", "5", "25", shutdown)
}

fn run_repeated_projection_pair(
    shutdown: Option<ShutdownArgsSpec>,
) -> (LivePostgresPhase6Projection, LivePostgresPhase6Projection) {
    let args = repeated_run_args(shutdown);
    let first = run_daemon_for_phase6_projection(args.clone());
    let second = run_daemon_for_phase6_projection(args);
    (first, second)
}

pub(crate) fn run_live_postgres_matrix_repeated_run_projections() -> Option<(
    LivePostgresPhase6Projection,
    LivePostgresPhase6Projection,
    LivePostgresPhase6Projection,
    LivePostgresPhase6Projection,
)> {
    let _lock = log_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let database_url = assert_live_postgres_gate_ready()?;
    apply_live_postgres_migrations(database_url);
    let (applied_first, applied_second) = run_repeated_projection_pair(None);
    let (deferred_first, deferred_second) = run_repeated_projection_pair(Some(("3", "2", "4")));
    Some((
        applied_first,
        applied_second,
        deferred_first,
        deferred_second,
    ))
}
