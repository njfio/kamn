use crate::commands::{command_output, connect_handle, required_arg, OutputValue};
use crate::{CommandOutput, ParsedCliArgs};
use kamn_agent_lib::AgentLibError;

/// Executes the send_message command.
pub fn execute(args: &ParsedCliArgs) -> Result<CommandOutput, AgentLibError> {
    let payload = required_arg(args, 0, "send_message_payload")?;
    let handle = connect_handle(args)?;
    let receipt = handle.send_message(payload)?;
    Ok(command_output(vec![
        ("message_id", OutputValue::String(receipt.message_id)),
        ("status", OutputValue::String(receipt.status)),
        ("runtime_mode", OutputValue::String(receipt.runtime_mode)),
    ]))
}
