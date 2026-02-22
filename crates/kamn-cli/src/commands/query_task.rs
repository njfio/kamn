use crate::commands::{command_output, connect_handle, required_arg, OutputValue};
use crate::{CommandOutput, ParsedCliArgs};
use kamn_agent_lib::AgentLibError;

/// Executes the query_task command.
pub fn execute(args: &ParsedCliArgs) -> Result<CommandOutput, AgentLibError> {
    let task_id = required_arg(args, 0, "query_task_id")?;
    let handle = connect_handle(args)?;
    let status = handle.query_task(task_id)?;
    Ok(command_output(vec![
        ("task_id", OutputValue::String(status.task_id)),
        ("state", OutputValue::String(status.state)),
    ]))
}
