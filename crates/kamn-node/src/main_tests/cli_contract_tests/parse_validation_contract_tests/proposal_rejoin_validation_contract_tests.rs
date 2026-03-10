use super::super::*;

#[test]
fn rejects_malformed_proposal_argument() {
    assert_parse_error(
        with_pairs(
            planning_args(),
            &[
                ("--expected-state-hash", "state-1"),
                ("--proposal", "tx-1|kamn:did:agent:aaa|state-1"),
            ],
        ),
        ConfigError::InvalidProposalArgument("tx-1|kamn:did:agent:aaa|state-1".to_owned()),
    );
}

#[test]
fn rejects_legacy_proposal_sender_did_argument() {
    assert_parse_error(
        with_pairs(
            planning_args(),
            &[
                ("--expected-state-hash", "state-1"),
                ("--proposal", "tx-1|did:kamn:agent:aaa|1|state-1"),
            ],
        ),
        ConfigError::InvalidProposalArgument("tx-1|did:kamn:agent:aaa|1|state-1".to_owned()),
    );
}

#[test]
fn rejects_malformed_rejoin_attempt_argument() {
    assert_parse_error(
        with_pairs(
            recovery_args(),
            &[
                ("--expected-state-version", "42"),
                ("--expected-state-hash", "state-42"),
                ("--rejoin-attempt", "node-a|42|state-42"),
            ],
        ),
        ConfigError::InvalidRejoinAttemptArgument("node-a|42|state-42".to_owned()),
    );
}

#[test]
fn rejects_invalid_expected_state_version_argument() {
    assert_parse_error(
        with_pairs(
            recovery_args(),
            &[
                ("--expected-state-version", "0"),
                ("--expected-state-hash", "state-42"),
                ("--rejoin-attempt", "node-a|42|state-42|resume-1"),
            ],
        ),
        ConfigError::InvalidExpectedStateVersion("0".to_owned()),
    );
}
