/// Harness agent identity entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessIdentity {
    /// Agent name.
    pub name: String,
    /// Agent DID.
    pub did: String,
}

/// Returns deterministic default identity set for core scenarios.
pub fn default_identities() -> Vec<HarnessIdentity> {
    vec![
        HarnessIdentity {
            name: "alice".to_owned(),
            did: "kamn:did:agent:alice".to_owned(),
        },
        HarnessIdentity {
            name: "bob".to_owned(),
            did: "kamn:did:agent:bob".to_owned(),
        },
        HarnessIdentity {
            name: "carol".to_owned(),
            did: "kamn:did:agent:carol".to_owned(),
        },
    ]
}
