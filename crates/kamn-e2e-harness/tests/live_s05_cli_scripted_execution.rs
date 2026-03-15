use kamn_e2e_harness::drivers::cli_scripted::CliScriptedDriver;
use kamn_e2e_harness::drivers::HarnessDriver;

const REQUIRED_ENV_KEYS: &[&str] = &[
    "KAMN_E2E_CLI_SCRIPTED_LIVE",
    "KAMN_E2E_CLI_BINARY",
    "KAMN_ENDPOINT",
    "KAMN_AGENT_NAME",
];

#[test]
#[ignore = "requires local Kolme + KAMN runtime with explicit live env"]
fn integration_live_s05_cli_scripted_escrow_settlement_probe_against_local_runtime() {
    require_envs(REQUIRED_ENV_KEYS);

    let driver = CliScriptedDriver::from_env();
    let result = driver.execute("S-05");

    assert_eq!(result.scenario_id, "S-05");
    assert_eq!(
        result.status, "pass",
        "live cli-scripted S-05 failed: {:?}",
        result.detail
    );
}

fn require_envs(keys: &[&str]) {
    for key in keys {
        require_env(key);
    }
}

fn require_env(key: &str) {
    let value = std::env::var(key)
        .unwrap_or_else(|_| panic!("required env missing for live CLI S-05 probe: {key}"));
    assert!(
        !value.trim().is_empty(),
        "required env must not be empty for live CLI S-05 probe: {key}"
    );
}
