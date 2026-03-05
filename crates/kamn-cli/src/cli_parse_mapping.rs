use super::{CommandKind, OutputFormat};

const OUTPUT_FORMAT_MAPPINGS: &[(&str, OutputFormat)] =
    &[("json", OutputFormat::Json), ("text", OutputFormat::Text)];

const COMMAND_KIND_MAPPINGS: &[(&str, CommandKind)] = &[
    ("help", CommandKind::Help),
    ("--help", CommandKind::Help),
    ("-h", CommandKind::Help),
    ("register", CommandKind::Register),
    ("send-message", CommandKind::SendMessage),
    ("create-channel", CommandKind::CreateChannel),
    ("list-messages", CommandKind::ListMessages),
    ("query-message", CommandKind::QueryMessage),
    ("query-task", CommandKind::QueryTask),
    ("query-agent-profile", CommandKind::QueryAgentProfile),
    ("register-content", CommandKind::RegisterContent),
    ("expire-content", CommandKind::ExpireContent),
    ("tombstone-content", CommandKind::TombstoneContent),
    ("query-content", CommandKind::QueryContent),
    ("submit-bridge-message", CommandKind::SubmitBridgeMessage),
    ("forward-bridge-message", CommandKind::ForwardBridgeMessage),
    ("query-bridge-message", CommandKind::QueryBridgeMessage),
    ("create-task", CommandKind::CreateTask),
    ("accept-task", CommandKind::AcceptTask),
    ("complete-task", CommandKind::CompleteTask),
    ("fund-escrow", CommandKind::FundEscrow),
    ("release-escrow", CommandKind::ReleaseEscrow),
    ("verify-proof", CommandKind::VerifyProof),
    ("health", CommandKind::Health),
];

pub(super) fn parse_output_format(raw: &str) -> Result<OutputFormat, String> {
    match lookup_mapping(raw, OUTPUT_FORMAT_MAPPINGS) {
        Some(value) => Ok(value),
        None => Err(format!("unsupported format: {raw}")),
    }
}

pub(super) fn parse_command_kind(raw: &str) -> Result<CommandKind, String> {
    match lookup_mapping(raw, COMMAND_KIND_MAPPINGS) {
        Some(value) => Ok(value),
        None => Err(format!("unsupported command: {raw}")),
    }
}

fn lookup_mapping<T: Copy>(raw: &str, mappings: &[(&str, T)]) -> Option<T> {
    mappings.iter().find_map(|(label, value)| {
        if *label == raw {
            return Some(*value);
        }
        None
    })
}
