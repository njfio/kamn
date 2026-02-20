use super::*;

pub(super) fn data_layer_pg_validate_non_empty_text(
    value: &str,
    field: &'static str,
) -> Result<(), DataLayerPgExecutionAdapterError> {
    if value.trim().is_empty() {
        return Err(
            DataLayerPgExecutionAdapterError::InvalidMerkleBatchPayload {
                field,
                detail: "must not be empty".to_owned(),
            },
        );
    }
    Ok(())
}

pub(super) fn data_layer_pg_validate_positive_unix_timestamp(
    value: i64,
    field: &'static str,
) -> Result<(), DataLayerPgExecutionAdapterError> {
    if value <= 0 {
        return Err(
            DataLayerPgExecutionAdapterError::InvalidMerkleBatchPayload {
                field,
                detail: "must be greater than zero".to_owned(),
            },
        );
    }
    Ok(())
}

pub(super) fn data_layer_pg_validate_uuid_text(
    value: &str,
    field: &'static str,
) -> Result<(), DataLayerPgExecutionAdapterError> {
    if data_layer_pg_is_uuid_text(value) {
        return Ok(());
    }
    Err(
        DataLayerPgExecutionAdapterError::InvalidMerkleBatchPayload {
            field,
            detail: format!(
                "{} ({})",
                "must be a canonical UUID string",
                DATA_LAYER_PG_EXECUTION_MERKLE_BATCH_PAYLOAD_FAILED_REASON_CODE
            ),
        },
    )
}

fn data_layer_pg_is_uuid_text(value: &str) -> bool {
    if value.len() != 36 {
        return false;
    }
    value.chars().enumerate().all(|(index, character)| {
        if matches!(index, 8 | 13 | 18 | 23) {
            character == '-'
        } else {
            character.is_ascii_hexdigit()
        }
    })
}
