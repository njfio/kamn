use std::fs;

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

#[test]
fn cli_module_extraction_contract_declares_config_layering_module() {
    let cli_rs = read_repo_file("src/cli.rs");
    assert!(
        cli_rs.contains("mod cli_config_layering;"),
        "cli.rs should declare cli_config_layering module"
    );
    assert!(
        cli_rs.contains("mod cli_value_parsers;"),
        "cli.rs should declare cli_value_parsers module"
    );
    assert!(
        cli_rs.contains("mod cli_runtime_mode_validation;"),
        "cli.rs should declare cli_runtime_mode_validation module"
    );
    assert!(
        cli_rs.contains("mod cli_post_parse_guards;"),
        "cli.rs should declare cli_post_parse_guards module"
    );
    assert!(
        cli_rs.contains("mod cli_endpoint_option_parsing;"),
        "cli.rs should declare cli_endpoint_option_parsing module"
    );
    assert!(
        cli_rs.contains("mod cli_daemon_option_parsing;"),
        "cli.rs should declare cli_daemon_option_parsing module"
    );
    assert!(
        cli_rs.contains("mod cli_kolme_live_option_parsing;"),
        "cli.rs should declare cli_kolme_live_option_parsing module"
    );
    assert!(
        cli_rs.contains("mod cli_planning_recovery_option_parsing;"),
        "cli.rs should declare cli_planning_recovery_option_parsing module"
    );
    assert!(
        cli_rs.contains("mod cli_core_common_option_parsing;"),
        "cli.rs should declare cli_core_common_option_parsing module"
    );
    assert!(
        cli_rs.contains("mod cli_parse_state;"),
        "cli.rs should declare cli_parse_state module"
    );
    assert!(
        cli_rs.contains("mod cli_parse_loop;"),
        "cli.rs should declare cli_parse_loop module"
    );
}

#[test]
fn cli_module_extraction_contract_removes_inline_config_layering_helpers() {
    let cli_rs = read_repo_file("src/cli.rs");
    for marker in [
        "fn read_env_var_trimmed(",
        "fn parse_bool_override(",
        "fn push_key_value_flag(",
        "fn map_config_entry_to_args(",
        "fn parse_config_file_args(",
        "fn append_env_override(",
        "fn collect_env_override_args(",
        "fn extract_config_file_path(",
        "fn build_layered_cli_args(",
    ] {
        assert!(
            !cli_rs.contains(marker),
            "cli.rs should not keep inline config layering helper: {marker}"
        );
    }
}

#[test]
fn cli_module_extraction_contract_keeps_helpers_in_new_module() {
    let config_layering_rs = read_repo_file("src/cli_config_layering.rs");
    let config_mapping_rs = read_repo_file("src/cli_config_layering/config_mapping.rs");
    assert!(
        config_layering_rs.contains("pub(super) fn build_layered_cli_args("),
        "cli_config_layering module should expose layered cli arg builder"
    );
    assert!(
        config_layering_rs.contains("mod config_mapping;"),
        "cli_config_layering module should declare config_mapping submodule"
    );
    assert!(
        config_mapping_rs.contains("fn map_config_entry_to_args("),
        "config_mapping submodule should own config entry-to-arg mapping"
    );
}

#[test]
fn cli_module_extraction_contract_removes_inline_value_parser_helpers() {
    let cli_rs = read_repo_file("src/cli.rs");
    for marker in [
        "fn parse_state_version_arg(",
        "fn parse_proposal_candidate(",
        "fn parse_rejoin_attempt(",
        "fn parse_daemon_control_arg(",
        "fn parse_daemon_lifecycle_event(",
    ] {
        assert!(
            !cli_rs.contains(marker),
            "cli.rs should not keep inline value parser helper: {marker}"
        );
    }
    let value_parsers_rs = read_repo_file("src/cli_value_parsers.rs");
    assert!(
        value_parsers_rs.contains("pub(super) fn parse_state_version_arg("),
        "cli_value_parsers module should expose state-version parser helper"
    );
    assert!(
        value_parsers_rs.contains("pub(super) fn parse_daemon_lifecycle_event("),
        "cli_value_parsers module should expose daemon lifecycle parser helper"
    );
}

