use crate::commands::unsupported;
use crate::ParsedCliArgs;
use kamn_agent_lib::AgentLibError;

/// Executes the send_message command.
pub fn execute(args: &ParsedCliArgs) -> Result<String, AgentLibError> {
    unsupported("send_message", args)
}
