use crate::ParsedCliArgs;
use kamn_agent_lib::AgentLibError;

/// `accept-task` command module.
pub mod accept_task;
/// `complete-task` command module.
pub mod complete_task;
/// `create-channel` command module.
pub mod create_channel;
/// `create-task` command module.
pub mod create_task;
/// `fund-escrow` command module.
pub mod fund_escrow;
/// `health` command module.
pub mod health;
/// `list-messages` command module.
pub mod list_messages;
/// `query-message` command module.
pub mod query_message;
/// `register` command module.
pub mod register;
/// `release-escrow` command module.
pub mod release_escrow;
/// `send-message` command module.
pub mod send_message;
/// `verify-proof` command module.
pub mod verify_proof;

fn unsupported(operation: &'static str, args: &ParsedCliArgs) -> Result<String, AgentLibError> {
    let _ = (&args.endpoint, args.output_format, args.passthrough.len());
    Err(AgentLibError::UnsupportedOperation(operation))
}
