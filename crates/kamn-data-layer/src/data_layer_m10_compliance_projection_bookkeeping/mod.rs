//! Deterministic M10 shred-completeness projection bookkeeping behind the
//! core-agnostic compliance projection port seam.

mod error;
mod helpers;
mod projector;
mod types;

pub use error::DataLayerM10ComplianceProjectionBookkeepingError;
pub use projector::data_layer_m10_project_partition_shred_completeness_with_port;
pub use types::{
    DataLayerM10ComplianceShredProjectionReport, DataLayerM10ComplianceShredProjectionRequest,
    DATA_LAYER_M10_COMPLIANCE_INPUT_INVALID_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_LEGAL_HOLD_ACTIVE_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_LOOKUP_FAILED_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_OWNER_SCOPE_DENIED_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_FALSE_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_TRUE_REASON_CODE,
};
