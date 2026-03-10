use super::*;

#[test]
fn unit_run_live_s01_cli_health_probe_sets_deterministic_identity_opt_in_env() {
    let updates = endpoint_identity_reset_updates();
    assert_scripted_probe_succeeds(
        "kamn-e2e-cli-s01-deterministic-opt-in",
        HEALTH_DETERMINISTIC_OPT_IN_SCRIPT,
        &updates,
        run_live_s01_cli_health_probe,
    );
}

#[test]
fn unit_run_live_s03_cli_group_channel_probe_rejects_query_message_id_mismatch() {
    let updates = endpoint_updates();
    assert_scripted_probe_error_contains(
        "kamn-e2e-cli-s03-query-mismatch",
        S03_QUERY_MISMATCH_SCRIPT,
        &updates,
        "mismatched message_id",
        run_live_s03_cli_group_channel_probe,
    );
}

#[test]
fn unit_run_live_s03_cli_group_channel_probe_rejects_list_channel_id_mismatch() {
    let updates = endpoint_updates();
    assert_scripted_probe_error_contains(
        "kamn-e2e-cli-s03-list-mismatch",
        S03_LIST_MISMATCH_SCRIPT,
        &updates,
        "mismatched channel_id",
        run_live_s03_cli_group_channel_probe,
    );
}

#[test]
fn unit_run_live_s06_cli_proof_verification_probe_accepts_success_payload() {
    let script = s06_success_payload_script();
    let updates = endpoint_updates();
    assert_scripted_probe_succeeds(
        "kamn-e2e-cli-s06-success",
        script.as_str(),
        &updates,
        run_live_s06_cli_proof_verification_probe,
    );
}
