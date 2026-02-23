use crate::commands::{command_output, connect_handle, required_arg, OutputValue};
use crate::{CommandOutput, ParsedCliArgs};
use kamn_agent_lib::AgentLibError;

/// Executes the expire_content command.
pub fn execute(args: &ParsedCliArgs) -> Result<CommandOutput, AgentLibError> {
    let content_id = required_arg(args, 0, "expire_content_id")?;
    let handle = connect_handle(args)?;
    let status = handle.expire_content(content_id)?;
    Ok(command_output(vec![
        ("content_id", OutputValue::String(status.content_id)),
        (
            "lifecycle_state",
            OutputValue::String(status.lifecycle_state),
        ),
        (
            "redaction_status",
            OutputValue::String(status.redaction_status),
        ),
    ]))
}
