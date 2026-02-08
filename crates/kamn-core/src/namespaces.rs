#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateNamespaces {
    pub did_registry: &'static str,
    pub channels: &'static str,
    pub messages: &'static str,
    pub tasks: &'static str,
    pub reputation: &'static str,
    pub escrows: &'static str,
}

impl StateNamespaces {
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

    pub fn all_unique(&self) -> bool {
        let mut items = self.as_list();
        items.sort_unstable();
        items.windows(2).all(|pair| pair[0] != pair[1])
    }
}

impl Default for StateNamespaces {
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
