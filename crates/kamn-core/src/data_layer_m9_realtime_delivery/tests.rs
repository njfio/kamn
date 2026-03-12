use super::{
    DataLayerM9RealtimeDeliveryError,
    DATA_LAYER_M9_BACKPRESSURE_ESCROW_EXTENSION_AFTER_SECONDS,
    DATA_LAYER_M9_BACKPRESSURE_WARNING_AFTER_SECONDS,
};
use super::dispatch::outcome::queue_escalation;
use super::validation::{authorize_owner_scope, normalize_pair};

#[test]
fn unit_queue_escalation_thresholds_are_monotonic() {
    assert_eq!(queue_escalation(None, 10_000), (false, false));

    let first_full_at = Some(1_000);
    assert_eq!(
        queue_escalation(
            first_full_at,
            1_000 + DATA_LAYER_M9_BACKPRESSURE_WARNING_AFTER_SECONDS
        ),
        (false, false)
    );
    assert_eq!(
        queue_escalation(
            first_full_at,
            1_000 + DATA_LAYER_M9_BACKPRESSURE_WARNING_AFTER_SECONDS + 1
        ),
        (true, false)
    );
    assert_eq!(
        queue_escalation(
            first_full_at,
            1_000 + DATA_LAYER_M9_BACKPRESSURE_ESCROW_EXTENSION_AFTER_SECONDS + 1
        ),
        (true, true)
    );
}

#[test]
fn unit_normalize_pair_orders_lexicographically_and_is_idempotent() {
    assert_eq!(
        normalize_pair("kamn:did:agent:zeta", "kamn:did:agent:alpha"),
        (
            "kamn:did:agent:alpha".to_owned(),
            "kamn:did:agent:zeta".to_owned()
        )
    );
    assert_eq!(
        normalize_pair("kamn:did:agent:alpha", "kamn:did:agent:alpha"),
        (
            "kamn:did:agent:alpha".to_owned(),
            "kamn:did:agent:alpha".to_owned()
        )
    );
}

#[test]
fn unit_authorize_owner_scope_rejects_non_matching_owner_dids() {
    assert!(authorize_owner_scope("kamn:did:owner:alpha", "kamn:did:owner:alpha").is_ok());
    assert!(matches!(
        authorize_owner_scope("kamn:did:owner:alpha", "kamn:did:owner:beta"),
        Err(DataLayerM9RealtimeDeliveryError::OwnerScopeViolation { .. })
    ));
}
