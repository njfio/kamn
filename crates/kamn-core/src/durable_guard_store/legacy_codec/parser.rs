mod channel;
mod delivery;
mod support;

use std::collections::{BTreeMap, BTreeSet};

use super::super::{DurableGuardSnapshotBundle, DurableGuardSnapshotStoreError};
use channel::parse_channel_section;
use delivery::parse_delivery_section;
use support::{finish_bundle, parse_schema_line};

pub(crate) fn deserialize_bundle_legacy(
    payload: &str,
) -> Result<DurableGuardSnapshotBundle, DurableGuardSnapshotStoreError> {
    let mut lines = payload.lines();
    let bundle_schema = parse_schema_line(&mut lines, "bundle_schema|", "bundle_schema")?;
    let delivery_schema = parse_schema_line(&mut lines, "delivery_schema|", "delivery_schema")?;
    let (next_nonce_by_sender, seen_message_ids): (BTreeMap<String, u64>, BTreeSet<String>) =
        parse_delivery_section(&mut lines)?;
    let channel_schema = parse_schema_line(&mut lines, "channel_schema|", "channel_schema")?;
    let channels = parse_channel_section(&mut lines)?;
    finish_bundle(
        &mut lines,
        bundle_schema,
        delivery_schema,
        channel_schema,
        next_nonce_by_sender,
        seen_message_ids,
        channels,
    )
}
