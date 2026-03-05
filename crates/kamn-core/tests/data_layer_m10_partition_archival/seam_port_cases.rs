use super::*;

const ALPHA_OWNER_DID: &str = "kamn:did:owner:alpha";
const PHASE6_OWNER_DID: &str = "kamn:did:owner:phase6-port";
const PROJECTION_MESSAGE_ID: &str = "m10-port-msg-1";
const PHASE6_MESSAGE_ID: &str = "phase6-port-msg-1";

#[derive(Debug, Clone, Copy)]
struct FakeProjectionPort;

impl DataLayerM10ComplianceProjectionPort for FakeProjectionPort {
    fn authorize_owner_scope(
        &self,
        requester_owner_did: &str,
        owner_did: &str,
    ) -> Result<String, DataLayerM10ComplianceProjectionPortError> {
        if requester_owner_did == owner_did {
            Ok(owner_did.to_owned())
        } else {
            Err(DataLayerM10ComplianceProjectionPortError::OwnerScopeViolation)
        }
    }

    fn message_for_owner(
        &self,
        owner_did: &str,
        message_id: &str,
    ) -> Result<
        DataLayerM10ComplianceProjectionMessageState,
        DataLayerM10ComplianceProjectionPortError,
    > {
        if owner_did != ALPHA_OWNER_DID || message_id != PROJECTION_MESSAGE_ID {
            return Err(DataLayerM10ComplianceProjectionPortError::LookupFailed(
                "message missing".to_owned(),
            ));
        }

        Ok(DataLayerM10ComplianceProjectionMessageState {
            message_id: message_id.to_owned(),
            legal_hold_active: false,
            shredded_at_epoch_seconds: Some(1_708_560_200),
        })
    }
}

#[derive(Debug, Clone)]
struct FakePhase6Port {
    due_message_ids: Vec<String>,
    message_states: BTreeMap<String, DataLayerM10ComplianceProjectionMessageState>,
}

impl DataLayerM10Phase6CompliancePort for FakePhase6Port {
    fn authorize_owner_scope(
        &self,
        requester_owner_did: &str,
        owner_did: &str,
    ) -> Result<String, DataLayerM10Phase6CompliancePortError> {
        if requester_owner_did == owner_did {
            Ok(owner_did.to_owned())
        } else {
            Err(DataLayerM10Phase6CompliancePortError::OwnerScopeViolation)
        }
    }

    fn retention_due_for_owner(
        &self,
        owner_did: &str,
        now_epoch_seconds: u64,
    ) -> Result<Vec<DataLayerM10Phase6RetentionDueCandidate>, DataLayerM10Phase6CompliancePortError>
    {
        if owner_did.is_empty() || now_epoch_seconds == 0 {
            return Err(DataLayerM10Phase6CompliancePortError::InvalidInput(
                "owner/now must be valid".to_owned(),
            ));
        }
        Ok(self
            .due_message_ids
            .iter()
            .cloned()
            .map(|message_id| DataLayerM10Phase6RetentionDueCandidate { message_id })
            .collect())
    }

    fn crypto_shred(
        &mut self,
        input: DataLayerM10Phase6CryptoShredInput,
    ) -> Result<(), DataLayerM10Phase6CompliancePortError> {
        let message = self
            .message_states
            .get_mut(input.message_id.as_str())
            .ok_or_else(|| {
                DataLayerM10Phase6CompliancePortError::LookupFailed(
                    "message missing for shred".to_owned(),
                )
            })?;
        message.shredded_at_epoch_seconds = Some(input.shredded_at_epoch_seconds);
        Ok(())
    }

    fn message_for_owner(
        &self,
        owner_did: &str,
        message_id: &str,
    ) -> Result<DataLayerM10ComplianceProjectionMessageState, DataLayerM10Phase6CompliancePortError>
    {
        if owner_did.is_empty() {
            return Err(DataLayerM10Phase6CompliancePortError::InvalidInput(
                "owner must not be empty".to_owned(),
            ));
        }
        self.message_states.get(message_id).cloned().ok_or_else(|| {
            DataLayerM10Phase6CompliancePortError::LookupFailed("missing".to_owned())
        })
    }
}

pub(super) fn run_spec_c37_partition_shred_projection_with_port_is_supported_without_direct_m8_registry_argument(
) {
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(202401, false))
        .expect("partition should register");

    let projection = m10_registry.project_partition_shred_completeness_with_port(
        &FakeProjectionPort,
        DataLayerM10ComplianceShredProjectionRequest {
            requester_owner_did: ALPHA_OWNER_DID.to_owned(),
            owner_did: ALPHA_OWNER_DID.to_owned(),
            partition_month_id: 202401,
            partition_message_ids: vec![PROJECTION_MESSAGE_ID.to_owned()],
        },
    );
    let projection = projection.expect("projection should pass via seam port");
    assert_eq!(
        projection.projection_reason_code,
        DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE
    );
    assert_eq!(projection.shredded_partition_messages, 1);
    assert!(projection.all_messages_shredded);
}

pub(super) fn run_spec_c38_phase6_orchestration_with_port_supports_seam_without_direct_m8_registry_argument(
) {
    let owner_did = PHASE6_OWNER_DID;
    let mut partition_registry = DataLayerM10PartitionLifecycleRegistry::new();
    partition_registry
        .register_partition(partition_input(202401, false))
        .expect("partition should register");

    let mut message_states = BTreeMap::new();
    message_states.insert(
        PHASE6_MESSAGE_ID.to_owned(),
        DataLayerM10ComplianceProjectionMessageState {
            message_id: PHASE6_MESSAGE_ID.to_owned(),
            legal_hold_active: false,
            shredded_at_epoch_seconds: None,
        },
    );
    let mut phase6_port = FakePhase6Port {
        due_message_ids: vec![PHASE6_MESSAGE_ID.to_owned()],
        message_states,
    };

    let mut partition_message_ids_by_month = BTreeMap::new();
    partition_message_ids_by_month.insert(202401, vec![PHASE6_MESSAGE_ID.to_owned()]);
    let report = data_layer_m10_execute_phase6_orchestration_tick_with_port(
        &mut phase6_port,
        &mut partition_registry,
        phase6_request(owner_did, partition_message_ids_by_month),
    )
    .expect("phase6 seam orchestration should succeed");

    assert_eq!(
        report.reason_code,
        DATA_LAYER_M10_PHASE6_EXECUTION_APPLIED_REASON_CODE
    );
    assert_eq!(report.due_candidate_count, 1);
    assert_eq!(
        report.shredded_message_ids,
        vec![PHASE6_MESSAGE_ID.to_owned()]
    );
    assert_eq!(report.projection_reports.len(), 1);
    assert!(report.projection_reports[0].all_messages_shredded);
}
