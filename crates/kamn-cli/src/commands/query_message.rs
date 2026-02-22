use crate::commands::{command_output, connect_handle, required_arg, OutputValue};
use crate::{CommandOutput, ParsedCliArgs};
use kamn_agent_lib::AgentLibError;

/// Executes the query_message command.
pub fn execute(args: &ParsedCliArgs) -> Result<CommandOutput, AgentLibError> {
    let message_id = required_arg(args, 0, "query_message_id")?;
    let handle = connect_handle(args)?;
    let status = handle.query_message(message_id)?;
    Ok(command_output(vec![
        ("message_id", OutputValue::String(status.message_id)),
        ("status", OutputValue::String(status.status)),
    ]))
}
