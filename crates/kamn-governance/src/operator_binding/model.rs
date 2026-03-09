use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Permissioned actions that an operator binding may authorize.
pub enum OperatorBindingAction {
    /// Allow configuration mutations.
    Configure,
    /// Allow binding revocation.
    Revoke,
    /// Allow audit/history reads.
    ReadHistory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Proof payload that attests operator authorization intent.
pub struct OperatorBindingProof {
    /// Proof type identifier.
    pub type_name: String,
    /// Proof creation timestamp.
    pub created: String,
    /// Verification method reference.
    pub verification_method: String,
    /// Proof signature value.
    pub proof_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Persisted operator binding record.
pub struct OperatorBindingRecord {
    /// Agent DID owner of the binding scope.
    pub agent_did: String,
    /// Operator DID granted permissions.
    pub operator_did: String,
    /// Optional proof object for binding establishment.
    pub proof: Option<OperatorBindingProof>,
    /// Authorized permission set.
    pub permissions: BTreeSet<OperatorBindingAction>,
    /// Revocation flag.
    pub revoked: bool,
}
