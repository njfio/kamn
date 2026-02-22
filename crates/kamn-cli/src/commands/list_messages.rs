use crate::commands::{connect_handle, required_arg};
use crate::ParsedCliArgs;
use kamn_agent_lib::AgentLibError;

/// Executes the list_messages command.
pub fn execute(args: &ParsedCliArgs) -> Result<String, AgentLibError> {
    let channel_id = required_arg(args, 0, "channel_id")?;
    let handle = connect_handle(args)?;
    let listing = handle.list_messages(channel_id)?;
    Ok(format!(
        "channel_id={} messages={}",
        listing.channel_id,
        listing.messages.join(",")
    ))
}
