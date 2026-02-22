use crate::commands::{command_output, connect_handle, required_arg, OutputValue};
use crate::{CommandOutput, ParsedCliArgs};
use kamn_agent_lib::AgentLibError;

/// Executes the create_channel command.
pub fn execute(args: &ParsedCliArgs) -> Result<CommandOutput, AgentLibError> {
    let payload = required_arg(args, 0, "create_channel_payload")?;
    let handle = connect_handle(args)?;
    let receipt = handle.create_channel(payload)?;
    Ok(command_output(vec![
        ("channel_id", OutputValue::String(receipt.channel_id)),
        ("status", OutputValue::String(receipt.status)),
    ]))
}
