use super::{env_var_or_default, CLI_BINARY_ENV, DEFAULT_CLI_BINARY};
use std::env;

#[path = "live_probe_tranche_one/channel_task_probes.rs"]
mod channel_task_probes;
#[path = "live_probe_tranche_one/discovery_direct_message_probes.rs"]
mod discovery_direct_message_probes;
#[path = "live_probe_tranche_one/escrow_probe_support.rs"]
mod escrow_probe_support;

pub(super) fn cli_binary() -> String {
    env_var_or_default(CLI_BINARY_ENV, DEFAULT_CLI_BINARY)
}

pub(super) fn endpoint() -> String {
    env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080")
}

pub(super) fn env_payload(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_owned())
}

pub(super) fn agent_name(default: &str) -> String {
    super::env_var_or_default("KAMN_AGENT_NAME", default)
}

pub(super) fn validate_non_empty(value: &str, error: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(error.to_owned());
    }
    Ok(())
}

pub(super) fn run_live_s01_cli_health_probe() -> Result<(), String> {
    discovery_direct_message_probes::run_live_s01_cli_health_probe()
}

pub(super) fn run_live_s02_cli_direct_message_probe() -> Result<(), String> {
    discovery_direct_message_probes::run_live_s02_cli_direct_message_probe()
}

pub(super) fn run_live_s03_cli_group_channel_probe() -> Result<(), String> {
    channel_task_probes::run_live_s03_cli_group_channel_probe()
}

pub(super) fn run_live_s04_cli_task_lifecycle_probe() -> Result<(), String> {
    channel_task_probes::run_live_s04_cli_task_lifecycle_probe()
}

pub(super) fn run_live_s05_cli_escrow_settlement_probe() -> Result<(), String> {
    escrow_probe_support::run_live_s05_cli_escrow_settlement_probe()
}
