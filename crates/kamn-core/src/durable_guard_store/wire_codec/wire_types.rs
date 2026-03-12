use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct DurableGuardSnapshotBundleWire {
    pub schema_version: u16,
    pub delivery_guard: DeliveryGuardSnapshotWire,
    pub channel_policy: ChannelPolicySnapshotWire,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct DeliveryGuardSnapshotWire {
    pub schema_version: u16,
    pub next_nonce_by_sender: Vec<DeliveryNonceWire>,
    pub seen_message_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct DeliveryNonceWire {
    pub sender: String,
    pub nonce: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct ChannelPolicySnapshotWire {
    pub schema_version: u16,
    pub channels: Vec<ChannelPolicySnapshotChannelWire>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct ChannelPolicySnapshotChannelWire {
    pub channel_id: String,
    pub members: Vec<String>,
    pub admins: Vec<String>,
    pub permissions: ChannelPermissionsWire,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct ChannelPermissionsWire {
    pub send: String,
    pub read: String,
    pub invite: String,
    pub remove: String,
    pub configure: String,
    pub retention: String,
}
