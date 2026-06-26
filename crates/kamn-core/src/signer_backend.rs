//! Signer backend contracts for local and secure-provider signing flows.

mod backends;
mod env;
mod errors;
mod provider_policy;
mod request;
mod router;
mod signing_cache;
#[cfg(test)]
mod tests;

pub use backends::{
    deterministic_secure_provider_client_sign, DeterministicSecureSignerProviderClient,
    LocalSignerBackend, SecureSignerBackend, SecureSignerProviderClient,
    SecureSignerProviderClientSignFn, SignerBackend,
};
pub use errors::SignerBackendError;
pub use provider_policy::{
    BackendSignature, CanonicalSecureKeyReference, SecureSignerProvider, SignerKeyRole,
    SignerProviderHandshakeMatrix, SignerProviderHandshakeStatus,
};
pub use request::SigningRequest;
pub use router::SignerBackendRouter;
