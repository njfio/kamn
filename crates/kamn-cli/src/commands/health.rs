use crate::commands::unsupported;
use crate::ParsedCliArgs;
use kamn_agent_lib::AgentLibError;

/// Executes the health command.
pub fn execute(args: &ParsedCliArgs) -> Result<String, AgentLibError> {
    unsupported("health", args)
}
