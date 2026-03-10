pub(crate) const fn convergence_reason_taxonomy_version() -> &'static str {
    "kamn.runtime.daemon.convergence.reason-taxonomy.v1"
}

pub(crate) const fn convergence_reason_codes_csv() -> &'static str {
    "convergence_promotion_gate_go,convergence_schema_drift_detected,convergence_error_path_drift_detected,convergence_concurrency_drift_detected,convergence_performance_budget_exceeded,convergence_cost_budget_exceeded"
}

const DAEMON_CONVERGENCE_DECISION_GO: &str = "go";
const DAEMON_CONVERGENCE_DECISION_NO_GO: &str = "no_go";
const DAEMON_CONVERGENCE_REASON_GO: &str = "convergence_promotion_gate_go";
const DAEMON_CONVERGENCE_REASON_SCHEMA_DRIFT: &str = "convergence_schema_drift_detected";
const DAEMON_CONVERGENCE_REASON_ERROR_PATH_DRIFT: &str = "convergence_error_path_drift_detected";
const DAEMON_CONVERGENCE_REASON_CONCURRENCY_DRIFT: &str = "convergence_concurrency_drift_detected";
const DAEMON_CONVERGENCE_REASON_PERFORMANCE_BUDGET: &str =
    "convergence_performance_budget_exceeded";
const DAEMON_CONVERGENCE_REASON_COST_BUDGET: &str = "convergence_cost_budget_exceeded";

pub(crate) struct DaemonConvergenceInput {
    pub(crate) schema_gate_passed: bool,
    pub(crate) error_path_gate_passed: bool,
    pub(crate) concurrency_gate_passed: bool,
    pub(crate) performance_budget_gate_passed: bool,
    pub(crate) cost_budget_gate_passed: bool,
}

pub(crate) struct DaemonConvergenceProjection {
    pub(crate) decision: &'static str,
    pub(crate) reason_code: &'static str,
    pub(crate) schema_gate_passed: bool,
    pub(crate) error_path_gate_passed: bool,
    pub(crate) concurrency_gate_passed: bool,
    pub(crate) performance_budget_gate_passed: bool,
    pub(crate) cost_budget_gate_passed: bool,
}

pub(crate) fn execute_daemon_convergence_projection(
    input: DaemonConvergenceInput,
) -> DaemonConvergenceProjection {
    let (decision, reason_code) = select_convergence_decision(&input);
    DaemonConvergenceProjection {
        decision,
        reason_code,
        schema_gate_passed: input.schema_gate_passed,
        error_path_gate_passed: input.error_path_gate_passed,
        concurrency_gate_passed: input.concurrency_gate_passed,
        performance_budget_gate_passed: input.performance_budget_gate_passed,
        cost_budget_gate_passed: input.cost_budget_gate_passed,
    }
}

fn select_convergence_decision(input: &DaemonConvergenceInput) -> (&'static str, &'static str) {
    if let Some(reason_code) = first_failed_reason(input) {
        return (DAEMON_CONVERGENCE_DECISION_NO_GO, reason_code);
    }
    (DAEMON_CONVERGENCE_DECISION_GO, DAEMON_CONVERGENCE_REASON_GO)
}

fn first_failed_reason(input: &DaemonConvergenceInput) -> Option<&'static str> {
    failed_reason(
        input.schema_gate_passed,
        DAEMON_CONVERGENCE_REASON_SCHEMA_DRIFT,
    )
    .or_else(|| {
        failed_reason(
            input.error_path_gate_passed,
            DAEMON_CONVERGENCE_REASON_ERROR_PATH_DRIFT,
        )
    })
    .or_else(|| {
        failed_reason(
            input.concurrency_gate_passed,
            DAEMON_CONVERGENCE_REASON_CONCURRENCY_DRIFT,
        )
    })
    .or_else(|| {
        failed_reason(
            input.performance_budget_gate_passed,
            DAEMON_CONVERGENCE_REASON_PERFORMANCE_BUDGET,
        )
    })
    .or_else(|| {
        failed_reason(
            input.cost_budget_gate_passed,
            DAEMON_CONVERGENCE_REASON_COST_BUDGET,
        )
    })
}

fn failed_reason(passed: bool, reason_code: &'static str) -> Option<&'static str> {
    if passed {
        return None;
    }
    Some(reason_code)
}
