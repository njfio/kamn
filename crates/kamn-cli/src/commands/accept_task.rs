use crate::commands::{connect_handle, required_arg};
use crate::ParsedCliArgs;
use kamn_agent_lib::AgentLibError;

/// Executes the accept_task command.
pub fn execute(args: &ParsedCliArgs) -> Result<String, AgentLibError> {
    let task_id = required_arg(args, 0, "task_id")?;
    let handle = connect_handle(args)?;
    let receipt = handle.accept_task(task_id)?;
    Ok(format!(
        "task_id={} state={}",
        receipt.task_id, receipt.state
    ))
}
