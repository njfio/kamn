use crate::data_layer_m8_compliance_lifecycle::{
    DataLayerM8ComplianceError, DataLayerM8ComplianceRegistry, DataLayerM8CryptoShredRequest,
    DataLayerM8LegalHoldRequest, DataLayerM8OwnerScopeQuery, DataLayerM8RetentionClass,
    DATA_LAYER_M8_CEK_TOMBSTONE_MARKER, DATA_LAYER_M8_CRYPTO_SHRED_REASON_CODE,
    DATA_LAYER_M8_OWNER_SCOPE_DENIED_REASON_CODE, DATA_LAYER_M8_RETENTION_DUE_REASON_CODE,
};

use super::support::{register_message, OWNER_DID};

#[test]
fn regression_m8_crypto_shred_fails_closed_under_legal_hold_then_tombstones_keys() {
    let mut registry = DataLayerM8ComplianceRegistry::new();
    register_message(
        &mut registry,
        "msg-legal-hold",
        1_000,
        DataLayerM8RetentionClass::Standard,
    );

    registry
        .set_legal_hold(DataLayerM8LegalHoldRequest {
            requester_owner_did: OWNER_DID.to_owned(),
            owner_did: OWNER_DID.to_owned(),
            message_id: "msg-legal-hold".to_owned(),
            legal_hold_active: true,
        })
        .expect("legal hold activation should succeed");

    assert_eq!(
        registry.crypto_shred(DataLayerM8CryptoShredRequest {
            requester_owner_did: OWNER_DID.to_owned(),
            owner_did: OWNER_DID.to_owned(),
            message_id: "msg-legal-hold".to_owned(),
            shredded_at_epoch_seconds: 4_000,
        }),
        Err(DataLayerM8ComplianceError::LegalHoldActive {
            message_id: "msg-legal-hold".to_owned()
        })
    );

    registry
        .set_legal_hold(DataLayerM8LegalHoldRequest {
            requester_owner_did: OWNER_DID.to_owned(),
            owner_did: OWNER_DID.to_owned(),
            message_id: "msg-legal-hold".to_owned(),
            legal_hold_active: false,
        })
        .expect("legal hold release should succeed");

    let shredded = registry
        .crypto_shred(DataLayerM8CryptoShredRequest {
            requester_owner_did: OWNER_DID.to_owned(),
            owner_did: OWNER_DID.to_owned(),
            message_id: "msg-legal-hold".to_owned(),
            shredded_at_epoch_seconds: 4_000,
        })
        .expect("crypto shred should succeed after legal hold release");
    assert_eq!(shredded.shredded_at_epoch_seconds, Some(4_000));
    assert_eq!(
        shredded.shred_reason_code,
        Some(DATA_LAYER_M8_CRYPTO_SHRED_REASON_CODE)
    );
    assert_eq!(shredded.wrapped_keys.len(), 1);
    assert_eq!(
        shredded.wrapped_keys[0].wrapped_cek,
        DATA_LAYER_M8_CEK_TOMBSTONE_MARKER
    );

    assert_eq!(
        registry.crypto_shred(DataLayerM8CryptoShredRequest {
            requester_owner_did: OWNER_DID.to_owned(),
            owner_did: OWNER_DID.to_owned(),
            message_id: "msg-legal-hold".to_owned(),
            shredded_at_epoch_seconds: 5_000,
        }),
        Err(DataLayerM8ComplianceError::AlreadyShredded {
            message_id: "msg-legal-hold".to_owned()
        })
    );
}

#[test]
fn unit_m8_due_projection_excludes_held_or_shredded_and_enforces_owner_scope() {
    let mut registry = DataLayerM8ComplianceRegistry::new();
    register_message(
        &mut registry,
        "msg-due",
        1_000,
        DataLayerM8RetentionClass::Standard,
    );
    register_message(
        &mut registry,
        "msg-held",
        1_000,
        DataLayerM8RetentionClass::Standard,
    );
    register_message(
        &mut registry,
        "msg-shredded",
        1_000,
        DataLayerM8RetentionClass::Standard,
    );

    registry
        .set_legal_hold(DataLayerM8LegalHoldRequest {
            requester_owner_did: OWNER_DID.to_owned(),
            owner_did: OWNER_DID.to_owned(),
            message_id: "msg-held".to_owned(),
            legal_hold_active: true,
        })
        .expect("legal hold should be set");
    registry
        .crypto_shred(DataLayerM8CryptoShredRequest {
            requester_owner_did: OWNER_DID.to_owned(),
            owner_did: OWNER_DID.to_owned(),
            message_id: "msg-shredded".to_owned(),
            shredded_at_epoch_seconds: 8_000_001,
        })
        .expect("shred should succeed for shred fixture");

    let due_candidates = registry
        .retention_due_for_owner(
            DataLayerM8OwnerScopeQuery {
                requester_owner_did: OWNER_DID.to_owned(),
                owner_did: OWNER_DID.to_owned(),
            },
            8_000_001,
        )
        .expect("owner-scoped due query should succeed");
    assert_eq!(due_candidates.len(), 1);
    assert_eq!(due_candidates[0].message_id, "msg-due");
    assert_eq!(
        due_candidates[0].reason_code,
        DATA_LAYER_M8_RETENTION_DUE_REASON_CODE
    );

    assert!(matches!(
        registry.retention_due_for_owner(
            DataLayerM8OwnerScopeQuery {
                requester_owner_did: "kamn:did:owner:outsider-6035".to_owned(),
                owner_did: OWNER_DID.to_owned(),
            },
            8_000_001,
        ),
        Err(DataLayerM8ComplianceError::OwnerScopeViolation {
            reason_code: DATA_LAYER_M8_OWNER_SCOPE_DENIED_REASON_CODE,
        })
    ));
}
