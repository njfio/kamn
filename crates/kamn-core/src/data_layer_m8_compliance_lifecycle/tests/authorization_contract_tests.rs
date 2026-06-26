use crate::data_layer_m8_compliance_lifecycle::{
    lifecycle::{authorize_owner_scope, validate_wrapped_keys},
    DataLayerM8ComplianceError, DataLayerM8WrappedCekInput,
};

#[test]
fn unit_validate_wrapped_keys_rejects_empty_duplicate_and_blank_wrapped_cek() {
    assert_eq!(
        validate_wrapped_keys(&[]),
        Err(DataLayerM8ComplianceError::EmptyWrappedKeys)
    );

    let duplicate_inputs = vec![
        DataLayerM8WrappedCekInput {
            recipient_did: "kamn:did:agent:bob".to_owned(),
            wrapped_cek: "cek-a".to_owned(),
        },
        DataLayerM8WrappedCekInput {
            recipient_did: "kamn:did:agent:bob".to_owned(),
            wrapped_cek: "cek-b".to_owned(),
        },
    ];
    assert_eq!(
        validate_wrapped_keys(&duplicate_inputs),
        Err(DataLayerM8ComplianceError::DuplicateWrappedKeyRecipient {
            recipient_did: "kamn:did:agent:bob".to_owned()
        })
    );

    let blank_wrapped_key = vec![DataLayerM8WrappedCekInput {
        recipient_did: "kamn:did:agent:alice".to_owned(),
        wrapped_cek: " ".to_owned(),
    }];
    assert_eq!(
        validate_wrapped_keys(&blank_wrapped_key),
        Err(DataLayerM8ComplianceError::InvalidWrappedKey("wrapped_cek"))
    );

    assert!(matches!(
        authorize_owner_scope("kamn:did:owner:outsider-6035", "kamn:did:owner:owner-6035"),
        Err(DataLayerM8ComplianceError::OwnerScopeViolation { .. })
    ));
}
