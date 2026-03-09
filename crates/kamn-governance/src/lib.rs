//! Governance workflow and operator-control contracts extracted from `kamn-core`.

pub mod governance_workflow;
pub mod operator_actions;
pub mod operator_binding;

pub use governance_workflow::{
    GovernanceExecutionRecord, GovernanceParameterChangeDraft, GovernanceProposalDraft,
    GovernanceProposalRecord, GovernanceProposalStatus, GovernanceVoteChoice, GovernanceVoteRecord,
    GovernanceWorkflow, GovernanceWorkflowError,
};
pub use operator_actions::{
    OperatorActionAuditRecord, OperatorActionOutcome, OperatorActionServiceError,
    PermissionedOperatorActionService,
};
pub use operator_binding::{
    OperatorBindingAction, OperatorBindingEngine, OperatorBindingError, OperatorBindingProof,
    OperatorBindingRecord,
};
