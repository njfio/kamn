use crate::ParsedCliArgs;
use kamn_agent_lib::{AgentLibError, KamnAgentHandle};

const DEFAULT_AGENT_NAME: &str = "kamn-cli";
const DEFAULT_KOLME_ENDPOINT: &str = "http://localhost:3000";

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

pub(crate) fn connect_handle(args: &ParsedCliArgs) -> Result<KamnAgentHandle, AgentLibError> {
    let agent_name =
        std::env::var("KAMN_AGENT_NAME").unwrap_or_else(|_| DEFAULT_AGENT_NAME.to_owned());
    let kolme_endpoint =
        std::env::var("KAMN_KOLME_ENDPOINT").unwrap_or_else(|_| DEFAULT_KOLME_ENDPOINT.to_owned());
    KamnAgentHandle::connect(
        args.endpoint.as_str(),
        kolme_endpoint.as_str(),
        agent_name.as_str(),
    )
}

pub(crate) fn required_arg<'a>(
    args: &'a ParsedCliArgs,
    index: usize,
    field: &'static str,
) -> Result<&'a str, AgentLibError> {
    let value = args
        .passthrough
        .get(index)
        .ok_or_else(|| AgentLibError::InvalidInput {
            field,
            reason: "missing required argument".to_owned(),
        })?;
    if value.trim().is_empty() {
        return Err(AgentLibError::InvalidInput {
            field,
            reason: "argument must not be empty".to_owned(),
        });
    }
    Ok(value.as_str())
}
