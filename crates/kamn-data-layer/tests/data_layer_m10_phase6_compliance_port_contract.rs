use kamn_data_layer::{
    DataLayerM10ComplianceProjectionMessageState, DataLayerM10Phase6CompliancePort,
    DataLayerM10Phase6CompliancePortError, DataLayerM10Phase6CryptoShredInput,
    DataLayerM10Phase6RetentionDueCandidate,
};

#[derive(Default)]
struct FakePhase6Port;

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
        Ok(vec![DataLayerM10Phase6RetentionDueCandidate {
            message_id: "msg-1".to_owned(),
        }])
    }

    fn crypto_shred(
        &mut self,
        input: DataLayerM10Phase6CryptoShredInput,
    ) -> Result<(), DataLayerM10Phase6CompliancePortError> {
        if input.message_id.is_empty() {
            return Err(DataLayerM10Phase6CompliancePortError::InvalidInput(
                "message_id must not be empty".to_owned(),
            ));
        }
        Ok(())
    }

    fn message_for_owner(
        &self,
        owner_did: &str,
        message_id: &str,
    ) -> Result<DataLayerM10ComplianceProjectionMessageState, DataLayerM10Phase6CompliancePortError>
    {
        if owner_did.is_empty() || message_id.is_empty() {
            return Err(DataLayerM10Phase6CompliancePortError::LookupFailed(
                "owner/message not found".to_owned(),
            ));
        }
        Ok(DataLayerM10ComplianceProjectionMessageState {
            message_id: message_id.to_owned(),
            legal_hold_active: false,
            shredded_at_epoch_seconds: Some(1_700_000_000),
        })
    }
}

#[test]
fn contract_m10_phase6_compliance_port_exposes_required_seam_methods() {
    let mut port = FakePhase6Port;
    let owner = port
        .authorize_owner_scope("kamn:did:owner:alpha", "kamn:did:owner:alpha")
        .expect("owner scope should authorize");
    let due = port
        .retention_due_for_owner(owner.as_str(), 1_700_000_000)
        .expect("due candidates should resolve");
    assert_eq!(due.len(), 1);
    port.crypto_shred(DataLayerM10Phase6CryptoShredInput {
        requester_owner_did: owner.clone(),
        owner_did: owner.clone(),
        message_id: due[0].message_id.clone(),
        shredded_at_epoch_seconds: 1_700_000_100,
    })
    .expect("crypto shred should apply");
    let message = port
        .message_for_owner(owner.as_str(), due[0].message_id.as_str())
        .expect("message state should resolve");
    assert!(message.shredded_at_epoch_seconds.is_some());
}
