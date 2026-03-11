use std::env;

use crate::{ExecutionMode, RunCommandConfig};

use super::super::ExternalRuntimeComponentBinaries;
use super::filesystem::ensure_binary_path_is_executable;

const EXTERNAL_KAMN_PROCESSOR_BINARY_ENV: &str = "KAMN_E2E_EXTERNAL_KAMN_PROCESSOR_BINARY";
const EXTERNAL_KAMN_LISTENER_BINARY_ENV: &str = "KAMN_E2E_EXTERNAL_KAMN_LISTENER_BINARY";
const EXTERNAL_KAMN_APPROVER_BINARY_ENV: &str = "KAMN_E2E_EXTERNAL_KAMN_APPROVER_BINARY";

pub(crate) fn ensure_external_execution_preflight(
    config: &RunCommandConfig,
    mode: ExecutionMode,
) -> Result<(), String> {
    if config.kolme_binary.trim().is_empty() {
        return Err("external execution preflight failed: kolme binary path is empty".to_owned());
    }
    ensure_binary_path_is_executable(config.kolme_binary.as_str(), "kolme")?;
    ensure_agent_binary_is_executable(config, mode)?;
    let component_binaries = resolve_external_runtime_component_binaries_from_env()?;
    ensure_runtime_component_binaries(&component_binaries)
}

pub(crate) fn resolve_external_runtime_component_binaries_from_env(
) -> Result<ExternalRuntimeComponentBinaries, String> {
    Ok(ExternalRuntimeComponentBinaries {
        kamn_processor_binary: resolve_required_external_runtime_binary_env(
            EXTERNAL_KAMN_PROCESSOR_BINARY_ENV,
            "kamn_processor",
        )?,
        kamn_listener_binary: resolve_required_external_runtime_binary_env(
            EXTERNAL_KAMN_LISTENER_BINARY_ENV,
            "kamn_listener",
        )?,
        kamn_approver_binary: resolve_required_external_runtime_binary_env(
            EXTERNAL_KAMN_APPROVER_BINARY_ENV,
            "kamn_approver",
        )?,
    })
}

fn ensure_agent_binary_is_executable(
    config: &RunCommandConfig,
    mode: ExecutionMode,
) -> Result<(), String> {
    if matches!(mode, ExecutionMode::McpTau | ExecutionMode::McpAny) {
        let agent_binary = config.agent_binary.as_deref().ok_or_else(|| {
            "external execution preflight failed: agent binary missing for MCP modes".to_owned()
        })?;
        if agent_binary.trim().is_empty() {
            return Err(
                "external execution preflight failed: agent binary path is empty".to_owned(),
            );
        }
        ensure_binary_path_is_executable(agent_binary, "agent")?;
    }
    Ok(())
}

fn ensure_runtime_component_binaries(
    component_binaries: &ExternalRuntimeComponentBinaries,
) -> Result<(), String> {
    ensure_binary_path_is_executable(
        component_binaries.kamn_processor_binary.as_str(),
        "kamn_processor",
    )?;
    ensure_binary_path_is_executable(
        component_binaries.kamn_listener_binary.as_str(),
        "kamn_listener",
    )?;
    ensure_binary_path_is_executable(
        component_binaries.kamn_approver_binary.as_str(),
        "kamn_approver",
    )?;
    Ok(())
}

fn resolve_required_external_runtime_binary_env(
    env_name: &str,
    label: &str,
) -> Result<String, String> {
    let value = env::var(env_name).map_err(|_| missing_runtime_binary_env(env_name, label))?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!(
            "external execution preflight failed: runtime component binary env is empty: {env_name} ({label})"
        ));
    }
    Ok(trimmed.to_owned())
}

fn missing_runtime_binary_env(env_name: &str, label: &str) -> String {
    format!(
        "external execution preflight failed: missing required runtime component binary env: {env_name} ({label})"
    )
}