#[test]
fn cli_module_extraction_contract_removes_inline_runtime_mode_guards() {
    let cli_rs = read_repo_file("src/cli.rs");
    for marker in [
        "if runtime_mode.kind == RuntimeModeKind::Planning {",
        "if runtime_mode.kind == RuntimeModeKind::RecoveryCheck {",
        "if matches!(",
        "RuntimeModeKind::Daemon | RuntimeModeKind::Full",
        "if runtime_mode.kind == RuntimeModeKind::KolmeLive {",
    ] {
        assert!(
            !cli_rs.contains(marker),
            "cli.rs should not keep inline runtime-mode validation marker: {marker}"
        );
    }
    let runtime_mode_validation_rs = read_repo_file("src/cli_runtime_mode_validation.rs");
    assert!(
        runtime_mode_validation_rs.contains("pub(super) fn validate_runtime_mode_requirements("),
        "cli_runtime_mode_validation module should expose runtime-mode validation entrypoint"
    );
}

#[test]
fn cli_module_extraction_contract_removes_inline_post_parse_guards() {
    let cli_rs = read_repo_file("src/cli.rs");
    for marker in [
        "if let Some(selected_profile) = profile {",
        "if api_bind_addr.is_none()",
        "if observability_endpoint_bind_addr.is_none()",
        "if observability_endpoint_bind_addr.is_some()",
    ] {
        assert!(
            !cli_rs.contains(marker),
            "cli.rs should not keep inline post-parse guard marker: {marker}"
        );
    }
    let post_parse_guards_rs = read_repo_file("src/cli_post_parse_guards.rs");
    assert!(
        post_parse_guards_rs.contains("pub(super) fn apply_profile_defaults("),
        "cli_post_parse_guards module should expose profile defaults helper"
    );
    assert!(
        post_parse_guards_rs.contains("pub(super) fn validate_endpoint_guards("),
        "cli_post_parse_guards module should expose endpoint guard validator"
    );
}

#[test]
fn cli_module_extraction_contract_removes_inline_endpoint_option_parsing() {
    let cli_rs = read_repo_file("src/cli.rs");
    for marker in [
        "\"--api-bind\" => {",
        "\"--api-max-requests\" => {",
        "\"--api-idle-timeout-ms\" => {",
        "\"--api-body-limit-bytes\" => {",
        "\"--api-concurrency-limit\" => {",
        "\"--api-rate-limit-per-second\" => {",
        "\"--observability-endpoint-bind\" => {",
        "\"--observability-endpoint-metrics-path\" => {",
        "\"--observability-endpoint-health-path\" => {",
        "\"--observability-endpoint-max-requests\" => {",
        "\"--observability-endpoint-idle-timeout-ms\" => {",
    ] {
        assert!(
            !cli_rs.contains(marker),
            "cli.rs should not keep inline endpoint option parsing marker: {marker}"
        );
    }
    let endpoint_option_parsing_rs = read_repo_file("src/cli_endpoint_option_parsing.rs");
    assert!(
        endpoint_option_parsing_rs.contains("pub(super) struct EndpointOptionState"),
        "cli_endpoint_option_parsing module should define endpoint option state"
    );
    assert!(
        endpoint_option_parsing_rs.contains("pub(super) fn try_parse_endpoint_option("),
        "cli_endpoint_option_parsing module should expose endpoint option parser entrypoint"
    );
}

#[test]
fn cli_module_extraction_contract_removes_inline_daemon_option_parsing() {
    let cli_rs = read_repo_file("src/cli.rs");
    for marker in [
        "\"--daemon-max-ticks\" => {",
        "\"--daemon-tick-interval-ms\" => {",
        "\"--daemon-shutdown-signal-tick\" => {",
        "\"--daemon-shutdown-os-signals\" => {",
        "\"--daemon-shutdown-drain-ticks\" => {",
        "\"--daemon-shutdown-timeout-ticks\" => {",
        "\"--daemon-peer-id\" => {",
        "\"--daemon-lifecycle-event\" => {",
    ] {
        assert!(
            !cli_rs.contains(marker),
            "cli.rs should not keep inline daemon option parsing marker: {marker}"
        );
    }
    let daemon_option_parsing_rs = read_repo_file("src/cli_daemon_option_parsing.rs");
    assert!(
        daemon_option_parsing_rs.contains("pub(super) struct DaemonOptionState"),
        "cli_daemon_option_parsing module should define daemon option state"
    );
    assert!(
        daemon_option_parsing_rs.contains("pub(super) fn try_parse_daemon_option("),
        "cli_daemon_option_parsing module should expose daemon option parser entrypoint"
    );
}

