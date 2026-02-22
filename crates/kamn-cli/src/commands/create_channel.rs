use crate::commands::{connect_handle, required_arg};
use crate::ParsedCliArgs;
use kamn_agent_lib::AgentLibError;

/// Executes the create_channel command.
pub fn execute(args: &ParsedCliArgs) -> Result<String, AgentLibError> {
    let payload = required_arg(args, 0, "create_channel_payload")?;
    let handle = connect_handle(args)?;
    let receipt = handle.create_channel(payload)?;
    Ok(format!(
        "channel_id={} status={}",
        receipt.channel_id, receipt.status
    ))
}
