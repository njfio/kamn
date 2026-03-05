use kamn_data_layer::{
    DataLayerM10ComplianceProjectionMessageState, DataLayerM10ComplianceProjectionPort,
    DataLayerM10ComplianceProjectionPortError,
};

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
        if owner_did.is_empty() || message_id.is_empty() {
            return Err(DataLayerM10ComplianceProjectionPortError::InvalidInput(
                "owner/message id cannot be empty".to_owned(),
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
fn contract_m10_projection_port_exposes_authorize_and_message_lookup_shape() {
    let port = FakeProjectionPort;
    let owner = port
        .authorize_owner_scope("kamn:did:owner:alpha", "kamn:did:owner:alpha")
        .expect("owner scope should authorize");
    assert_eq!(owner, "kamn:did:owner:alpha");
    let message = port
        .message_for_owner(owner.as_str(), "msg-1")
        .expect("message should resolve");
    assert_eq!(message.message_id, "msg-1");
    assert!(message.shredded_at_epoch_seconds.is_some());
}
