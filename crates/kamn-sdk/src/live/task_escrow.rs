mod aliases;
mod payloads;

use crate::AgentDid;

#[derive(Debug, Clone)]
pub(crate) struct LiveTaskAlias {
    pub(crate) service_id: String,
    pub(crate) creator: AgentDid,
    pub(crate) assignee: Option<AgentDid>,
}

#[derive(Debug, Clone)]
pub(crate) struct LiveEscrowAlias {
    pub(crate) service_id: String,
    pub(crate) payer: AgentDid,
}

pub(crate) use self::aliases::{
    deterministic_u64_tag, prepare_escrow_release, prepare_task_accept,
    prepare_task_artifact_submission, prepare_task_complete, remember_artifact_alias,
    remember_escrow_alias, remember_task_alias,
};
pub(crate) use self::payloads::{artifact_payload, escape_json, escrow_payload, task_payload};
