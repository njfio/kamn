mod context;
mod m0_to_m1;
mod m2_m3;
mod m4_m5;
mod m6_m7;
mod m8_m11;
mod support;

use super::ServiceApiDataLayerRuntimeEvidenceRecord;
use context::{build_runtime_evidence_context, build_runtime_evidence_identities};
use m0_to_m1::build_runtime_evidence_m0_to_m1;
use m2_m3::build_runtime_evidence_m2_to_m5;
use m6_m7::build_runtime_evidence_m6_to_m11;
use support::assemble_runtime_evidence_record;

pub(super) fn build_data_layer_runtime_evidence(
    message_id: &str,
    payload: &str,
    sender_did: Option<&str>,
    recipient_did: Option<&str>,
) -> Result<ServiceApiDataLayerRuntimeEvidenceRecord, String> {
    let context = build_runtime_evidence_context(message_id, payload);
    let identities = build_runtime_evidence_identities(sender_did, recipient_did);
    let m0_to_m1 = build_runtime_evidence_m0_to_m1(&context, &identities)?;
    let m2_to_m5 = build_runtime_evidence_m2_to_m5(&context, &identities)?;
    let m6_to_m11 = build_runtime_evidence_m6_to_m11(&context, &identities)?;
    Ok(assemble_runtime_evidence_record(
        m0_to_m1, m2_to_m5, m6_to_m11,
    ))
}
