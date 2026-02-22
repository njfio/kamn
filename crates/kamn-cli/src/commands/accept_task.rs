use crate::commands::{command_output, connect_handle, required_arg, OutputValue};
use crate::{CommandOutput, ParsedCliArgs};
use kamn_agent_lib::AgentLibError;

/// Executes the accept_task command.
pub fn execute(args: &ParsedCliArgs) -> Result<CommandOutput, AgentLibError> {
    let task_id = required_arg(args, 0, "task_id")?;
    let handle = connect_handle(args)?;
    let receipt = handle.accept_task(task_id)?;
    Ok(command_output(vec![
        ("task_id", OutputValue::String(receipt.task_id)),
        ("state", OutputValue::String(receipt.state)),
    ]))
}
