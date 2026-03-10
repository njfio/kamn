pub(crate) fn endpoint_updates() -> [(&'static str, Option<&'static str>); 1] {
    [("KAMN_ENDPOINT", Some("http://localhost:8080"))]
}

pub(crate) fn endpoint_identity_reset_updates() -> [(&'static str, Option<&'static str>); 2] {
    [
        ("KAMN_ENDPOINT", Some("http://localhost:8080")),
        ("KAMN_AGENT_LIB_ALLOW_DETERMINISTIC_IDENTITY", None),
    ]
}

pub(crate) fn failover_updates() -> [(&'static str, Option<&'static str>); 2] {
    [
        ("KAMN_ENDPOINT", Some("http://localhost:8080")),
        (
            "KAMN_E2E_S09_FAILOVER_ENDPOINT",
            Some("http://localhost:8081"),
        ),
    ]
}

pub(crate) fn topology_updates() -> [(&'static str, Option<&'static str>); 3] {
    [
        (
            "KAMN_E2E_S10_PRIMARY_ENDPOINT",
            Some("http://localhost:8080"),
        ),
        (
            "KAMN_E2E_S10_SECONDARY_ENDPOINT",
            Some("http://localhost:8081"),
        ),
        (
            "KAMN_E2E_S10_TERTIARY_ENDPOINT",
            Some("http://localhost:8082"),
        ),
    ]
}

pub(crate) fn s11_updates() -> [(&'static str, Option<&'static str>); 3] {
    [
        ("KAMN_ENDPOINT", Some("http://localhost:8080")),
        (
            "KAMN_E2E_S11_PRIMARY_AGENT_NAME",
            Some("kamn-e2e-cli-s11-primary"),
        ),
        (
            "KAMN_E2E_S11_ROTATED_AGENT_NAME",
            Some("kamn-e2e-cli-s11-rotated"),
        ),
    ]
}

pub(crate) fn s14_updates() -> [(&'static str, Option<&'static str>); 2] {
    [
        ("KAMN_ENDPOINT", Some("http://localhost:8080")),
        ("KAMN_E2E_S14_AGENT_NAME", Some("kamn-e2e-cli-s14")),
    ]
}
