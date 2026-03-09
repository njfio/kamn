use super::*;

#[test]
fn unit_run_live_s01_mcp_probe_rejects_missing_binary() {
    assert_missing_binary_probe_failure(&base_probe_updates(), run_live_s01_mcp_probe);
}

#[test]
fn unit_run_live_s02_mcp_direct_message_probe_rejects_missing_binary() {
    assert_missing_binary_probe_failure(&base_probe_updates(), run_live_s02_mcp_direct_message_probe);
}

#[test]
fn unit_run_live_s03_mcp_group_channel_probe_rejects_missing_binary() {
    assert_missing_binary_probe_failure(&base_probe_updates(), run_live_s03_mcp_group_channel_probe);
}

#[test]
fn unit_run_live_s04_mcp_task_lifecycle_probe_rejects_missing_binary() {
    assert_missing_binary_probe_failure(&base_probe_updates(), run_live_s04_mcp_task_lifecycle_probe);
}

#[test]
fn unit_run_live_s05_mcp_escrow_settlement_probe_rejects_missing_binary() {
    assert_missing_binary_probe_failure(&base_probe_updates(), run_live_s05_mcp_escrow_settlement_probe);
}

#[test]
fn unit_run_live_s06_mcp_proof_verification_probe_rejects_missing_binary() {
    assert_missing_binary_probe_failure(&base_probe_updates(), run_live_s06_mcp_proof_verification_probe);
}

#[test]
fn unit_run_live_s07_mcp_replay_protection_probe_rejects_missing_binary() {
    assert_missing_binary_probe_failure(&replay_probe_updates(), run_live_s07_mcp_replay_protection_probe);
}

#[test]
fn unit_run_live_s08_mcp_crash_recovery_probe_rejects_missing_binary() {
    assert_missing_binary_probe_failure(&replay_probe_updates(), run_live_s08_mcp_crash_recovery_probe);
}

fn base_probe_updates() -> [(&'static str, Option<&'static str>); 3] {
    [
        ("KAMN_ENDPOINT", Some("http://localhost:8080")),
        ("KAMN_AGENT_NAME", Some("probe")),
        ("KAMN_AGENT_KEY_FILE", Some("/tmp/probe.key")),
    ]
}

fn replay_probe_updates() -> [(&'static str, Option<&'static str>); 2] {
    [
        ("KAMN_ENDPOINT", Some("http://localhost:8080")),
        ("KAMN_AGENT_KEY_FILE", Some("/tmp/probe.key")),
    ]
}
