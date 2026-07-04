use crate::data_layer_m8_compliance_lifecycle::{
    data_layer_m8_retention_window_seconds, lifecycle::authorize_owner_scope,
    DataLayerM8ComplianceError, DataLayerM8RetentionClass,
    DATA_LAYER_M8_EPHEMERAL_RETENTION_SECONDS, DATA_LAYER_M8_STANDARD_RETENTION_SECONDS,
};

#[test]
fn unit_retention_windows_and_owner_scope_authorization_are_deterministic() {
    assert_eq!(
        data_layer_m8_retention_window_seconds(DataLayerM8RetentionClass::Standard),
        Some(DATA_LAYER_M8_STANDARD_RETENTION_SECONDS)
    );
    assert_eq!(
        data_layer_m8_retention_window_seconds(DataLayerM8RetentionClass::Ephemeral),
        Some(DATA_LAYER_M8_EPHEMERAL_RETENTION_SECONDS)
    );
    assert_eq!(
        data_layer_m8_retention_window_seconds(DataLayerM8RetentionClass::LegalHold),
        None
    );

    assert!(authorize_owner_scope("kamn:did:owner:alpha", "kamn:did:owner:alpha").is_ok());
    assert!(matches!(
        authorize_owner_scope("kamn:did:owner:alpha", "kamn:did:owner:beta"),
        Err(DataLayerM8ComplianceError::OwnerScopeViolation { .. })
    ));
}
