use super::{cli, DiagnosticsMode, OutputMode, RuntimeMode};
use kamn_core::{NodeRole, SyncMode};

#[test]
fn cli_module_parses_required_role_and_defaults() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
    ];

    let parsed = cli::parse_args(args).expect("args should parse");
    assert_eq!(parsed.role, NodeRole::Processor);
    assert_eq!(parsed.chain_id, "kamn-devnet");
    assert_eq!(parsed.chain_version, "v0.1.0");
    assert_eq!(parsed.storage_dir, "./data");
    assert!(parsed.enable_gossip);
    assert_eq!(parsed.sync_mode, SyncMode::Fast);
    assert_eq!(parsed.runtime_mode, RuntimeMode::bootstrap());
    assert!(parsed.daemon_shutdown_signal_ticks.is_empty());
    assert_eq!(parsed.daemon_shutdown_drain_ticks, None);
    assert_eq!(parsed.daemon_shutdown_timeout_ticks, None);
    assert_eq!(parsed.output_mode, OutputMode::text());
    assert_eq!(parsed.diagnostics_mode, DiagnosticsMode::basic());
}
