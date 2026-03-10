use super::*;

struct LivePostgresValidationGuards {
    _lock: std::sync::MutexGuard<'static, ()>,
    _level_guard: EnvVarGuard,
    _format_guard: EnvVarGuard,
}

pub(super) fn sample_topology_fingerprint(
    topology_id: &str,
    host_a: &str,
    host_b: &str,
    lane_id: &str,
) -> String {
    format_parallel_lane_topology_fingerprint(
        topology_id,
        host_a,
        host_b,
        vec![sample_lane_fingerprint(lane_id)],
    )
}

fn sample_lane_fingerprint(lane_id: &str) -> String {
    format_parallel_lane_fingerprint(
        lane_id,
        &sample_phase6_projection(),
        &sample_phase6_projection(),
    )
}

fn sample_phase6_projection() -> LivePostgresPhase6Projection {
    LivePostgresPhase6Projection {
        reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE.to_owned(),
        reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
    }
}

pub(super) fn with_live_postgres_validation_database_url(run: impl FnOnce(String)) {
    let _guards = live_postgres_validation_guards();
    let (gate_reason_code, maybe_database_url) = resolve_live_postgres_gate_decision();
    let Some(database_url) = maybe_database_url else {
        assert_eq!(gate_reason_code, LIVE_POSTGRES_ENV_UNSET_REASON_CODE);
        return;
    };
    assert_eq!(
        gate_reason_code,
        LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE
    );
    run(database_url);
}

fn live_postgres_validation_guards() -> LivePostgresValidationGuards {
    let lock = log_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    LivePostgresValidationGuards {
        _lock: lock,
        _level_guard: level_guard,
        _format_guard: format_guard,
    }
}

pub(super) fn apply_live_postgres_validation_migrations(database_url: String) {
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

pub(super) fn permuted_topology_profiles(
    permutation: &str,
) -> Vec<LivePostgresParallelLaneTopologyProfile> {
    permute_parallel_lane_topology_profiles(
        project_live_postgres_parallel_lane_topology_profiles(),
        permutation,
    )
}

pub(super) fn assert_rows_stable_across_permutations<F>(
    baseline_rows: &[String],
    collect_rows: F,
    context: &str,
) where
    F: Fn(&str) -> Vec<String>,
{
    for permutation in ["reverse", "rotate_left_1"] {
        let permuted_rows = collect_rows(permutation);
        assert_eq!(
            baseline_rows, permuted_rows,
            "{context} under permutation {permutation}"
        );
    }
}
