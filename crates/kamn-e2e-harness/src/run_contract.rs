use crate::{ExecutionMode, PhaseResultStatus};

mod evidence_io;
mod external_runtime;
mod orchestration;
#[cfg(test)]
mod tests;

#[cfg(test)]
use external_runtime::{
    probe_binary_invocation_with_status_runner, probe_command_args_for_label,
    should_retry_text_file_busy, ETXTBSY_ERRNO, TEXT_FILE_BUSY_RETRY_LIMIT,
};
pub(crate) use orchestration::aggregate_status;
pub use orchestration::execute_run_contract;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScenarioExecutionResult {
    id: String,
    status: PhaseResultStatus,
    detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExternalRuntimeComponentProbe {
    status: PhaseResultStatus,
    detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExternalRuntimeComponentBinaries {
    kamn_processor_binary: String,
    kamn_listener_binary: String,
    kamn_approver_binary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExternalRuntimeProbeSummary {
    status: PhaseResultStatus,
    detail: String,
    kolme: ExternalRuntimeComponentProbe,
    kamn_processor: ExternalRuntimeComponentProbe,
    kamn_listener: ExternalRuntimeComponentProbe,
    kamn_approver: ExternalRuntimeComponentProbe,
    agent: ExternalRuntimeComponentProbe,
}

fn escape_json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn is_mcp_mode(mode: ExecutionMode) -> bool {
    matches!(mode, ExecutionMode::McpTau | ExecutionMode::McpAny)
}
