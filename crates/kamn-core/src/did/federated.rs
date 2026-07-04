use std::collections::BTreeSet;

mod evaluator;
mod models;

pub use evaluator::*;
pub use models::*;

/// Public contract trait for Federated Did Trust Store.
pub trait FederatedDidTrustStore {
    /// Runs the is trusted contract operation.
    fn is_trusted(&self, network: &str, subject_did: &str) -> bool;
}

#[derive(Debug, Clone, Default)]
/// Public contract model for In Memory Federated Did Trust Store.
pub struct InMemoryFederatedDidTrustStore {
    entries: BTreeSet<(String, String)>,
}

impl InMemoryFederatedDidTrustStore {
    /// Creates a new value for this public contract type.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates or updates state through the from entries contract operation.
    pub fn from_entries<I, N, D>(entries: I) -> Self
    where
        I: IntoIterator<Item = (N, D)>,
        N: Into<String>,
        D: Into<String>,
    {
        let mut trust_store = Self::new();
        for (network, subject_did) in entries {
            trust_store.insert(network.into().as_str(), subject_did.into().as_str());
        }
        trust_store
    }

    /// Runs the insert contract operation.
    pub fn insert(&mut self, network: &str, subject_did: &str) {
        self.entries
            .insert((network.trim().to_owned(), subject_did.trim().to_owned()));
    }
}

impl FederatedDidTrustStore for InMemoryFederatedDidTrustStore {
    fn is_trusted(&self, network: &str, subject_did: &str) -> bool {
        self.entries
            .contains(&(network.trim().to_owned(), subject_did.trim().to_owned()))
    }
}
