use super::super::DOC;
use super::super::fairness_deletion_support::assert_contains_all;

pub(super) fn assert_runtime_local_contract_lane_markers(
    heading: &str,
    commands: &[&str],
    local_boundary: &str,
    policy_markers: &[&str],
    label: &str,
) {
    assert_contains_all(DOC, &[heading], label);
    assert_contains_all(DOC, commands, label);
    assert_mode_markers(label);
    assert_contains_all(DOC, &[local_boundary], label);
    assert_contains_all(DOC, policy_markers, label);
}

fn assert_mode_markers(label: &str) {
    assert_contains_all(
        DOC,
        &[
            "ci-fast-gate mode: fast",
            "local-dev mode: local",
            "manual-hardened mode: manual",
        ],
        label,
    );
}
