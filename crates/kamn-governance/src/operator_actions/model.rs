use crate::operator_binding::OperatorBindingAction;

/// Authorization outcome for a requested operator action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperatorActionOutcome {
    /// Action passed authorization and was applied.
    Allowed,
    /// Action failed authorization and was denied.
    Denied,
}

/// Immutable audit record for a permissioned operator action request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorActionAuditRecord {
    /// Agent DID owning the target configuration namespace.
    pub agent_did: String,
    /// Operator DID requesting the action.
    pub operator_did: String,
    /// Action type requested through operator binding policy.
    pub action: OperatorBindingAction,
    /// Action target key/resource.
    pub target: String,
    /// Optional action value payload.
    pub value: Option<String>,
    /// Request timestamp in unix seconds.
    pub requested_at_unix: u64,
    /// Final authorization outcome.
    pub outcome: OperatorActionOutcome,
}
