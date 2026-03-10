use super::super::*;

#[test]
fn regression_runtime_planning_rejects_duplicate_candidate_ids() {
    let args = with_pairs(
        planning_args(),
        &[
            ("--expected-state-hash", "state-1"),
            ("--proposal", "tx-1|kamn:did:agent:aaa|1|state-1"),
            ("--proposal", "tx-1|kamn:did:agent:bbb|2|state-1"),
        ],
    );
    let parsed = parse_cli(args, "planning args should parse");
    assert_eq!(
        execute(parsed),
        Err(ConfigError::RuntimePlanner("duplicate proposal candidate id: tx-1".to_owned()))
    );
}

#[test]
fn regression_runtime_planning_rejects_stale_state_hash() {
    let args = with_pairs(
        planning_args(),
        &[
            ("--expected-state-hash", "state-1"),
            ("--proposal", "tx-1|kamn:did:agent:aaa|1|state-2"),
        ],
    );
    let parsed = parse_cli(args, "planning args should parse");
    assert_eq!(
        execute(parsed),
        Err(ConfigError::RuntimePlanner(
            "proposal candidate state hash mismatch: expected state-1, found state-2".to_owned()
        ))
    );
}

#[test]
fn regression_runtime_recovery_rejects_replay_resume_token() {
    let args = with_pairs(
        recovery_args(),
        &[
            ("--expected-state-version", "42"),
            ("--expected-state-hash", "state-42"),
            ("--rejoin-attempt", "node-a|42|state-42|resume-1"),
            ("--rejoin-attempt", "node-a|42|state-42|resume-1"),
        ],
    );
    let parsed = parse_cli(args, "recovery-check args should parse");
    assert_eq!(
        execute(parsed),
        Err(ConfigError::RuntimeRecovery("rejoin resume token replayed: resume-1".to_owned()))
    );
}

#[test]
fn regression_runtime_recovery_rejects_state_version_mismatch() {
    let args = recovery_runtime_args("node-a|43|state-43|resume-1");
    let parsed = parse_cli(args, "recovery-check args should parse");
    assert_eq!(
        execute(parsed),
        Err(ConfigError::RuntimeRecovery(
            "rejoin state version mismatch: expected 42, found 43".to_owned()
        ))
    );
}

#[test]
fn regression_runtime_recovery_rejects_state_hash_mismatch() {
    let args = recovery_runtime_args("node-a|42|state-41|resume-1");
    let parsed = parse_cli(args, "recovery-check args should parse");
    assert_eq!(
        execute(parsed),
        Err(ConfigError::RuntimeRecovery(
            "rejoin state hash mismatch: expected state-42, found state-41".to_owned()
        ))
    );
}

fn recovery_runtime_args(rejoin_attempt: &str) -> Vec<String> {
    with_pairs(
        recovery_args(),
        &[
            ("--expected-state-version", "42"),
            ("--expected-state-hash", "state-42"),
            ("--rejoin-attempt", rejoin_attempt),
        ],
    )
}
