//! M2 access-gateway contracts for DID authn/authz, RLS templates, and access auditing.
//!
//! This module models the PRD M2 control surface as deterministic Rust contracts:
//! DID-authenticated session issuance, ABAC message visibility checks, RLS policy
//! template emission, and append-only audit logging with hash-chain verification.

mod audit;
mod auth;
mod authorization;
mod models;
mod rls;
#[cfg(test)]
mod tests;

pub use audit::{
    DataLayerM2AccessAuditInput, DataLayerM2AccessAuditLedger, DataLayerM2AccessAuditRecord,
};
pub use auth::{
    DataLayerM2DidAuthRequest, DataLayerM2DidAuthRequestValidated, DataLayerM2DidSessionService,
    DataLayerM2SessionToken,
};
pub use authorization::{
    DataLayerM2AbacEngine, DataLayerM2ActorRole, DataLayerM2AuthorizationDecision,
    DataLayerM2MessageScope, DataLayerM2MessageScopeValidated,
    DataLayerM2NegativeAuthorizationAuditFixture, DataLayerM2NegativeAuthorizationCase,
    DataLayerM2NegativeAuthorizationMatrixDecision, DataLayerM2NegativeAuthorizationMatrixReport,
};
pub use models::{
    DataLayerM2GatewayError, DATA_LAYER_M2_AUDIT_HASH_CHAIN_GENESIS, DATA_LAYER_M2_HASH_ALGORITHM,
    DATA_LAYER_M2_INVALID_AUDITOR_DID_REASON_CODE,
    DATA_LAYER_M2_INVALID_OWNER_RECIPIENT_DID_REASON_CODE,
    DATA_LAYER_M2_INVALID_OWNER_SENDER_DID_REASON_CODE,
    DATA_LAYER_M2_INVALID_RECIPIENT_DID_REASON_CODE,
    DATA_LAYER_M2_INVALID_REQUESTER_DID_REASON_CODE, DATA_LAYER_M2_INVALID_SENDER_DID_REASON_CODE,
    DATA_LAYER_M2_NEGATIVE_MATRIX_ALL_DENIED_REASON_CODE,
    DATA_LAYER_M2_NEGATIVE_MATRIX_DRIFT_DETECTED_REASON_CODE,
    DATA_LAYER_M2_REASON_ABAC_SCOPE_DENIED, DATA_LAYER_M2_REASON_AGENT_COUNTERPARTY_SCOPE_ALLOWED,
    DATA_LAYER_M2_REASON_ESCROW_AUDITOR_SCOPE_ALLOWED, DATA_LAYER_M2_REASON_OWNER_SCOPE_ALLOWED,
    DATA_LAYER_M2_REQUESTER_DID_SETTING,
};
pub use rls::{data_layer_m2_default_rls_policies, DataLayerM2RlsPolicy};
