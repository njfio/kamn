//! DID registry lifecycle, idempotent submission, and finality tracking contracts.

mod chain_submission;
mod lifecycle;
mod models;
mod store;
#[cfg(test)]
mod tests;
mod validation;

pub use chain_submission::{
    FileDidRegistrationChainAdapter, InMemoryDidRegistrationChainAdapter,
    KolmeDidLifecycleChainAdapter,
};
pub use models::{
    DidChainSubmissionOutcome, DidChainSubmissionReceipt, DidChainSubmissionRequest,
    DidChainSubmissionResult, DidLifecycleChainAdapter, DidLifecycleChainSubmissionRequest,
    DidLifecycleChainSubmissionResult, DidLifecycleMutationAction, DidLifecycleMutationEvidence,
    DidLifecycleMutationRequest, DidRegistrationChainAdapter, DidSubmissionFinalityRecord,
    DidSubmissionFinalityStatus, DidSubmissionRetryClass,
};
pub use store::DidRegistry;
pub use validation::DidRegistryError;
