pub(crate) fn concurrency_replay_fixture() -> [&'static [&'static str]; 3] {
    [
        &["kamn:did:agent:worker-1", "kamn:did:agent:worker-2", "kamn:did:agent:worker-3"],
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
