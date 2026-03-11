use super::super::*;

impl ServiceApiMessageStore {
    pub(crate) fn get_or_create_agent_profile(
        &mut self,
        agent_did: &str,
    ) -> Result<ServiceApiAgentGetBody, String> {
        let record = self.get_or_create_agent_record(agent_did)?;
        Ok(agent_profile_body(&record))
    }

    pub(crate) fn register_agent_profile(
        &mut self,
        agent_did: &str,
        registration: &ServiceApiAgentRegisterRequestBody,
    ) -> Result<ServiceApiAgentGetBody, ServiceApiAgentRegistrationStoreError> {
        self.refresh_from_disk()
            .map_err(ServiceApiAgentRegistrationStoreError::Persistence)?;
        let normalized_did = agent_did.trim();
        if normalized_did.is_empty() {
            return Err(ServiceApiAgentRegistrationStoreError::Persistence(
                "agent did must not be empty".to_owned(),
            ));
        }

        let expected_capabilities: Vec<String> = registration
            .capabilities
            .iter()
            .map(|value| value.trim().to_owned())
            .collect();

        let existing = self
            .snapshot
            .agents
            .get(normalized_did)
            .cloned()
            .unwrap_or_else(|| default_agent_record(normalized_did));
        if existing.registered
            && (record_agent_type(&existing) != registration.agent_type.trim()
                || record_model_family(&existing) != registration.model_family.trim()
                || record_capabilities(&existing) != expected_capabilities)
        {
            return Err(ServiceApiAgentRegistrationStoreError::Conflict(
                "agent registration metadata mismatch for existing did".to_owned(),
            ));
        }

        let mut record = existing;
        record.registered = true;
        record.agent_type = registration.agent_type.trim().to_owned();
        record.model_family = registration.model_family.trim().to_owned();
        record.capabilities = expected_capabilities;
        self.snapshot
            .agents
            .insert(normalized_did.to_owned(), record.clone());
        self.persist()
            .map_err(ServiceApiAgentRegistrationStoreError::Persistence)?;
        Ok(agent_profile_body(&record))
    }

    pub(crate) fn search_agent_profiles(
        &mut self,
        search: &ServiceApiAgentSearchRequestBody,
    ) -> Result<Vec<ServiceApiAgentGetBody>, String> {
        self.refresh_from_disk()?;
        let capability = search.capability.as_deref();
        let model_family = search.model_family.as_deref();
        let mut results: Vec<ServiceApiAgentGetBody> = self
            .snapshot
            .agents
            .values()
            .filter(|record| record.registered)
            .filter(|record| match model_family {
                Some(expected) => record_model_family(record) == expected,
                None => true,
            })
            .filter(|record| match capability {
                Some(expected) => record_capabilities(record)
                    .iter()
                    .any(|value| value == expected),
                None => true,
            })
            .map(agent_profile_body)
            .collect();
        results.sort_by(|left, right| left.did.cmp(&right.did));
        Ok(results)
    }

    pub(crate) fn get_or_create_agent_balance(
        &mut self,
        agent_did: &str,
    ) -> Result<ServiceApiAgentBalanceBody, String> {
        let record = self.get_or_create_agent_record(agent_did)?;
        Ok(ServiceApiAgentBalanceBody {
            did: record.did,
            balance: record.balance.unwrap_or(INITIAL_SERVICE_API_AGENT_BALANCE),
        })
    }

    fn get_or_create_agent_record(
        &mut self,
        agent_did: &str,
    ) -> Result<ServiceApiPersistedAgentRecord, String> {
        self.refresh_from_disk()?;
        let normalized_did = agent_did.trim();
        if normalized_did.is_empty() {
            return Err("agent did must not be empty".to_owned());
        }

        let mut persisted = false;
        let record = match self.snapshot.agents.get_mut(normalized_did) {
            Some(record) => {
                if record.balance.is_none() {
                    record.balance = Some(INITIAL_SERVICE_API_AGENT_BALANCE);
                    persisted = true;
                }
                record.clone()
            }
            None => {
                let record = default_agent_record(normalized_did);
                self.snapshot
                    .agents
                    .insert(normalized_did.to_owned(), record.clone());
                persisted = true;
                record
            }
        };

        if persisted {
            self.persist()?;
        }
        Ok(record)
    }
}

fn default_agent_record(agent_did: &str) -> ServiceApiPersistedAgentRecord {
    ServiceApiPersistedAgentRecord {
        did: agent_did.to_owned(),
        reputation_score: INITIAL_SERVICE_API_AGENT_REPUTATION_SCORE,
        balance: Some(INITIAL_SERVICE_API_AGENT_BALANCE),
        registered: false,
        agent_type: default_agent_type(),
        model_family: default_model_family(),
        capabilities: default_capabilities(),
    }
}

fn agent_profile_body(record: &ServiceApiPersistedAgentRecord) -> ServiceApiAgentGetBody {
    ServiceApiAgentGetBody {
        did: record.did.clone(),
        reputation_score: record.reputation_score,
        agent_type: record_agent_type(record),
        model_family: record_model_family(record),
        capabilities: record_capabilities(record),
    }
}

fn record_agent_type(record: &ServiceApiPersistedAgentRecord) -> String {
    if record.agent_type.trim().is_empty() {
        return default_agent_type();
    }
    record.agent_type.clone()
}

fn record_model_family(record: &ServiceApiPersistedAgentRecord) -> String {
    if record.model_family.trim().is_empty() {
        return default_model_family();
    }
    record.model_family.clone()
}

fn record_capabilities(record: &ServiceApiPersistedAgentRecord) -> Vec<String> {
    if record.capabilities.is_empty() {
        return default_capabilities();
    }
    record.capabilities.clone()
}

fn default_agent_type() -> String {
    "service-agent".to_owned()
}

fn default_model_family() -> String {
    "service-api".to_owned()
}

fn default_capabilities() -> Vec<String> {
    vec!["profile:read".to_owned()]
}


pub(crate) fn normalize_agent_did(candidate: Option<&str>, fallback: &str) -> String {
    match candidate {
        Some(value) if AgentDid::parse(value).is_ok() => value.to_owned(),
        _ => fallback.to_owned(),
    }
}
