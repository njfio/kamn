use super::{
    DidChainSubmissionOutcome, DidChainSubmissionRequest, DidLifecycleChainSubmissionRequest,
};
use crate::did_registry::DidRegistryError;

/// Chain adapter abstraction for DID registration backends.
pub trait DidRegistrationChainAdapter {
    /// Submits a DID registration request via backing provider.
    fn submit_registration(
        &mut self,
        request: &DidChainSubmissionRequest,
    ) -> Result<DidChainSubmissionOutcome, DidRegistryError>;
}

/// Chain adapter abstraction for DID lifecycle mutation backends.
pub trait DidLifecycleChainAdapter {
    /// Submits a DID lifecycle mutation request via backing provider.
    fn submit_lifecycle_mutation(
        &mut self,
        request: &DidLifecycleChainSubmissionRequest,
    ) -> Result<DidChainSubmissionOutcome, DidRegistryError>;
}
