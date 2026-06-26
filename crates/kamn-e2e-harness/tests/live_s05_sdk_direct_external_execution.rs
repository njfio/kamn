use kamn_e2e_harness::drivers::sdk_direct::SdkDirectDriver;
use kamn_e2e_harness::drivers::HarnessDriver;

#[test]
#[ignore = "requires local Kolme + KAMN runtime with explicit live env"]
fn integration_live_s05_sdk_direct_escrow_settlement_probe_against_local_runtime() {
    require_env("KAMN_E2E_SDK_DIRECT_LIVE");
    require_env("KAMN_ENDPOINT");
    require_env("KAMN_KOLME_ENDPOINT");
    require_env("KAMN_AGENT_NAME");

    let driver = SdkDirectDriver::from_env();
    let result = driver.execute("S-05");

    assert_eq!(result.scenario_id, "S-05");
    assert_eq!(
        result.status, "pass",
        "live sdk-direct S-05 failed: {:?}",
        result.detail
    );
}

fn require_env(key: &str) {
    let value = std::env::var(key)
        .unwrap_or_else(|_| panic!("required env missing for live S-05 probe: {key}"));
    assert!(
        !value.trim().is_empty(),
        "required env must not be empty for live S-05 probe: {key}"
    );
}
