use super::*;

#[test]
fn unit_run_live_s08_cli_crash_recovery_probe_accepts_distinct_pre_post_message_ids() {
    let updates = endpoint_updates();
    assert_scripted_probe_succeeds(
        "kamn-e2e-cli-s08-success",
        S08_CONTINUITY_SCRIPT,
        &updates,
        run_live_s08_cli_crash_recovery_probe,
    );
}

#[test]
fn unit_run_live_s09_cli_transport_failover_probe_accepts_distinct_pre_post_message_ids() {
    let updates = failover_updates();
    assert_scripted_probe_succeeds(
        "kamn-e2e-cli-s09-success",
        S08_CONTINUITY_SCRIPT,
        &updates,
        run_live_s09_cli_transport_failover_probe,
    );
}

#[test]
fn unit_run_live_s10_cli_topology_coherence_probe_accepts_topology_query_continuity() {
    let updates = topology_updates();
    assert_scripted_probe_succeeds(
        "kamn-e2e-cli-s10-success",
        S10_TOPOLOGY_SCRIPT,
        &updates,
        run_live_s10_cli_topology_coherence_probe,
    );
}
