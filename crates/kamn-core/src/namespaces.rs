//! State namespace constants and helper utilities for top-level KAMN domains.

/// Canonical state-namespace labels for top-level KAMN domains.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateNamespaces {
    /// Namespace key used for DID registry records.
    pub did_registry: &'static str,
    /// Namespace key used for channel and membership records.
    pub channels: &'static str,
    /// Namespace key used for message envelopes and delivery state.
    pub messages: &'static str,
    /// Namespace key used for task lifecycle and DAG state.
    pub tasks: &'static str,
    /// Namespace key used for reputation and trust scoring state.
    pub reputation: &'static str,
    /// Namespace key used for escrow lifecycle state.
    pub escrows: &'static str,
}

impl StateNamespaces {
    /// Returns all namespace identifiers in deterministic domain order.
    pub fn as_list(&self) -> [&'static str; 6] {
        [
            self.did_registry,
            self.channels,
            self.messages,
            self.tasks,
            self.reputation,
            self.escrows,
        ]
    }

    /// Returns true when all namespace identifiers are pairwise distinct.
    pub fn all_unique(&self) -> bool {
        let mut items = self.as_list();
        items.sort_unstable();
        items.windows(2).all(|pair| pair[0] != pair[1])
    }
}

impl Default for StateNamespaces {
    /// Builds the default namespace mapping used by baseline runtime state.
    fn default() -> Self {
        Self {
            did_registry: "kamn.identity.did_registry",
            channels: "kamn.channels.membership",
            messages: "kamn.messaging.envelopes",
            tasks: "kamn.tasks.state",
            reputation: "kamn.reputation.scores",
            escrows: "kamn.economics.escrows",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StateNamespaces;

    #[test]
    fn default_namespaces_are_unique() {
        let namespaces = StateNamespaces::default();
        assert!(namespaces.all_unique());
    }

    #[test]
    fn list_contains_expected_prefixes() {
        let namespaces = StateNamespaces::default();
        for item in namespaces.as_list() {
            assert!(item.starts_with("kamn."));
        }
    }
}
