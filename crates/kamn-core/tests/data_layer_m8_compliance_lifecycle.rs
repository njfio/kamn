use kamn_core::{
    data_layer_m8_retention_window_aligned_with_content_lifecycle,
    data_layer_m8_retention_window_seconds, ContentRetentionClass, DataLayerM8ComplianceError,
    DataLayerM8ComplianceRegistry, DataLayerM8CryptoShredRequest, DataLayerM8LegalHoldRequest,
    DataLayerM8MessageRecordInput, DataLayerM8OwnerScopeQuery, DataLayerM8RetentionClass,
    DataLayerM8RetentionInteropError, DataLayerM8WrappedCekInput,
    DATA_LAYER_M8_CEK_TOMBSTONE_MARKER, DATA_LAYER_M8_EPHEMERAL_RETENTION_SECONDS,
    DATA_LAYER_M8_EXTENDED_RETENTION_SECONDS, DATA_LAYER_M8_OWNER_SCOPE_DENIED_REASON_CODE,
    DATA_LAYER_M8_STANDARD_RETENTION_SECONDS,
};

fn message_input(
    owner_did: &str,
    message_id: &str,
    created_at_epoch_seconds: u64,
    retention_class: DataLayerM8RetentionClass,
    retention_extension_seconds: u64,
) -> DataLayerM8MessageRecordInput {
    DataLayerM8MessageRecordInput {
        owner_did: owner_did.to_owned(),
        message_id: message_id.to_owned(),
        created_at_epoch_seconds,
        content_hash: format!("sha256:{message_id}:content"),
        hash_chain_prev: format!("sha256:{message_id}:prev"),
        retention_class,
        retention_extension_seconds,
        wrapped_keys: vec![
            DataLayerM8WrappedCekInput {
                recipient_did: "kamn:did:agent:recipient-a".to_owned(),
                wrapped_cek: format!("wrapped:{message_id}:a"),
            },
            DataLayerM8WrappedCekInput {
                recipient_did: "kamn:did:agent:recipient-b".to_owned(),
                wrapped_cek: format!("wrapped:{message_id}:b"),
            },
        ],
    }
}

#[test]
fn spec_c01_crypto_shred_replaces_wrapped_keys_and_preserves_integrity_markers() {
    let mut registry = DataLayerM8ComplianceRegistry::new();
    let before = registry
        .register_message(message_input(
            "kamn:did:owner:alpha",
            "m8-msg-001",
            1_708_560_100,
            DataLayerM8RetentionClass::Standard,
            0,
        ))
        .expect("message registration should succeed");

    let after = registry
        .crypto_shred(DataLayerM8CryptoShredRequest {
            requester_owner_did: "kamn:did:owner:alpha".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
            message_id: "m8-msg-001".to_owned(),
            shredded_at_epoch_seconds: 1_708_660_100,
        })
        .expect("crypto-shred should succeed");

    assert_eq!(after.content_hash, before.content_hash);
    assert_eq!(after.hash_chain_prev, before.hash_chain_prev);
    assert_eq!(after.shredded_at_epoch_seconds, Some(1_708_660_100));
    assert_eq!(after.wrapped_keys.len(), 1);
    assert_eq!(
        after.wrapped_keys[0].wrapped_cek,
        DATA_LAYER_M8_CEK_TOMBSTONE_MARKER
    );
}

