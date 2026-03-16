use kamn_e2e_harness::drivers::sdk_direct::SdkDirectDriver;
use kamn_e2e_harness::drivers::HarnessDriver;

const REQUIRED_ENV_KEYS: &[&str] = &[
    "KAMN_E2E_SDK_DIRECT_LIVE",
    "KAMN_ENDPOINT",
    "KAMN_KOLME_ENDPOINT",
    "KAMN_AGENT_NAME",
];

#[test]
#[ignore = "requires local Kolme + KAMN runtime with explicit live env"]
fn integration_live_s04_sdk_direct_task_lifecycle_probe_against_local_runtime() {
    require_envs(REQUIRED_ENV_KEYS);

    let driver = SdkDirectDriver::from_env();
    let result = driver.execute("S-04");

    assert_eq!(result.scenario_id, "S-04");
    assert_eq!(
        result.status, "pass",
        "live sdk-direct S-04 failed: {:?}",
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
        .unwrap_or_else(|_| panic!("required env missing for live S-04 probe: {key}"));
    assert!(
        !value.trim().is_empty(),
        "required env must not be empty for live S-04 probe: {key}"
    );
}
