use super::super::*;

#[test]
fn rejects_missing_role() {
    assert_parse_error(cli_args(), missing_arg("--role"));
}

#[test]
fn rejects_planning_without_expected_state_hash() {
    let args = with_pairs(
        planning_args(),
        &[("--proposal", "tx-1|kamn:did:agent:aaa|1|state-1")],
    );
    assert_parse_error(args, missing_arg("--expected-state-hash"));
}

#[test]
fn rejects_planning_without_proposal() {
    let args = with_pairs(planning_args(), &[("--expected-state-hash", "state-1")]);
    assert_parse_error(args, missing_arg("--proposal"));
}

#[test]
fn rejects_recovery_check_without_expected_state_version() {
    let args = with_pairs(
        recovery_args(),
        &[
            ("--expected-state-hash", "state-42"),
            ("--rejoin-attempt", "node-a|42|state-42|resume-1"),
        ],
    );
    assert_parse_error(args, missing_arg("--expected-state-version"));
}

#[test]
fn rejects_recovery_check_without_expected_state_hash() {
    let args = with_pairs(
        recovery_args(),
        &[
            ("--expected-state-version", "42"),
            ("--rejoin-attempt", "node-a|42|state-42|resume-1"),
        ],
    );
    assert_parse_error(args, missing_arg("--expected-state-hash"));
}

#[test]
fn rejects_recovery_check_without_rejoin_attempt() {
    let args = with_pairs(
        recovery_args(),
        &[
            ("--expected-state-version", "42"),
            ("--expected-state-hash", "state-42"),
        ],
    );
    assert_parse_error(args, missing_arg("--rejoin-attempt"));
}
