use crate::commands::{connect_handle, required_arg};
use crate::ParsedCliArgs;
use kamn_agent_lib::AgentLibError;

/// Executes the send_message command.
pub fn execute(args: &ParsedCliArgs) -> Result<String, AgentLibError> {
    let payload = required_arg(args, 0, "send_message_payload")?;
    let handle = connect_handle(args)?;
    let receipt = handle.send_message(payload)?;
    Ok(format!(
        "message_id={} status={} runtime_mode={}",
        receipt.message_id, receipt.status, receipt.runtime_mode
    ))
}
