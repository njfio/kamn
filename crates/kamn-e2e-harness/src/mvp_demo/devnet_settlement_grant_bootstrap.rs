use std::path::Path;

use kamn_agent_lib::AgentIdentity;

use super::devnet_settlement_service::CREATOR_AGENT_NAME;
use super::report::escape_json;

const GRANT_ID: &str = "mvp-demo-task-create";

pub(super) fn write_creator_grant_bootstrap(
    state_file: &Path,
    run_dir: &Path,
) -> Result<(), String> {
    let identity = AgentIdentity::from_agent_name(CREATOR_AGENT_NAME)
        .map_err(|error| format!("failed to derive MVP creator identity: {error}"))?;
    let grant = grant_json(identity.did().as_str(), GRANT_ID);
    let state = state_json(grant.as_str());
    std::fs::write(state_file, state)
        .map_err(|error| format!("failed to bootstrap MVP creator grant: {error}"))?;
    let proof = proof_json(identity.did().as_str(), grant.as_str());
    std::fs::write(
        run_dir.join("proof/local-operator-grant-bootstrap.json"),
        proof,
    )
    .map_err(|error| format!("failed to write MVP creator grant proof: {error}"))
}

fn state_json(grant: &str) -> String {
    format!(
        "{{\"schema_version\":\"kamn.runtime.service-api-message-store.v4\",\"messages\":{{}},\"channel_messages\":{{}},\"agent_grants\":{{\"{GRANT_ID}\":{grant}}}}}"
    )
}

fn proof_json(did: &str, grant: &str) -> String {
    format!(
        "{{\"schema_version\":\"kamn.mvp.local-operator-grant.v1\",\"claim_scope\":\"local-only\",\"actor_did\":\"{}\",\"grant\":{grant}}}",
        escape_json(did),
    )
}

fn grant_json(did: &str, id: &str) -> String {
    format!(
        "{{\"did\":\"{}\",\"resource\":\"transaction:new\",\"role\":\"initiator\",\"action\":\"task:create\",\"status\":\"active\",\"idempotency_key\":\"{}\"}}",
        escape_json(did), escape_json(id),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_operator_bootstrap_grants_only_task_creation() {
        let json = grant_json("kamn:did:agent:creator-contract", GRANT_ID);

        assert!(json.contains(r#""resource":"transaction:new""#));
        assert!(json.contains(r#""role":"initiator""#));
        assert!(json.contains(r#""action":"task:create""#));
        assert!(!json.contains(r#""resource":"*""#));
        assert!(!json.contains("escrow:release"));
    }
}
