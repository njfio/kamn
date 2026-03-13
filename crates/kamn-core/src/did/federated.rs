use std::collections::BTreeSet;

mod evaluator;
mod models;

pub use evaluator::*;
pub use models::*;

pub trait FederatedDidTrustStore {
    fn is_trusted(&self, network: &str, subject_did: &str) -> bool;
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryFederatedDidTrustStore {
    entries: BTreeSet<(String, String)>,
}

impl InMemoryFederatedDidTrustStore {
    pub fn new() -> Self {
        Self::default()
    }

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
