use std::process::{Command, Stdio};
use std::time::Duration;

use crate::{ExecutionMode, PhaseResultStatus, RunCommandConfig};

use super::super::{
    aggregate_status, is_mcp_mode, ExternalRuntimeComponentProbe, ExternalRuntimeProbeSummary,
};
use super::preflight::resolve_external_runtime_component_binaries_from_env;

pub(crate) const ETXTBSY_ERRNO: i32 = 26;
pub(crate) const TEXT_FILE_BUSY_RETRY_LIMIT: usize = 3;

pub(crate) fn should_retry_text_file_busy(error: &std::io::Error, retry_attempt: usize) -> bool {
    error.raw_os_error() == Some(ETXTBSY_ERRNO) && retry_attempt < TEXT_FILE_BUSY_RETRY_LIMIT
}

pub(crate) fn probe_command_args_for_label(label: &str) -> &'static [&'static str] {
    match label {
        "kamn_processor" => &["--role", "processor"],
        "kamn_listener" => &["--role", "listener"],
        "kamn_approver" => &["--role", "approver"],
        _ => &["--help"],
    }
}

pub(crate) fn probe_binary_invocation_with_status_runner<F>(
    label: &str,
    mut status_runner: F,
) -> (PhaseResultStatus, String)
where
    F: FnMut() -> std::io::Result<std::process::ExitStatus>,
{
    for retry_attempt in 0..=TEXT_FILE_BUSY_RETRY_LIMIT {
        match evaluate_probe_attempt(label, retry_attempt, status_runner()) {
            ProbeAttempt::Retry => std::thread::sleep(Duration::from_millis(10)),
            ProbeAttempt::Resolved(result) => return result,
        }
    }
    fail_probe(label, "retry budget exhausted".to_owned())
}

pub(crate) fn probe_external_runtime(
    config: &RunCommandConfig,
    mode: ExecutionMode,
) -> ExternalRuntimeProbeSummary {
    let component_binaries = match resolve_external_runtime_component_binaries_from_env() {
        Ok(binaries) => binaries,
        Err(error) => return unresolved_runtime_summary(error),
    };
    let runtime = runtime_component_probes(config, &component_binaries);
    if is_mcp_mode(mode) && config.agent_binary.as_deref().is_none() {
        return missing_agent_summary(
            &runtime.kolme,
            &runtime.kamn_processor,
            &runtime.kamn_listener,
            &runtime.kamn_approver,
        );
    }
    let agent = probe_agent(config, mode);
    build_runtime_probe_summary(runtime, agent)
}

fn probe_component(binary: &str, label: &str) -> ExternalRuntimeComponentProbe {
    let args = probe_command_args_for_label(label);
    let (status, detail) = probe_binary_invocation_with_status_runner(label, || {
        let mut command = Command::new(binary);
        command
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        command.status()
    });
    ExternalRuntimeComponentProbe { status, detail }
}

fn unresolved_runtime_summary(error: String) -> ExternalRuntimeProbeSummary {
    ExternalRuntimeProbeSummary {
        status: PhaseResultStatus::Fail,
        detail: error.clone(),
        kolme: ExternalRuntimeComponentProbe {
            status: PhaseResultStatus::Skip,
            detail: "kolme probe skipped due unresolved runtime component binary envs".to_owned(),
        },
        kamn_processor: failing_component(error.clone()),
        kamn_listener: failing_component(error.clone()),
        kamn_approver: failing_component(error.clone()),
        agent: ExternalRuntimeComponentProbe {
            status: PhaseResultStatus::Skip,
            detail: "agent probe skipped due unresolved runtime component binary envs".to_owned(),
        },
    }
}

fn probe_agent(config: &RunCommandConfig, mode: ExecutionMode) -> ExternalRuntimeComponentProbe {
    if !is_mcp_mode(mode) {
        return ExternalRuntimeComponentProbe {
            status: PhaseResultStatus::Skip,
            detail: "agent probe skipped (mode does not require agent binary)".to_owned(),
        };
    }
    let Some(agent_binary) = config.agent_binary.as_deref() else {
        return missing_agent_runtime_probe();
    };
    probe_component(agent_binary, "agent")
}

fn missing_agent_runtime_probe() -> ExternalRuntimeComponentProbe {
    ExternalRuntimeComponentProbe {
        status: PhaseResultStatus::Fail,
        detail: "mcp agent runtime probe missing after validation".to_owned(),
    }
}

