use super::*;

#[test]
fn unit_run_live_s01_cli_health_probe_rejects_missing_binary() {
    let updates = endpoint_updates();
    assert_missing_binary_probe_failure(&updates, run_live_s01_cli_health_probe);
}

#[test]
fn unit_run_live_s02_cli_direct_message_probe_rejects_missing_binary() {
    let updates = endpoint_updates();
    assert_missing_binary_probe_failure(&updates, run_live_s02_cli_direct_message_probe);
}

#[test]
fn unit_run_live_s03_cli_group_channel_probe_rejects_missing_binary() {
    let updates = endpoint_updates();
    assert_missing_binary_probe_failure(&updates, run_live_s03_cli_group_channel_probe);
}

#[test]
fn unit_run_live_s04_cli_task_lifecycle_probe_rejects_missing_binary() {
    let updates = endpoint_updates();
    assert_missing_binary_probe_failure(&updates, run_live_s04_cli_task_lifecycle_probe);
}

#[test]
fn unit_run_live_s05_cli_escrow_settlement_probe_rejects_missing_binary() {
    let updates = endpoint_updates();
    assert_missing_binary_probe_failure(&updates, run_live_s05_cli_escrow_settlement_probe);
}

#[test]
fn unit_run_live_s06_cli_proof_verification_probe_rejects_missing_binary() {
    let updates = endpoint_updates();
    assert_missing_binary_probe_failure(&updates, run_live_s06_cli_proof_verification_probe);
}

#[test]
fn unit_run_live_s07_cli_replay_protection_probe_rejects_missing_binary() {
    let updates = endpoint_updates();
    assert_missing_binary_probe_failure(&updates, run_live_s07_cli_replay_protection_probe);
}

#[test]
fn unit_run_live_s08_cli_crash_recovery_probe_rejects_missing_binary() {
    let updates = endpoint_updates();
    assert_missing_binary_probe_failure(&updates, run_live_s08_cli_crash_recovery_probe);
}
