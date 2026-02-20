use std::collections::BTreeMap;

use sqlx::Row;

use super::*;

pub(super) fn data_layer_pg_decode_stored_message(
    row: sqlx::postgres::PgRow,
) -> Result<DataLayerPgStoredMessage, DataLayerPgExecutionAdapterError> {
    let envelope_ciphertext_bytes: Vec<u8> =
        row.try_get("envelope_ciphertext").map_err(|error| {
            DataLayerPgExecutionAdapterError::DecodeFailed {
                field: "envelope_ciphertext",
                detail: error.to_string(),
            }
        })?;
    let envelope_ciphertext = String::from_utf8(envelope_ciphertext_bytes).map_err(|error| {
        DataLayerPgExecutionAdapterError::DecodeFailed {
            field: "envelope_ciphertext",
            detail: error.to_string(),
        }
    })?;

    Ok(DataLayerPgStoredMessage {
        message_id: row.try_get("message_id").map_err(|error| {
            DataLayerPgExecutionAdapterError::DecodeFailed {
                field: "message_id",
                detail: error.to_string(),
            }
        })?,
        owner_did: row.try_get("owner_did").map_err(|error| {
            DataLayerPgExecutionAdapterError::DecodeFailed {
                field: "owner_did",
                detail: error.to_string(),
            }
        })?,
        sender_did: row.try_get("sender_did").map_err(|error| {
            DataLayerPgExecutionAdapterError::DecodeFailed {
                field: "sender_did",
                detail: error.to_string(),
            }
        })?,
        recipient_did: row.try_get("recipient_did").map_err(|error| {
            DataLayerPgExecutionAdapterError::DecodeFailed {
                field: "recipient_did",
                detail: error.to_string(),
            }
        })?,
        envelope_ciphertext,
        content_hash_sha256: row.try_get("content_hash_sha256").map_err(|error| {
            DataLayerPgExecutionAdapterError::DecodeFailed {
                field: "content_hash_sha256",
                detail: error.to_string(),
            }
        })?,
        hash_chain_prev: row.try_get("hash_chain_prev").map_err(|error| {
            DataLayerPgExecutionAdapterError::DecodeFailed {
                field: "hash_chain_prev",
                detail: error.to_string(),
            }
        })?,
        retention_class: row.try_get("retention_class").map_err(|error| {
            DataLayerPgExecutionAdapterError::DecodeFailed {
                field: "retention_class",
                detail: error.to_string(),
            }
        })?,
    })
}

pub(super) fn data_layer_pg_encode_blind_indexes_json(
    blind_indexes: &BTreeMap<String, String>,
) -> Result<String, DataLayerPgExecutionAdapterError> {
    if blind_indexes.is_empty() {
        return Ok("{}".to_owned());
    }

    let mut json = String::from("{");
    for (index, (field_name, token)) in blind_indexes.iter().enumerate() {
        if field_name.trim().is_empty() {
            return Err(
                DataLayerPgExecutionAdapterError::InvalidBlindIndexesPayload {
                    field: "blind_index_field_name",
                    detail: "field name must not be empty".to_owned(),
                },
            );
        }
        if token.trim().is_empty() {
            return Err(
                DataLayerPgExecutionAdapterError::InvalidBlindIndexesPayload {
                    field: "blind_index_token",
                    detail: format!("token for field {field_name} must not be empty"),
                },
            );
        }
        if index > 0 {
            json.push(',');
        }
        json.push('"');
        json.push_str(data_layer_pg_escape_json(field_name.as_str()).as_str());
        json.push_str("\":\"");
        json.push_str(data_layer_pg_escape_json(token.as_str()).as_str());
        json.push('"');
    }
    json.push('}');
    Ok(json)
}

fn data_layer_pg_escape_json(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(character),
        }
    }
    escaped
}

pub(super) fn data_layer_pg_decode_blind_index_search_row(
    row: sqlx::postgres::PgRow,
) -> Result<DataLayerPgBlindIndexSearchRow, DataLayerPgExecutionAdapterError> {
    Ok(DataLayerPgBlindIndexSearchRow {
        message_id: row.try_get("message_id").map_err(|error| {
            DataLayerPgExecutionAdapterError::DecodeFailed {
                field: "message_id",
                detail: error.to_string(),
            }
        })?,
        owner_did: row.try_get("owner_did").map_err(|error| {
            DataLayerPgExecutionAdapterError::DecodeFailed {
                field: "owner_did",
                detail: error.to_string(),
            }
        })?,
        sender_did: row.try_get("sender_did").map_err(|error| {
            DataLayerPgExecutionAdapterError::DecodeFailed {
                field: "sender_did",
                detail: error.to_string(),
            }
        })?,
        recipient_did: row.try_get("recipient_did").map_err(|error| {
            DataLayerPgExecutionAdapterError::DecodeFailed {
                field: "recipient_did",
                detail: error.to_string(),
            }
        })?,
        content_hash_sha256: row.try_get("content_hash_sha256").map_err(|error| {
            DataLayerPgExecutionAdapterError::DecodeFailed {
                field: "content_hash_sha256",
                detail: error.to_string(),
            }
        })?,
    })
}
