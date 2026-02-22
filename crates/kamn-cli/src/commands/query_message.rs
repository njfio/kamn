use crate::commands::unsupported;
use crate::ParsedCliArgs;
use kamn_agent_lib::AgentLibError;

/// Executes the query_message command.
pub fn execute(args: &ParsedCliArgs) -> Result<String, AgentLibError> {
    unsupported("query_message", args)
}
