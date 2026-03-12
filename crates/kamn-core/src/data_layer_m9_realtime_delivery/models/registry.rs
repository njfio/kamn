use std::collections::{BTreeMap, BTreeSet};

use super::types::DataLayerM9PresenceRecord;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct DataLayerM9RecipientQueueState {
    pub(in crate::data_layer_m9_realtime_delivery) pending_message_ids: Vec<String>,
    pub(in crate::data_layer_m9_realtime_delivery) deferred_message_ids: Vec<String>,
    pub(in crate::data_layer_m9_realtime_delivery) first_full_at_epoch_seconds: Option<u64>,
}

/// M9 realtime delivery registry.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerM9RealtimeDeliveryRegistry {
    pub(in crate::data_layer_m9_realtime_delivery) presence_by_agent:
        BTreeMap<String, DataLayerM9PresenceRecord>,
    pub(in crate::data_layer_m9_realtime_delivery) queue_by_recipient:
        BTreeMap<String, DataLayerM9RecipientQueueState>,
    pub(in crate::data_layer_m9_realtime_delivery) interaction_pairs: BTreeSet<(String, String)>,
    pub(in crate::data_layer_m9_realtime_delivery) shared_escrow_pairs: BTreeSet<(String, String)>,
}

impl DataLayerM9RealtimeDeliveryRegistry {
    /// Creates an empty realtime delivery registry.
    pub fn new() -> Self {
        Self::default()
    }
}
