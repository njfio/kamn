use crate::commands::connect_handle;
use crate::ParsedCliArgs;
use kamn_agent_lib::AgentLibError;

/// Executes the health command.
pub fn execute(args: &ParsedCliArgs) -> Result<String, AgentLibError> {
    let handle = connect_handle(args)?;
    let health = handle.health()?;
    Ok(format!(
        "status={} runtime_mode={} role={} observability_source={} observability_health={}",
        health.status,
        health.runtime_mode,
        health.role,
        health.observability_source,
        health.observability_health
    ))
}