#[test]
fn spec_c02_retention_due_windows_are_deterministic_across_classes_and_extensions() {
    let mut registry = DataLayerM8ComplianceRegistry::new();
    let created_at = 1_708_560_100;
    registry
        .register_message(message_input(
            "kamn:did:owner:alpha",
            "m8-ephemeral-due",
            created_at,
            DataLayerM8RetentionClass::Ephemeral,
            0,
        ))
        .expect("ephemeral message should register");
    registry
        .register_message(message_input(
            "kamn:did:owner:alpha",
            "m8-standard-due",
            created_at,
            DataLayerM8RetentionClass::Standard,
            0,
        ))
        .expect("standard message should register");
    registry
        .register_message(message_input(
            "kamn:did:owner:alpha",
            "m8-standard-extended",
            created_at,
            DataLayerM8RetentionClass::Standard,
            3_600,
        ))
        .expect("extended standard message should register");
    registry
        .register_message(message_input(
            "kamn:did:owner:alpha",
            "m8-permanent",
            created_at,
            DataLayerM8RetentionClass::Permanent,
            0,
        ))
        .expect("permanent message should register");

    let due = registry
        .retention_due_for_owner(
            DataLayerM8OwnerScopeQuery {
                requester_owner_did: "kamn:did:owner:alpha".to_owned(),
                owner_did: "kamn:did:owner:alpha".to_owned(),
            },
            created_at + DATA_LAYER_M8_STANDARD_RETENTION_SECONDS + 1,
        )
        .expect("retention due query should succeed");

    assert_eq!(due.len(), 2);
    assert_eq!(due[0].message_id, "m8-ephemeral-due");
    assert_eq!(due[1].message_id, "m8-standard-due");
}

#[test]
fn spec_c03_legal_hold_blocks_shredding_and_due_candidates_until_released() {
    let mut registry = DataLayerM8ComplianceRegistry::new();
    registry
        .register_message(message_input(
            "kamn:did:owner:alpha",
            "m8-legal-hold",
            1_708_560_100,
            DataLayerM8RetentionClass::LegalHold,
            0,
        ))
        .expect("legal-hold message should register");

    let due = registry
        .retention_due_for_owner(
            DataLayerM8OwnerScopeQuery {
                requester_owner_did: "kamn:did:owner:alpha".to_owned(),
                owner_did: "kamn:did:owner:alpha".to_owned(),
            },
            1_908_560_100,
        )
        .expect("retention due query should succeed");
    assert!(due.is_empty());

    let blocked = registry.crypto_shred(DataLayerM8CryptoShredRequest {
        requester_owner_did: "kamn:did:owner:alpha".to_owned(),
        owner_did: "kamn:did:owner:alpha".to_owned(),
        message_id: "m8-legal-hold".to_owned(),
        shredded_at_epoch_seconds: 1_908_560_101,
    });
    assert!(matches!(
        blocked,
        Err(DataLayerM8ComplianceError::LegalHoldActive { message_id })
        if message_id == "m8-legal-hold"
    ));

    registry
        .set_legal_hold(DataLayerM8LegalHoldRequest {
            requester_owner_did: "kamn:did:owner:alpha".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
            message_id: "m8-legal-hold".to_owned(),
            legal_hold_active: false,
        })
        .expect("legal hold release should succeed");
    registry
        .crypto_shred(DataLayerM8CryptoShredRequest {
            requester_owner_did: "kamn:did:owner:alpha".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
            message_id: "m8-legal-hold".to_owned(),
            shredded_at_epoch_seconds: 1_908_560_102,
        })
        .expect("crypto-shred should succeed after legal hold release");
}

#[test]
fn spec_c04_cross_owner_operations_are_denied_fail_closed() {
    let mut registry = DataLayerM8ComplianceRegistry::new();
    registry
        .register_message(message_input(
            "kamn:did:owner:alpha",
            "m8-alpha-only",
            1_708_560_100,
            DataLayerM8RetentionClass::Standard,
            0,
        ))
        .expect("message registration should succeed");

    let denied_due = registry.retention_due_for_owner(
        DataLayerM8OwnerScopeQuery {
            requester_owner_did: "kamn:did:owner:intruder".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
        },
        1_908_560_100,
    );
    assert!(matches!(
        denied_due,
        Err(DataLayerM8ComplianceError::OwnerScopeViolation {
            reason_code: DATA_LAYER_M8_OWNER_SCOPE_DENIED_REASON_CODE,
        })
    ));

    let denied_shred = registry.crypto_shred(DataLayerM8CryptoShredRequest {
        requester_owner_did: "kamn:did:owner:intruder".to_owned(),
        owner_did: "kamn:did:owner:alpha".to_owned(),
        message_id: "m8-alpha-only".to_owned(),
        shredded_at_epoch_seconds: 1_908_560_101,
    });
    assert!(matches!(
        denied_shred,
        Err(DataLayerM8ComplianceError::OwnerScopeViolation {
            reason_code: DATA_LAYER_M8_OWNER_SCOPE_DENIED_REASON_CODE,
        })
    ));
}

