use super::{DataLayerM7TimeseriesError, DATA_LAYER_M7_DAILY_BUCKET_SECONDS, DATA_LAYER_M7_HOURLY_BUCKET_SECONDS};

pub(crate) fn validate_kamn_did(value: &str) -> Result<(), DataLayerM7TimeseriesError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || !trimmed.starts_with("kamn:did:") {
        return Err(DataLayerM7TimeseriesError::InvalidDid(value.to_owned()));
    }
    let segments = trimmed.split(':').collect::<Vec<_>>();
    if segments.len() < 4 || segments.iter().any(|segment| segment.is_empty()) {
        return Err(DataLayerM7TimeseriesError::InvalidDid(value.to_owned()));
    }
    Ok(())
}

pub(crate) fn authorize_owner_scope(
    requester_owner_did: &str,
    owner_did: &str,
    reason_code: &'static str,
) -> Result<(), DataLayerM7TimeseriesError> {
    validate_kamn_did(requester_owner_did)?;
    validate_kamn_did(owner_did)?;
    if requester_owner_did != owner_did {
        return Err(DataLayerM7TimeseriesError::OwnerScopeViolation { reason_code });
    }
    Ok(())
}

pub(crate) fn hourly_bucket(timestamp_epoch_seconds: u64) -> u64 {
    timestamp_epoch_seconds - (timestamp_epoch_seconds % DATA_LAYER_M7_HOURLY_BUCKET_SECONDS)
}

pub(crate) fn daily_bucket(timestamp_epoch_seconds: u64) -> u64 {
    timestamp_epoch_seconds - (timestamp_epoch_seconds % DATA_LAYER_M7_DAILY_BUCKET_SECONDS)
}

pub(crate) fn validate_daily_bucket(
    bucket_day_epoch_seconds: u64,
) -> Result<(), DataLayerM7TimeseriesError> {
    if bucket_day_epoch_seconds == 0
        || !bucket_day_epoch_seconds.is_multiple_of(DATA_LAYER_M7_DAILY_BUCKET_SECONDS)
    {
        return Err(DataLayerM7TimeseriesError::InvalidBucketDayEpochSeconds(
            bucket_day_epoch_seconds,
        ));
    }
    Ok(())
}
