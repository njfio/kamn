use crate::data_layer_hashing::tagged_sha256;
use crate::data_layer_m3_blind_index_search::{
    canonical_field_name, normalize_blind_index_value, validate_non_empty, DataLayerM3SearchError,
    DATA_LAYER_M3_BLIND_INDEX_NORMALIZATION_PROFILE, DATA_LAYER_M3_HASH_ALGORITHM,
};

/// Derives one deterministic owner-scoped blind-index token.
pub fn data_layer_m3_compute_blind_index(
    blind_index_key_material: &str,
    field_name: &str,
    value: &str,
) -> Result<String, DataLayerM3SearchError> {
    validate_non_empty(blind_index_key_material, "blind_index_key_material")?;
    let field_name = canonical_field_name(field_name)?;
    let value = normalize_blind_index_value(value)?;
    Ok(tagged_digest(
        format!(
            "m3-blind-index|key:{}|field:{}|value:{}|profile:{}",
            blind_index_key_material.trim(),
            field_name,
            value,
            DATA_LAYER_M3_BLIND_INDEX_NORMALIZATION_PROFILE
        )
        .as_str(),
    ))
}

/// Normalizes one value for M3 blind-index derivation.
pub fn data_layer_m3_normalize_blind_index_value(
    value: &str,
) -> Result<String, DataLayerM3SearchError> {
    normalize_blind_index_value(value)
}

fn tagged_digest(value: &str) -> String {
    tagged_sha256(value, DATA_LAYER_M3_HASH_ALGORITHM)
}