#[test]
fn spec_c05_double_shred_is_rejected_with_stable_error() {
    let mut registry = DataLayerM8ComplianceRegistry::new();
    registry
        .register_message(message_input(
            "kamn:did:owner:alpha",
            "m8-double-shred",
            1_708_560_100,
            DataLayerM8RetentionClass::Standard,
            0,
        ))
        .expect("message registration should succeed");
    registry
        .crypto_shred(DataLayerM8CryptoShredRequest {
            requester_owner_did: "kamn:did:owner:alpha".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
            message_id: "m8-double-shred".to_owned(),
            shredded_at_epoch_seconds: 1_808_560_100,
        })
        .expect("first shred should succeed");

    let duplicate = registry.crypto_shred(DataLayerM8CryptoShredRequest {
        requester_owner_did: "kamn:did:owner:alpha".to_owned(),
        owner_did: "kamn:did:owner:alpha".to_owned(),
        message_id: "m8-double-shred".to_owned(),
        shredded_at_epoch_seconds: 1_808_560_101,
    });
    assert!(matches!(
        duplicate,
        Err(DataLayerM8ComplianceError::AlreadyShredded { message_id })
        if message_id == "m8-double-shred"
    ));
}

#[test]
fn spec_c06_duplicate_wrapped_key_recipient_is_rejected_fail_closed() {
    let mut registry = DataLayerM8ComplianceRegistry::new();
    let mut input = message_input(
        "kamn:did:owner:alpha",
        "m8-duplicate-recipient",
        1_708_560_100,
        DataLayerM8RetentionClass::Standard,
        0,
    );
    input.wrapped_keys = vec![
        DataLayerM8WrappedCekInput {
            recipient_did: "kamn:did:agent:recipient-a".to_owned(),
            wrapped_cek: "wrapped:m8-duplicate-recipient:a".to_owned(),
        },
        DataLayerM8WrappedCekInput {
            recipient_did: "kamn:did:agent:recipient-a".to_owned(),
            wrapped_cek: "wrapped:m8-duplicate-recipient:b".to_owned(),
        },
    ];

    let duplicate = registry.register_message(input);
    assert!(matches!(
        duplicate,
        Err(DataLayerM8ComplianceError::DuplicateWrappedKeyRecipient { recipient_did })
        if recipient_did == "kamn:did:agent:recipient-a"
    ));
}

#[test]
fn spec_c07_m8_bridge_maps_representable_retention_classes() {
    assert_eq!(
        ContentRetentionClass::try_from(DataLayerM8RetentionClass::Extended)
            .expect("extended should map"),
        ContentRetentionClass::Compliance
    );
    assert_eq!(
        DataLayerM8RetentionClass::try_from(ContentRetentionClass::Compliance)
            .expect("compliance should map"),
        DataLayerM8RetentionClass::Extended
    );
}

#[test]
fn spec_c08_m8_bridge_rejects_non_representable_legacy_retention_mapping() {
    let ephemeral = ContentRetentionClass::try_from(DataLayerM8RetentionClass::Ephemeral);
    assert!(matches!(
        ephemeral,
        Err(
            DataLayerM8RetentionInteropError::LegacyRetentionClassUnavailable(
                DataLayerM8RetentionClass::Ephemeral
            )
        )
    ));

    let standard = ContentRetentionClass::try_from(DataLayerM8RetentionClass::Standard);
    assert!(matches!(
        standard,
        Err(
            DataLayerM8RetentionInteropError::LegacyRetentionClassUnavailable(
                DataLayerM8RetentionClass::Standard
            )
        )
    ));

    let legal_hold = ContentRetentionClass::try_from(DataLayerM8RetentionClass::LegalHold);
    assert!(matches!(
        legal_hold,
        Err(
            DataLayerM8RetentionInteropError::LegacyRetentionClassUnavailable(
                DataLayerM8RetentionClass::LegalHold
            )
        )
    ));

    let permanent = ContentRetentionClass::try_from(DataLayerM8RetentionClass::Permanent);
    assert!(matches!(
        permanent,
        Err(
            DataLayerM8RetentionInteropError::LegacyRetentionClassUnavailable(
                DataLayerM8RetentionClass::Permanent
            )
        )
    ));

    let short_lived = DataLayerM8RetentionClass::try_from(ContentRetentionClass::ShortLived);
    assert!(matches!(
        short_lived,
        Err(
            DataLayerM8RetentionInteropError::M8RetentionClassUnavailable(
                ContentRetentionClass::ShortLived
            )
        )
    ));

    let standard_legacy = DataLayerM8RetentionClass::try_from(ContentRetentionClass::Standard);
    assert!(matches!(
        standard_legacy,
        Err(
            DataLayerM8RetentionInteropError::M8RetentionClassUnavailable(
                ContentRetentionClass::Standard
            )
        )
    ));
}

