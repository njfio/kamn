use crate::commands::{connect_handle, required_arg};
use crate::ParsedCliArgs;
use kamn_agent_lib::AgentLibError;

/// Executes the create_task command.
pub fn execute(args: &ParsedCliArgs) -> Result<String, AgentLibError> {
    let payload = required_arg(args, 0, "create_task_payload")?;
    let handle = connect_handle(args)?;
    let receipt = handle.create_task(payload)?;
    Ok(format!(
        "task_id={} state={}",
        receipt.task_id, receipt.state
    ))
}
