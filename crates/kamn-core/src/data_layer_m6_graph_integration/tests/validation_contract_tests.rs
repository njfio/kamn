use super::*;

#[test]
fn unit_validate_weight_rejects_non_finite_and_out_of_range_values() {
    assert_eq!(
        validate_weight(0.0),
        Err(DataLayerM6GraphIntegrationError::InvalidWeight(0.0))
    );
    assert_eq!(
        validate_weight(1.01),
        Err(DataLayerM6GraphIntegrationError::InvalidWeight(1.01))
    );
    assert!(matches!(
        validate_weight(f32::NAN),
        Err(DataLayerM6GraphIntegrationError::InvalidWeight(value)) if value.is_nan()
    ));
    assert!(validate_weight(0.75).is_ok());
}

#[test]
fn unit_resolve_limit_defaults_and_rejects_zero_limit() {
    assert_eq!(resolve_limit(None), Ok(20));
    assert_eq!(resolve_limit(Some(7)), Ok(7));
    assert_eq!(
        resolve_limit(Some(0)),
        Err(DataLayerM6GraphIntegrationError::InvalidLimit(0))
    );
}

#[test]
fn unit_validate_non_empty_rejects_whitespace_only_input() {
    assert_eq!(
        validate_non_empty(" \t", "node_id"),
        Err(DataLayerM6GraphIntegrationError::EmptyField("node_id"))
    );
    assert!(validate_non_empty("node-a", "node_id").is_ok());
}
