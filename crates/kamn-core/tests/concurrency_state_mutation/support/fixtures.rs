pub(crate) fn concurrency_replay_fixture() -> [&'static [&'static str]; 3] {
    [
        &[
            "kamn:did:agent:worker-1",
            "kamn:did:agent:worker-2",
            "kamn:did:agent:worker-3",
        ],
        &[
            "kamn:did:agent:worker-a",
            "kamn:did:agent:worker-b",
            "kamn:did:agent:worker-c",
            "kamn:did:agent:worker-d",
        ],
        &[
            "kamn:did:agent:worker-11",
            "kamn:did:agent:worker-12",
            "kamn:did:agent:worker-13",
            "kamn:did:agent:worker-14",
            "kamn:did:agent:worker-15",
        ],
    ]
}

pub(crate) fn regression_accept_contenders() -> [&'static str; 4] {
    [
        "kamn:did:agent:worker-reg-1",
        "kamn:did:agent:worker-reg-2",
        "kamn:did:agent:worker-reg-3",
        "kamn:did:agent:worker-reg-4",
    ]
}

pub(crate) fn performance_accept_contenders() -> [&'static str; 3] {
    [
        "kamn:did:agent:worker-perf-1",
        "kamn:did:agent:worker-perf-2",
        "kamn:did:agent:worker-perf-3",
    ]
}

pub(crate) fn deep_lane_accept_contenders() -> [&'static str; 5] {
    [
        "kamn:did:agent:worker-deep-1",
        "kamn:did:agent:worker-deep-2",
        "kamn:did:agent:worker-deep-3",
        "kamn:did:agent:worker-deep-4",
        "kamn:did:agent:worker-deep-5",
    ]
}
