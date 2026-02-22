use crate::commands::unsupported;
use crate::ParsedCliArgs;
use kamn_agent_lib::AgentLibError;

/// Executes the verify_proof command.
pub fn execute(args: &ParsedCliArgs) -> Result<String, AgentLibError> {
    unsupported("verify_proof", args)
}
