use crate::commands::{command_output, connect_handle, OutputValue};
use crate::{CommandOutput, ParsedCliArgs};
use kamn_agent_lib::AgentLibError;

/// Executes the health command.
pub fn execute(args: &ParsedCliArgs) -> Result<CommandOutput, AgentLibError> {
    let handle = connect_handle(args)?;
    let health = handle.health()?;
    Ok(command_output(vec![
        ("status", OutputValue::String(health.status)),
        ("runtime_mode", OutputValue::String(health.runtime_mode)),
        ("role", OutputValue::String(health.role)),
        (
            "observability_source",
            OutputValue::String(health.observability_source),
        ),
        (
            "observability_health",
            OutputValue::String(health.observability_health),
        ),
    ]))
}
