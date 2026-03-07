use std::collections::BTreeMap;

use kamn_data_layer::{
    data_layer_m10_project_partition_shred_completeness_with_port,
    DataLayerM10ComplianceProjectionBookkeepingError,
    DataLayerM10ComplianceProjectionMessageState, DataLayerM10ComplianceProjectionPort,
    DataLayerM10ComplianceProjectionPortError, DataLayerM10ComplianceShredProjectionRequest,
    DataLayerM10PartitionRecordInput, DataLayerM10PartitionRegistryStateMachine,
    DATA_LAYER_M10_COMPLIANCE_INPUT_INVALID_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_LEGAL_HOLD_ACTIVE_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_LOOKUP_FAILED_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_TRUE_REASON_CODE,
};

struct FakeProjectionPort {
    messages: BTreeMap<String, DataLayerM10ComplianceProjectionMessageState>,
}

impl FakeProjectionPort {
    fn with_messages(messages: Vec<(&str, bool, Option<u64>)>) -> Self {
        let messages = messages
            .into_iter()
            .map(|(message_id, legal_hold_active, shredded_at_epoch_seconds)| {
                (
                    message_id.to_owned(),
                    DataLayerM10ComplianceProjectionMessageState {
                        message_id: message_id.to_owned(),
                        legal_hold_active,
                        shredded_at_epoch_seconds,
                    },
                )
            })
            .collect();
        Self { messages }
    }
}

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
        if owner_did.trim().is_empty() || message_id.trim().is_empty() {
            return Err(DataLayerM10ComplianceProjectionPortError::InvalidInput(
                "owner/message id cannot be empty".to_owned(),
            ));
        }
        self.messages.get(message_id).cloned().ok_or_else(|| {
            DataLayerM10ComplianceProjectionPortError::LookupFailed(message_id.to_owned())
        })
    }
}

#[test]
fn integration_projection_bookkeeping_updates_registry_and_projects_reason_codes() {
    let mut state_machine = DataLayerM10PartitionRegistryStateMachine::new();
    state_machine
        .register_partition(DataLayerM10PartitionRecordInput {
            partition_month_id: 202401,
            all_messages_shredded: false,
        })
        .expect("partition should register");
    let port = FakeProjectionPort::with_messages(vec![
        ("msg-1", false, Some(1_700_000_001)),
        ("msg-2", false, Some(1_700_000_002)),
    ]);

    let report = data_layer_m10_project_partition_shred_completeness_with_port(
        &mut state_machine,
        &port,
        DataLayerM10ComplianceShredProjectionRequest {
            requester_owner_did: "kamn:did:owner:alpha".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
            partition_month_id: 202401,
            partition_message_ids: vec!["msg-1".to_owned(), "msg-2".to_owned()],
        },
    )
    .expect("projection should succeed");

    assert_eq!(
        report.reason_code,
        DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_TRUE_REASON_CODE
    );
    assert_eq!(
        report.projection_reason_code,
        DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE
    );
    assert!(report.all_messages_shredded);
}

#[test]
fn integration_projection_bookkeeping_marks_legal_hold_before_completeness() {
    let mut state_machine = DataLayerM10PartitionRegistryStateMachine::new();
    state_machine
        .register_partition(DataLayerM10PartitionRecordInput {
            partition_month_id: 202401,
            all_messages_shredded: false,
        })
        .expect("partition should register");
    let port = FakeProjectionPort::with_messages(vec![
        ("msg-1", false, Some(1_700_000_001)),
        ("msg-2", true, Some(1_700_000_002)),
    ]);

    let report = data_layer_m10_project_partition_shred_completeness_with_port(
        &mut state_machine,
        &port,
        DataLayerM10ComplianceShredProjectionRequest {
            requester_owner_did: "kamn:did:owner:alpha".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
            partition_month_id: 202401,
            partition_message_ids: vec!["msg-1".to_owned(), "msg-2".to_owned()],
        },
    )
    .expect("projection should succeed");

    assert_eq!(
        report.reason_code,
        DATA_LAYER_M10_COMPLIANCE_LEGAL_HOLD_ACTIVE_REASON_CODE
    );
    assert!(report.all_messages_shredded);
}

#[test]
fn integration_projection_bookkeeping_fails_closed_on_lookup_and_invalid_input() {
    let mut state_machine = DataLayerM10PartitionRegistryStateMachine::new();
    state_machine
        .register_partition(DataLayerM10PartitionRecordInput {
            partition_month_id: 202401,
            all_messages_shredded: false,
        })
        .expect("partition should register");
    let port = FakeProjectionPort::with_messages(vec![("msg-1", false, None)]);

    assert_eq!(
        data_layer_m10_project_partition_shred_completeness_with_port(
            &mut state_machine,
            &port,
            DataLayerM10ComplianceShredProjectionRequest {
                requester_owner_did: "kamn:did:owner:alpha".to_owned(),
                owner_did: "kamn:did:owner:alpha".to_owned(),
                partition_month_id: 202401,
                partition_message_ids: vec!["msg-missing".to_owned()],
            },
        ),
        Err(DataLayerM10ComplianceProjectionBookkeepingError::PortLookupFailed {
            reason_code: DATA_LAYER_M10_COMPLIANCE_LOOKUP_FAILED_REASON_CODE,
            detail: "msg-missing".to_owned(),
        })
    );

    assert_eq!(
        data_layer_m10_project_partition_shred_completeness_with_port(
            &mut state_machine,
            &port,
            DataLayerM10ComplianceShredProjectionRequest {
                requester_owner_did: "kamn:did:owner:alpha".to_owned(),
                owner_did: "kamn:did:owner:alpha".to_owned(),
                partition_month_id: 202401,
                partition_message_ids: vec!["".to_owned()],
            },
        ),
        Err(DataLayerM10ComplianceProjectionBookkeepingError::PortInvalidInput {
            reason_code: DATA_LAYER_M10_COMPLIANCE_INPUT_INVALID_REASON_CODE,
            detail: "owner/message id cannot be empty".to_owned(),
        })
    );
}
