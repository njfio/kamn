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
    parse_mapping(raw, OUTPUT_FORMAT_MAPPINGS, "unsupported format")
}

pub(super) fn parse_command_kind(raw: &str) -> Result<CommandKind, String> {
    parse_mapping(raw, COMMAND_KIND_MAPPINGS, "unsupported command")
}

fn parse_mapping<T: Copy>(
    raw: &str,
    mappings: &[(&str, T)],
    unsupported_label: &str,
) -> Result<T, String> {
    match lookup_mapping(raw, mappings) {
        Some(value) => Ok(value),
        None => Err(format!("{unsupported_label}: {raw}")),
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