#[test]
fn spec_c09_m8_retention_windows_align_with_content_lifecycle_profiles() {
    assert_eq!(
        data_layer_m8_retention_window_seconds(DataLayerM8RetentionClass::Ephemeral),
        Some(DATA_LAYER_M8_EPHEMERAL_RETENTION_SECONDS)
    );
    assert_eq!(
        data_layer_m8_retention_window_seconds(DataLayerM8RetentionClass::Standard),
        Some(DATA_LAYER_M8_STANDARD_RETENTION_SECONDS)
    );
    assert_eq!(
        data_layer_m8_retention_window_seconds(DataLayerM8RetentionClass::Extended),
        Some(DATA_LAYER_M8_EXTENDED_RETENTION_SECONDS)
    );

    assert_eq!(
        data_layer_m8_retention_window_aligned_with_content_lifecycle(
            DataLayerM8RetentionClass::Ephemeral
        ),
        Some(false)
    );
    assert_eq!(
        data_layer_m8_retention_window_aligned_with_content_lifecycle(
            DataLayerM8RetentionClass::Standard
        ),
        Some(false)
    );
    assert_eq!(
        data_layer_m8_retention_window_aligned_with_content_lifecycle(
            DataLayerM8RetentionClass::Extended
        ),
        Some(true)
    );
    assert_eq!(
        data_layer_m8_retention_window_aligned_with_content_lifecycle(
            DataLayerM8RetentionClass::LegalHold
        ),
        None
    );
    assert_eq!(
        data_layer_m8_retention_window_aligned_with_content_lifecycle(
            DataLayerM8RetentionClass::Permanent
        ),
        None
    );
}

#[test]
fn spec_c11_message_lookup_accepts_canonical_equivalent_owner_did() {
    let mut registry = DataLayerM8ComplianceRegistry::new();
    registry
        .register_message(message_input(
            "kamn:did:owner:alpha",
            "m8-canonical-lookup",
            1_708_560_100,
            DataLayerM8RetentionClass::Standard,
            0,
        ))
        .expect("message registration should succeed");

    let lookup = registry.message_for_owner("  kamn:did:owner:alpha  ", "m8-canonical-lookup");
    assert!(
        lookup.is_ok(),
        "canonical-equivalent owner DID should resolve owner-scoped message lookup"
    );
}

#[test]
fn spec_c12_owner_scope_accepts_canonical_equivalent_requester_owner_did() {
    let mut registry = DataLayerM8ComplianceRegistry::new();
    let created_at = 1_708_560_100;
    registry
        .register_message(message_input(
            "kamn:did:owner:alpha",
            "m8-canonical-owner-scope",
            created_at,
            DataLayerM8RetentionClass::Standard,
            0,
        ))
        .expect("message registration should succeed");

    let due = registry.retention_due_for_owner(
        DataLayerM8OwnerScopeQuery {
            requester_owner_did: "  kamn:did:owner:alpha  ".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
        },
        created_at + DATA_LAYER_M8_STANDARD_RETENTION_SECONDS + 1,
    );
    assert!(
        due.is_ok(),
        "canonical-equivalent requester owner DID should authorize owner scope"
    );
}