#[test]
fn cli_module_extraction_contract_removes_inline_kolme_live_option_parsing() {
    let cli_rs = read_repo_file("src/cli.rs");
    for marker in [
        "\"--kolme-live-base-url\" => {",
        "\"--kolme-live-provider-hint\" => {",
        "\"--kolme-live-signing-profile\" => {",
        "\"--kolme-live-strict-signer-contracts\" => {",
        "\"--kolme-live-signer-profile\" => {",
        "\"--kolme-live-signer-key-source\" => {",
    ] {
        assert!(
            !cli_rs.contains(marker),
            "cli.rs should not keep inline kolme live option parsing marker: {marker}"
        );
    }
    let kolme_live_option_parsing_rs = read_repo_file("src/cli_kolme_live_option_parsing.rs");
    assert!(
        kolme_live_option_parsing_rs.contains("pub(super) struct KolmeLiveOptionState"),
        "cli_kolme_live_option_parsing module should define kolme live option state"
    );
    assert!(
        kolme_live_option_parsing_rs.contains("pub(super) fn try_parse_kolme_live_option("),
        "cli_kolme_live_option_parsing module should expose kolme live option parser entrypoint"
    );
}

#[test]
fn cli_module_extraction_contract_removes_inline_planning_recovery_option_parsing() {
    let cli_rs = read_repo_file("src/cli.rs");
    for marker in [
        "\"--expected-state-version\" => {",
        "\"--expected-state-hash\" => {",
        "\"--proposal\" => {",
        "\"--rejoin-attempt\" => {",
    ] {
        assert!(
            !cli_rs.contains(marker),
            "cli.rs should not keep inline planning/recovery option parsing marker: {marker}"
        );
    }
    let planning_recovery_option_parsing_rs =
        read_repo_file("src/cli_planning_recovery_option_parsing.rs");
    assert!(
        planning_recovery_option_parsing_rs
            .contains("pub(super) struct PlanningRecoveryOptionState"),
        "cli_planning_recovery_option_parsing module should define planning/recovery option state"
    );
    assert!(
        planning_recovery_option_parsing_rs.contains("pub(super) fn try_parse_planning_recovery_option("),
        "cli_planning_recovery_option_parsing module should expose planning/recovery parser entrypoint"
    );
}

#[test]
fn cli_module_extraction_contract_removes_inline_core_common_option_parsing() {
    let cli_rs = read_repo_file("src/cli.rs");
    for marker in [
        "\"--role\" => {",
        "\"--profile\" => {",
        "\"--chain-id\" => {",
        "\"--chain-version\" => {",
        "\"--storage-dir\" => {",
        "\"--disable-gossip\" => {",
        "\"--sync-mode\" => {",
        "\"--runtime-mode\" => {",
        "\"--output\" => {",
        "\"--diagnostics\" => {",
    ] {
        assert!(
            !cli_rs.contains(marker),
            "cli.rs should not keep inline core/common option parsing marker: {marker}"
        );
    }
    let core_common_option_parsing_rs = read_repo_file("src/cli_core_common_option_parsing.rs");
    assert!(
        core_common_option_parsing_rs.contains("pub(super) struct CoreCommonOptionState"),
        "cli_core_common_option_parsing module should define core/common option state"
    );
    assert!(
        core_common_option_parsing_rs.contains("pub(super) fn try_parse_core_common_option("),
        "cli_core_common_option_parsing module should expose core/common parser entrypoint"
    );
}

#[test]
fn cli_module_extraction_contract_removes_inline_parse_state_initialization() {
    let cli_rs = read_repo_file("src/cli.rs");
    for marker in [
        "let mut role: Option<NodeRole> = None;",
        "let mut daemon_shutdown_signal_ticks: Vec<u64> = Vec::new();",
        "let mut observability_endpoint_idle_timeout_ms_overridden = false;",
        "let mut iter = layered_args.into_iter();",
    ] {
        assert!(
            !cli_rs.contains(marker),
            "cli.rs should not keep inline parse-state initialization marker: {marker}"
        );
    }
    let parse_state_rs = read_repo_file("src/cli_parse_state.rs");
    assert!(
        parse_state_rs.contains("pub(super) struct CliParseState"),
        "cli_parse_state module should define parse-state struct"
    );
    assert!(
        parse_state_rs.contains("pub(super) fn new() -> Self"),
        "cli_parse_state module should expose state constructor"
    );
    let parse_loop_rs = read_repo_file("src/cli_parse_loop.rs");
    assert!(
        parse_loop_rs.contains("pub(super) fn parse_layered_args_into_state("),
        "cli_parse_loop module should expose layered-arg parse entrypoint"
    );
}
