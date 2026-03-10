pub(super) use crate::drivers::shared_helpers::{
    env_var_or_default, env_var_or_else, is_live_bound_scenario_id,
    live_execution_enabled_from_env as shared_live_execution_enabled_from_env,
    live_s07_probe_agent_suffix, parse_s15_budget_env_u128,
    validate_live_s05_release_escrow_response, validate_s15_latency_budget_samples,
};

const CLI_SCRIPTED_LIVE_ENV: &str = "KAMN_E2E_CLI_SCRIPTED_LIVE";
const CLI_BINARY_ENV: &str = "KAMN_E2E_CLI_BINARY";
const DEFAULT_CLI_BINARY: &str = "kamn-cli";
const AGENT_LIB_DETERMINISTIC_IDENTITY_OPT_IN_ENV: &str =
    "KAMN_AGENT_LIB_ALLOW_DETERMINISTIC_IDENTITY";
const AGENT_LIB_DETERMINISTIC_IDENTITY_OPT_IN_VALUE: &str = "1";
const DEFAULT_S02_AGENT_NAME: &str = "kamn-e2e-cli-s02";
const DEFAULT_S02_MESSAGE_PAYLOAD: &str = r#"{"message":"cli-scripted-live-s02"}"#;
const DEFAULT_S02_REPLY_PAYLOAD: &str = r#"{"message":"cli-scripted-live-s02-reply"}"#;
const DEFAULT_S03_AGENT_NAME: &str = "kamn-e2e-cli-s03";
const DEFAULT_S03_CHANNEL_PAYLOAD: &str =
    r#"{"name":"cli-scripted-live-s03","members":["alice","bob","carol"]}"#;
const DEFAULT_S03_MESSAGE_PAYLOAD: &str = r#"{"message":"cli-scripted-live-s03-channel-message"}"#;
const DEFAULT_S04_AGENT_NAME: &str = "kamn-e2e-cli-s04";
const DEFAULT_S04_CREATE_TASK_PAYLOAD: &str =
    r#"{"title":"cli-scripted-live-s04","description":"live task lifecycle probe"}"#;
const DEFAULT_S04_ESCROW_AMOUNT: u64 = 1;
const DEFAULT_S05_AGENT_NAME: &str = "kamn-e2e-cli-s05";
const DEFAULT_S05_FUND_ESCROW_PAYLOAD: &str = r#"{"task_id":"cli-scripted-live-s05","amount":1}"#;
const DEFAULT_S07_AGENT_NAME: &str = "kamn-e2e-cli-s07";
const DEFAULT_S07_MESSAGE_PAYLOAD: &str = r#"{"message":"cli-scripted-live-s07-replay"}"#;
const DEFAULT_S08_AGENT_NAME: &str = "kamn-e2e-cli-s08";
const DEFAULT_S08_PRE_MESSAGE_PAYLOAD: &str = r#"{"message":"cli-scripted-live-s08-pre"}"#;
const DEFAULT_S08_POST_MESSAGE_PAYLOAD: &str = r#"{"message":"cli-scripted-live-s08-post"}"#;
const DEFAULT_S09_AGENT_NAME: &str = "kamn-e2e-cli-s09";
const DEFAULT_S09_PRE_MESSAGE_PAYLOAD: &str = r#"{"message":"cli-scripted-live-s09-pre"}"#;
const DEFAULT_S09_POST_MESSAGE_PAYLOAD: &str = r#"{"message":"cli-scripted-live-s09-post"}"#;
const DEFAULT_S10_AGENT_NAME: &str = "kamn-e2e-cli-s10";
const DEFAULT_S10_MESSAGE_PAYLOAD: &str = r#"{"message":"cli-scripted-live-s10-topology"}"#;
const DEFAULT_S06_MESSAGE_ID: &str = "s06-live-proof";
const DEFAULT_S06_TX_HASH: &str = "sha256:s06-live-proof";
const DEFAULT_S06_BLOCK_HEIGHT: u64 = 1;
const DEFAULT_S06_FINALITY: &str = "final";

type LiveCliRunner = dyn Fn() -> Result<(), String> + Send + Sync + 'static;

#[path = "cli_scripted/command_support.rs"]
mod command_support;
#[path = "cli_scripted/driver_core.rs"]
mod driver_core;
#[path = "cli_scripted/live_probe_tranche_one.rs"]
mod live_probe_tranche_one;
#[path = "cli_scripted/live_probe_tranche_three.rs"]
mod live_probe_tranche_three;
#[path = "cli_scripted/live_probe_tranche_two.rs"]
mod live_probe_tranche_two;
#[path = "cli_scripted/runner_registry.rs"]
mod runner_registry;

pub(super) use command_support::{
    parse_text_output_field, run_cli_command_capture_stdout,
    run_cli_command_capture_stdout_with_agent_name, run_cli_command_expect_failure_with_agent_name,
};
pub use driver_core::CliScriptedDriver;
use live_probe_tranche_one::{
    run_live_s01_cli_health_probe, run_live_s02_cli_direct_message_probe,
    run_live_s03_cli_group_channel_probe, run_live_s04_cli_task_lifecycle_probe,
    run_live_s05_cli_escrow_settlement_probe,
};
use live_probe_tranche_three::{
    run_live_s11_cli_signer_rotation_probe, run_live_s12_cli_retention_deletion_probe,
    run_live_s13_cli_bridge_forwarding_probe, run_live_s14_cli_batch_merkle_probe,
    run_live_s15_cli_performance_smoke_probe,
};
use live_probe_tranche_two::{
    run_live_s06_cli_proof_verification_probe, run_live_s07_cli_replay_protection_probe,
    run_live_s08_cli_crash_recovery_probe, run_live_s09_cli_transport_failover_probe,
    run_live_s10_cli_topology_coherence_probe, validate_s08_distinct_message_ids,
    validate_s08_message_receipt_fields, validate_s08_query_message_response,
};

#[cfg(test)]
#[path = "cli_scripted_tests.rs"]
mod cli_scripted_tests;
