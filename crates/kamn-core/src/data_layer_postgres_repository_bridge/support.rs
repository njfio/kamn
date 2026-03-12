use super::DataLayerPgRepositoryBridgeError;

pub(crate) fn validate_non_empty(
    value: &str,
    field: &'static str,
) -> Result<(), DataLayerPgRepositoryBridgeError> {
    if value.trim().is_empty() {
        return Err(DataLayerPgRepositoryBridgeError::EmptyField(field));
    }
    Ok(())
}
