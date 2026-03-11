use crate::{AgentDid, DidDocument};
use std::collections::BTreeMap;

mod adapter_traits;
mod core_types;
mod submission_types;

pub(crate) const DID_CHAIN_ADAPTER_SCHEMA_VERSION: &str = "kamn.did.chain-adapter.v1";
pub(crate) type SubmissionReceiptIndex =
    BTreeMap<String, submission_types::DidChainSubmissionReceipt>;
pub(crate) type SubmissionRejectIndex = BTreeMap<String, String>;
pub(crate) type PersistedDidChainAdapterState = (SubmissionReceiptIndex, SubmissionRejectIndex);
pub(crate) type DidMutationSubmissionKey = (AgentDid, u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DidRegistryRecord {
    pub(crate) document: DidDocument,
    pub(crate) revoked: bool,
}

pub use adapter_traits::{DidLifecycleChainAdapter, DidRegistrationChainAdapter};
pub use core_types::{
    DidLifecycleMutationAction, DidLifecycleMutationEvidence, DidLifecycleMutationRequest,
    DidSubmissionFinalityRecord, DidSubmissionFinalityStatus, DidSubmissionRetryClass,
};
pub use submission_types::{
    DidChainSubmissionOutcome, DidChainSubmissionReceipt, DidChainSubmissionRequest,
    DidChainSubmissionResult, DidLifecycleChainSubmissionRequest,
    DidLifecycleChainSubmissionResult,
};
