use super::{cli_args::help_output, CommandKind, CommandOutput, ParsedCliArgs};

pub(super) fn dispatch_impl(
    parsed: &ParsedCliArgs,
) -> Result<CommandOutput, kamn_agent_lib::AgentLibError> {
    match parsed.command {
        CommandKind::Help => dispatch_help(),
        CommandKind::Register
        | CommandKind::SendMessage
        | CommandKind::CreateChannel
        | CommandKind::ListMessages
        | CommandKind::QueryMessage => dispatch_message_commands(parsed),
        CommandKind::RegisterContent
        | CommandKind::ExpireContent
        | CommandKind::TombstoneContent
        | CommandKind::QueryContent => dispatch_content_commands(parsed),
        CommandKind::SubmitBridgeMessage
        | CommandKind::ForwardBridgeMessage
        | CommandKind::QueryBridgeMessage
        | CommandKind::VerifyProof
        | CommandKind::Health => dispatch_bridge_and_health_commands(parsed),
        CommandKind::QueryTask
        | CommandKind::QueryAgentProfile
        | CommandKind::CreateTask
        | CommandKind::AcceptTask
        | CommandKind::CompleteTask
        | CommandKind::FundEscrow
        | CommandKind::ReleaseEscrow => dispatch_task_commands(parsed),
    }
}

fn dispatch_help() -> Result<CommandOutput, kamn_agent_lib::AgentLibError> {
    Ok(help_output())
}

fn dispatch_message_commands(
    parsed: &ParsedCliArgs,
) -> Result<CommandOutput, kamn_agent_lib::AgentLibError> {
    match parsed.command {
        CommandKind::Register => super::commands::register::execute(parsed),
        CommandKind::SendMessage => super::commands::send_message::execute(parsed),
        CommandKind::CreateChannel => super::commands::create_channel::execute(parsed),
        CommandKind::ListMessages => super::commands::list_messages::execute(parsed),
        CommandKind::QueryMessage => super::commands::query_message::execute(parsed),
        _ => unreachable!("dispatch_message_commands only accepts message commands"),
    }
}

fn dispatch_content_commands(
    parsed: &ParsedCliArgs,
) -> Result<CommandOutput, kamn_agent_lib::AgentLibError> {
    match parsed.command {
        CommandKind::RegisterContent => super::commands::register_content::execute(parsed),
        CommandKind::ExpireContent => super::commands::expire_content::execute(parsed),
        CommandKind::TombstoneContent => super::commands::tombstone_content::execute(parsed),
        CommandKind::QueryContent => super::commands::query_content::execute(parsed),
        _ => unreachable!("dispatch_content_commands only accepts content commands"),
    }
}

fn dispatch_bridge_and_health_commands(
    parsed: &ParsedCliArgs,
) -> Result<CommandOutput, kamn_agent_lib::AgentLibError> {
    match parsed.command {
        CommandKind::SubmitBridgeMessage => super::commands::submit_bridge_message::execute(parsed),
        CommandKind::ForwardBridgeMessage => {
            super::commands::forward_bridge_message::execute(parsed)
        }
        CommandKind::QueryBridgeMessage => super::commands::query_bridge_message::execute(parsed),
        CommandKind::VerifyProof => super::commands::verify_proof::execute(parsed),
        CommandKind::Health => super::commands::health::execute(parsed),
        _ => {
            unreachable!("dispatch_bridge_and_health_commands only accepts bridge/health commands")
        }
    }
}

fn dispatch_task_commands(
    parsed: &ParsedCliArgs,
) -> Result<CommandOutput, kamn_agent_lib::AgentLibError> {
    match parsed.command {
        CommandKind::QueryTask => super::commands::query_task::execute(parsed),
        CommandKind::QueryAgentProfile => super::commands::query_agent_profile::execute(parsed),
        CommandKind::CreateTask => super::commands::create_task::execute(parsed),
        CommandKind::AcceptTask => super::commands::accept_task::execute(parsed),
        CommandKind::CompleteTask => super::commands::complete_task::execute(parsed),
        CommandKind::FundEscrow => super::commands::fund_escrow::execute(parsed),
        CommandKind::ReleaseEscrow => super::commands::release_escrow::execute(parsed),
        _ => unreachable!("dispatch_task_commands only accepts task commands"),
    }
}
