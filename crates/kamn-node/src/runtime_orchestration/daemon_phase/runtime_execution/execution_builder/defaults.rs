pub(super) struct EmptyObservabilityFields {
    pub(super) latency_p50_ms: u64,
    pub(super) latency_p99_ms: u64,
    pub(super) throughput_tps: u64,
    pub(super) error_rate_bps: u64,
    pub(super) availability_bps: u64,
    pub(super) health: String,
    pub(super) alert_count: usize,
    pub(super) reason_code: String,
    pub(super) transport_checkpoint_failures: u64,
    pub(super) signer_checkpoint_failures: u64,
    pub(super) commit_checkpoint_failures: u64,
}

pub(super) struct EmptyProjectionFields {
    pub(super) phase6_runtime_reason_taxonomy_version: String,
    pub(super) phase6_runtime_reason_codes_csv: String,
    pub(super) phase6_runtime_reason_code: String,
    pub(super) phase6_runtime_total_cycles: u64,
    pub(super) phase6_runtime_executed_cycles: u64,
    pub(super) phase6_runtime_deferred_cycles: u64,
    pub(super) phase6_runtime_fail_closed_cycles: u64,
    pub(super) convergence_reason_taxonomy_version: String,
    pub(super) convergence_reason_codes_csv: String,
    pub(super) convergence_decision: String,
    pub(super) convergence_reason_code: String,
    pub(super) convergence_schema_gate_passed: bool,
    pub(super) convergence_error_path_gate_passed: bool,
    pub(super) convergence_concurrency_gate_passed: bool,
    pub(super) convergence_performance_budget_gate_passed: bool,
    pub(super) convergence_cost_budget_gate_passed: bool,
    pub(super) live_postgres_multi_host_execution_bundle_schema_version: String,
    pub(super) live_postgres_multi_host_execution_bundle_selector_prefix: String,
    pub(super) live_postgres_multi_host_execution_bundle_row_count: usize,
    pub(super) live_postgres_multi_host_execution_bundle_selector_rows_fingerprint: String,
}

pub(super) fn empty_observability_fields() -> EmptyObservabilityFields {
    EmptyObservabilityFields {
        latency_p50_ms: 0,
        latency_p99_ms: 0,
        throughput_tps: 0,
        error_rate_bps: 0,
        availability_bps: 0,
        health: String::new(),
        alert_count: 0,
        reason_code: String::new(),
        transport_checkpoint_failures: 0,
        signer_checkpoint_failures: 0,
        commit_checkpoint_failures: 0,
    }
}

pub(super) fn empty_projection_fields() -> EmptyProjectionFields {
    EmptyProjectionFields {
        phase6_runtime_reason_taxonomy_version: String::new(),
        phase6_runtime_reason_codes_csv: String::new(),
        phase6_runtime_reason_code: String::new(),
        phase6_runtime_total_cycles: 0,
        phase6_runtime_executed_cycles: 0,
        phase6_runtime_deferred_cycles: 0,
        phase6_runtime_fail_closed_cycles: 0,
        convergence_reason_taxonomy_version: String::new(),
        convergence_reason_codes_csv: String::new(),
        convergence_decision: String::new(),
        convergence_reason_code: String::new(),
        convergence_schema_gate_passed: false,
        convergence_error_path_gate_passed: false,
        convergence_concurrency_gate_passed: false,
        convergence_performance_budget_gate_passed: false,
        convergence_cost_budget_gate_passed: false,
        live_postgres_multi_host_execution_bundle_schema_version: String::new(),
        live_postgres_multi_host_execution_bundle_selector_prefix: String::new(),
        live_postgres_multi_host_execution_bundle_row_count: 0,
        live_postgres_multi_host_execution_bundle_selector_rows_fingerprint: String::new(),
    }
}
