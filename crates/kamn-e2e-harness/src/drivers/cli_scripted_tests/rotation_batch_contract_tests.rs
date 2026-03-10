use super::*;

#[test]
fn unit_run_live_s11_cli_signer_rotation_probe_accepts_rotation_continuity() {
    let updates = s11_updates();
    assert_scripted_probe_succeeds(
        "kamn-e2e-cli-s11-success",
        S11_ROTATION_SCRIPT,
        &updates,
        run_live_s11_cli_signer_rotation_probe,
    );
}

#[test]
fn unit_run_live_s14_cli_batch_merkle_probe_accepts_distinct_batch_ids_and_final_proofs() {
    let updates = s14_updates();
    assert_scripted_probe_succeeds(
        "kamn-e2e-cli-s14-success",
        S14_BATCH_MERKLE_SCRIPT,
        &updates,
        run_live_s14_cli_batch_merkle_probe,
    );
}
