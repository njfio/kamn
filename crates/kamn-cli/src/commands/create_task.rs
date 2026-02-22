use crate::commands::{command_output, connect_handle, required_arg, OutputValue};
use crate::{CommandOutput, ParsedCliArgs};
use kamn_agent_lib::AgentLibError;

/// Executes the create_task command.
pub fn execute(args: &ParsedCliArgs) -> Result<CommandOutput, AgentLibError> {
    let payload = required_arg(args, 0, "create_task_payload")?;
    let handle = connect_handle(args)?;
    let receipt = handle.create_task(payload)?;
    Ok(command_output(vec![
        ("task_id", OutputValue::String(receipt.task_id)),
        ("state", OutputValue::String(receipt.state)),
    ]))
}
