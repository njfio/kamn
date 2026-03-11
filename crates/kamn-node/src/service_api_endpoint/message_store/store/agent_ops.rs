use super::super::*;

mod defaults;

pub(crate) use defaults::normalize_agent_did;
use defaults::{
    agent_profile_body, default_agent_record, record_agent_type, record_capabilities,
    record_model_family,
};

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
        let normalized_did = validate_agent_did(agent_did)
            .map_err(ServiceApiAgentRegistrationStoreError::Persistence)?;
        let expected_capabilities = trimmed_capabilities(registration);
        let mut record = existing_or_default_agent(self, normalized_did);
        ensure_agent_registration_matches(&record, registration, &expected_capabilities)?;
        apply_agent_registration(&mut record, registration, expected_capabilities);
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
        let mut results: Vec<ServiceApiAgentGetBody> = self
            .snapshot
            .agents
            .values()
            .filter(|record| agent_matches_search(record, search))
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
        let normalized_did = validate_agent_did(agent_did)?;
        let (record, persisted) = load_or_insert_agent_record(self, normalized_did);
        if persisted {
            self.persist()?;
        }
        Ok(record)
    }
}

fn validate_agent_did(agent_did: &str) -> Result<&str, String> {
    let normalized_did = agent_did.trim();
    if normalized_did.is_empty() {
        return Err("agent did must not be empty".to_owned());
    }
    Ok(normalized_did)
}

fn trimmed_capabilities(registration: &ServiceApiAgentRegisterRequestBody) -> Vec<String> {
    registration
        .capabilities
        .iter()
        .map(|value| value.trim().to_owned())
        .collect()
}

fn existing_or_default_agent(
    store: &ServiceApiMessageStore,
    normalized_did: &str,
) -> ServiceApiPersistedAgentRecord {
    store
        .snapshot
        .agents
        .get(normalized_did)
        .cloned()
        .unwrap_or_else(|| default_agent_record(normalized_did))
}

fn ensure_agent_registration_matches(
    existing: &ServiceApiPersistedAgentRecord,
    registration: &ServiceApiAgentRegisterRequestBody,
    expected_capabilities: &[String],
) -> Result<(), ServiceApiAgentRegistrationStoreError> {
    let matches = !existing.registered
        || (record_agent_type(existing) == registration.agent_type.trim()
            && record_model_family(existing) == registration.model_family.trim()
            && record_capabilities(existing) == expected_capabilities);
    if matches {
        return Ok(());
    }
    Err(ServiceApiAgentRegistrationStoreError::Conflict(
        "agent registration metadata mismatch for existing did".to_owned(),
    ))
}

fn apply_agent_registration(
    record: &mut ServiceApiPersistedAgentRecord,
    registration: &ServiceApiAgentRegisterRequestBody,
    expected_capabilities: Vec<String>,
) {
    record.registered = true;
    record.agent_type = registration.agent_type.trim().to_owned();
    record.model_family = registration.model_family.trim().to_owned();
    record.capabilities = expected_capabilities;
}

fn agent_matches_search(
    record: &ServiceApiPersistedAgentRecord,
    search: &ServiceApiAgentSearchRequestBody,
) -> bool {
    record.registered
        && model_family_matches(record, search.model_family.as_deref())
        && capability_matches(record, search.capability.as_deref())
}

fn model_family_matches(record: &ServiceApiPersistedAgentRecord, expected: Option<&str>) -> bool {
    expected.is_none_or(|value| record_model_family(record) == value)
}

fn capability_matches(record: &ServiceApiPersistedAgentRecord, expected: Option<&str>) -> bool {
    expected.is_none_or(|value| {
        record_capabilities(record)
            .iter()
            .any(|candidate| candidate == value)
    })
}

fn load_or_insert_agent_record(
    store: &mut ServiceApiMessageStore,
    normalized_did: &str,
) -> (ServiceApiPersistedAgentRecord, bool) {
    match store.snapshot.agents.get_mut(normalized_did) {
        Some(record) => ensure_agent_balance(record),
        None => insert_default_agent_record(store, normalized_did),
    }
}

fn ensure_agent_balance(
    record: &mut ServiceApiPersistedAgentRecord,
) -> (ServiceApiPersistedAgentRecord, bool) {
    if record.balance.is_none() {
        record.balance = Some(INITIAL_SERVICE_API_AGENT_BALANCE);
        return (record.clone(), true);
    }
    (record.clone(), false)
}

fn insert_default_agent_record(
    store: &mut ServiceApiMessageStore,
    normalized_did: &str,
) -> (ServiceApiPersistedAgentRecord, bool) {
    let record = default_agent_record(normalized_did);
    store
        .snapshot
        .agents
        .insert(normalized_did.to_owned(), record.clone());
    (record, true)
}
