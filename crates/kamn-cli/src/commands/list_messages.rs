use crate::commands::unsupported;
use crate::ParsedCliArgs;
use kamn_agent_lib::AgentLibError;

/// Executes the list_messages command.
pub fn execute(args: &ParsedCliArgs) -> Result<String, AgentLibError> {
    unsupported("list_messages", args)
}
