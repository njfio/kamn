pub(crate) fn invalid_endpoint_updates() -> [(&'static str, Option<&'static str>); 2] {
    [
        ("KAMN_ENDPOINT", Some("invalid-endpoint")),
        ("KAMN_KOLME_ENDPOINT", Some("http://localhost:3000")),
    ]
}

pub(crate) fn invalid_endpoint_agent_updates() -> [(&'static str, Option<&'static str>); 3] {
    [
        ("KAMN_ENDPOINT", Some("not-a-valid-endpoint")),
        ("KAMN_KOLME_ENDPOINT", Some("http://localhost:3000")),
        ("KAMN_AGENT_NAME", Some("sdk-driver-test")),
    ]
}

pub(crate) fn invalid_failover_updates() -> [(&'static str, Option<&'static str>); 3] {
    [
        ("KAMN_ENDPOINT", Some("invalid-endpoint")),
        (
            "KAMN_E2E_S09_FAILOVER_ENDPOINT",
            Some("invalid-failover-endpoint"),
        ),
        ("KAMN_KOLME_ENDPOINT", Some("http://localhost:3000")),
    ]
}

pub(crate) fn invalid_topology_updates() -> [(&'static str, Option<&'static str>); 4] {
    [
        (
            "KAMN_E2E_S10_PRIMARY_ENDPOINT",
            Some("invalid-primary-endpoint"),
        ),
        (
            "KAMN_E2E_S10_SECONDARY_ENDPOINT",
            Some("invalid-secondary-endpoint"),
        ),
        (
            "KAMN_E2E_S10_TERTIARY_ENDPOINT",
            Some("invalid-tertiary-endpoint"),
        ),
        ("KAMN_KOLME_ENDPOINT", Some("http://localhost:3000")),
    ]
}