fn missing_agent_summary(
    kolme: &ExternalRuntimeComponentProbe,
    kamn_processor: &ExternalRuntimeComponentProbe,
    kamn_listener: &ExternalRuntimeComponentProbe,
    kamn_approver: &ExternalRuntimeComponentProbe,
) -> ExternalRuntimeProbeSummary {
    ExternalRuntimeProbeSummary {
        status: PhaseResultStatus::Fail,
        detail: "agent probe failed (missing binary path)".to_owned(),
        kolme: kolme.clone(),
        kamn_processor: kamn_processor.clone(),
        kamn_listener: kamn_listener.clone(),
        kamn_approver: kamn_approver.clone(),
        agent: failing_component("agent probe failed (missing binary path)".to_owned()),
    }
}

struct RuntimeComponentProbes {
    kolme: ExternalRuntimeComponentProbe,
    kamn_processor: ExternalRuntimeComponentProbe,
    kamn_listener: ExternalRuntimeComponentProbe,
    kamn_approver: ExternalRuntimeComponentProbe,
}

fn build_runtime_probe_summary(
    runtime: RuntimeComponentProbes,
    agent: ExternalRuntimeComponentProbe,
) -> ExternalRuntimeProbeSummary {
    let status = aggregate_status(&component_statuses(&runtime, &agent));
    let detail = runtime_probe_detail(&runtime, &agent);
    ExternalRuntimeProbeSummary {
        status,
        detail,
        kolme: runtime.kolme,
        kamn_processor: runtime.kamn_processor,
        kamn_listener: runtime.kamn_listener,
        kamn_approver: runtime.kamn_approver,
        agent,
    }
}

fn failing_component(detail: String) -> ExternalRuntimeComponentProbe {
    ExternalRuntimeComponentProbe {
        status: PhaseResultStatus::Fail,
        detail,
    }
}

enum ProbeAttempt {
    Retry,
    Resolved((PhaseResultStatus, String)),
}

fn evaluate_probe_attempt(
    label: &str,
    retry_attempt: usize,
    result: std::io::Result<std::process::ExitStatus>,
) -> ProbeAttempt {
    match result {
        Ok(status) => ProbeAttempt::Resolved(status_result(label, status)),
        Err(error) if should_retry_text_file_busy(&error, retry_attempt) => ProbeAttempt::Retry,
        Err(error) => ProbeAttempt::Resolved(fail_probe(label, error.to_string())),
    }
}

fn status_result(label: &str, status: std::process::ExitStatus) -> (PhaseResultStatus, String) {
    if status.success() {
        return (PhaseResultStatus::Pass, format!("{label} probe passed"));
    }
    fail_probe(label, exit_status_detail(status))
}

fn fail_probe(label: &str, detail: String) -> (PhaseResultStatus, String) {
    (
        PhaseResultStatus::Fail,
        format!("{label} probe failed ({detail})"),
    )
}

fn exit_status_detail(status: std::process::ExitStatus) -> String {
    status
        .code()
        .map(|code| format!("exit_status={code}"))
        .unwrap_or_else(|| "exit_status=signal".to_owned())
}

fn runtime_component_probes(
    config: &RunCommandConfig,
    component_binaries: &super::super::ExternalRuntimeComponentBinaries,
) -> RuntimeComponentProbes {
    RuntimeComponentProbes {
        kolme: probe_component(config.kolme_binary.as_str(), "kolme"),
        kamn_processor: probe_component(
            component_binaries.kamn_processor_binary.as_str(),
            "kamn_processor",
        ),
        kamn_listener: probe_component(
            component_binaries.kamn_listener_binary.as_str(),
            "kamn_listener",
        ),
        kamn_approver: probe_component(
            component_binaries.kamn_approver_binary.as_str(),
            "kamn_approver",
        ),
    }
}

fn component_statuses(
    runtime: &RuntimeComponentProbes,
    agent: &ExternalRuntimeComponentProbe,
) -> [PhaseResultStatus; 5] {
    [
        runtime.kolme.status,
        runtime.kamn_processor.status,
        runtime.kamn_listener.status,
        runtime.kamn_approver.status,
        agent.status,
    ]
}

fn runtime_probe_detail(
    runtime: &RuntimeComponentProbes,
    agent: &ExternalRuntimeComponentProbe,
) -> String {
    format!(
        "{}; {}; {}; {}; {}",
        runtime.kolme.detail,
        runtime.kamn_processor.detail,
        runtime.kamn_listener.detail,
        runtime.kamn_approver.detail,
        agent.detail
    )
}
