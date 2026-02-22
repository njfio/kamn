use kamn_cli::{parse_cli_args, CommandKind, OutputFormat};

#[test]
fn spec_c05_cli_exposes_required_phase2_subcommands() {
    let cases = [
        ("register", CommandKind::Register),
        ("send-message", CommandKind::SendMessage),
        ("create-channel", CommandKind::CreateChannel),
        ("list-messages", CommandKind::ListMessages),
        ("query-message", CommandKind::QueryMessage),
        ("create-task", CommandKind::CreateTask),
        ("accept-task", CommandKind::AcceptTask),
        ("complete-task", CommandKind::CompleteTask),
        ("fund-escrow", CommandKind::FundEscrow),
        ("release-escrow", CommandKind::ReleaseEscrow),
        ("verify-proof", CommandKind::VerifyProof),
        ("health", CommandKind::Health),
    ];

    for (command, expected_kind) in cases {
        let parsed = parse_cli_args(["kamn-cli", command]).expect("parse");
        assert_eq!(parsed.command, expected_kind, "command mismatch for {command}");
    }
}

#[test]
fn spec_c06_cli_supports_format_flag_and_env_defaults() {
    let parsed = parse_cli_args([
        "kamn-cli",
        "health",
        "--format",
        "json",
        "--endpoint",
        "http://localhost:8080",
    ])
    .expect("parse");

    assert_eq!(parsed.output_format, OutputFormat::Json);
    assert_eq!(parsed.endpoint, "http://localhost:8080");
}
