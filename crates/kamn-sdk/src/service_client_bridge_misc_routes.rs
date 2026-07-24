use super::super::{
    expect_status, json_string_array_field, json_string_field, json_u64_field,
    normalize_route_segment, parse_bridge_status, profile_commitment, SdkError,
    ServiceAgentBalance, ServiceAgentProfile, ServiceBridgeStatus, ServiceBridgeSubmission,
    ServiceHealthStatus, ServiceRequestAuth,
};
use super::ServiceApiClient;
use crate::{service_agent_registration_payload, AgentDid, AgentMetadata, AgentQuery};

impl ServiceApiClient {
    /// Submits one bridge message via `POST /v1/bridge/submit`.
    pub fn submit_bridge_message(
        &self,
        payload: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceBridgeSubmission, SdkError> {
        let response = self.request("POST", "/v1/bridge/submit", payload, Some(auth))?;
        expect_status(response.status, 202)?;
        Ok(ServiceBridgeSubmission {
            bridge_id: json_string_field(response.body.as_str(), "bridge_id")?,
            source_message_id: json_string_field(response.body.as_str(), "source_message_id")?,
            bridge_status: json_string_field(response.body.as_str(), "bridge_status")?,
        })
    }

    /// Forwards one submitted bridge message via `POST /v1/bridge/{id}/forward`.
    pub fn forward_bridge_message(
        &self,
        bridge_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceBridgeStatus, SdkError> {
        let bridge_id = normalize_route_segment("bridge_id", bridge_id)?;
        let route = format!("/v1/bridge/{bridge_id}/forward");
        let response = self.request("POST", route.as_str(), "{}", Some(auth))?;
        expect_status(response.status, 200)?;
        parse_bridge_status(response.body.as_str())
    }

    /// Queries one bridge forwarding status via `GET /v1/bridge/{id}`.
    pub fn get_bridge_message(
        &self,
        bridge_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceBridgeStatus, SdkError> {
        let bridge_id = normalize_route_segment("bridge_id", bridge_id)?;
        let route = format!("/v1/bridge/{bridge_id}");
        let response = self.request("GET", route.as_str(), "", Some(auth))?;
        expect_status(response.status, 200)?;
        parse_bridge_status(response.body.as_str())
    }

    /// Queries an agent reputation/profile by DID.
    pub fn get_agent_profile(
        &self,
        did: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceAgentProfile, SdkError> {
        let did = normalize_route_segment("did", did)?;
        AgentDid::parse(did.as_str()).map_err(SdkError::from)?;
        let route = format!("/v1/agents/{did}");
        let response = self.request("GET", route.as_str(), "", Some(auth))?;
        expect_status(response.status, 200)?;
        parse_agent_profile_response(response.body.as_str())
    }

    /// Registers the authenticated sender DID as an agent profile.
    pub fn register_agent(
        &self,
        metadata: &AgentMetadata,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceAgentProfile, SdkError> {
        let payload = service_agent_registration_payload(metadata)?;
        let response = self.request("POST", "/v1/agents/register", payload.as_str(), Some(auth))?;
        expect_status(response.status, 201)?;
        parse_agent_profile_response(response.body.as_str())
    }

    /// Searches registered agent profiles via `POST /v1/agents/search`.
    pub fn search_agents(
        &self,
        query: &AgentQuery,
        auth: &ServiceRequestAuth,
    ) -> Result<Vec<ServiceAgentProfile>, SdkError> {
        let payload = agent_search_payload(query)?;
        let response = self.request("POST", "/v1/agents/search", payload.as_str(), Some(auth))?;
        expect_status(response.status, 200)?;
        parse_agent_profile_list_response(response.body.as_str())
    }

    /// Queries an agent balance by DID.
    pub fn get_agent_balance(
        &self,
        did: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceAgentBalance, SdkError> {
        let did = normalize_route_segment("did", did)?;
        AgentDid::parse(did.as_str()).map_err(SdkError::from)?;
        let route = format!("/v1/agents/{did}/balance");
        let response = self.request("GET", route.as_str(), "", Some(auth))?;
        expect_status(response.status, 200)?;
        Ok(ServiceAgentBalance {
            did: json_string_field(response.body.as_str(), "did")?,
            balance: json_u64_field(response.body.as_str(), "balance")?,
        })
    }

    /// Queries service health route without request auth.
    pub fn health(&self) -> Result<ServiceHealthStatus, SdkError> {
        let response = self.request("GET", "/healthz", "", None)?;
        expect_status(response.status, 200)?;
        Ok(ServiceHealthStatus {
            status: json_string_field(response.body.as_str(), "status")?,
            runtime_mode: json_string_field(response.body.as_str(), "runtime_mode")?,
            role: json_string_field(response.body.as_str(), "role")?,
            observability_source: json_string_field(
                response.body.as_str(),
                "observability_source",
            )?,
            observability_health: json_string_field(
                response.body.as_str(),
                "observability_health",
            )?,
        })
    }

    /// Reads raw prometheus metrics exposition text.
    pub fn metrics(&self) -> Result<String, SdkError> {
        let response = self.request("GET", "/metrics", "", None)?;
        expect_status(response.status, 200)?;
        Ok(response.body)
    }
}

fn parse_agent_profile_response(body: &str) -> Result<ServiceAgentProfile, SdkError> {
    Ok(ServiceAgentProfile {
        did: json_string_field(body, "did")?,
        reputation_score: json_u64_field(body, "reputation_score")?,
        agent_type: json_string_field(body, "agent_type")?,
        model_family: json_string_field(body, "model_family")?,
        capabilities: json_string_array_field(body, "capabilities")?,
        profile_commitment: profile_commitment(body)?,
    })
}

fn parse_agent_profile_list_response(body: &str) -> Result<Vec<ServiceAgentProfile>, SdkError> {
    let rows = serde_json::from_str::<Vec<serde_json::Value>>(body).map_err(|_| {
        SdkError::TransportFailure("service returned invalid search result payload")
    })?;
    rows.into_iter()
        .map(|row| parse_agent_profile_response(row.to_string().as_str()))
        .collect()
}

pub(crate) fn agent_search_payload(query: &AgentQuery) -> Result<String, SdkError> {
    let capability = normalize_optional_search_filter("capability", query.capability.as_deref())?;
    let model_family =
        normalize_optional_search_filter("model_family", query.model_family.as_deref())?;
    Ok(serde_json::json!({
        "capability": capability,
        "model_family": model_family,
    })
    .to_string())
}

fn normalize_optional_search_filter(
    field: &'static str,
    value: Option<&str>,
) -> Result<Option<String>, SdkError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(SdkError::InvalidInput {
            field,
            reason: "must not be empty when provided",
        });
    }
    Ok(Some(normalized.to_owned()))
}
