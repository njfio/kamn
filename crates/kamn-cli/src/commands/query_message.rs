use crate::commands::{connect_handle, required_arg};
use crate::ParsedCliArgs;
use kamn_agent_lib::AgentLibError;

/// Executes the query_message command.
pub fn execute(args: &ParsedCliArgs) -> Result<String, AgentLibError> {
    let message_id = required_arg(args, 0, "query_message_id")?;
    let handle = connect_handle(args)?;
    let status = handle.query_message(message_id)?;
    Ok(format!(
        "message_id={} status={}",
        status.message_id, status.status
    ))
}
