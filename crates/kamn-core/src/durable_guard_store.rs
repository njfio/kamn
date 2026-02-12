//! Durable snapshot store contracts for delivery guard and channel policy state.

use crate::{
    ChannelPermissionEngine, ChannelPermissions, ChannelPolicySnapshot,
    ChannelPolicySnapshotChannel, ChannelPolicySnapshotError, DeliveryGuardSnapshot,
    DeliveryGuardSnapshotError, MessageDeliveryGuards, PermissionRule, RetentionPolicy,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

/// Schema version for serialized durable guard bundles.
pub const DURABLE_GUARD_BUNDLE_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Combined snapshot bundle for delivery guards and channel policy state.
pub struct DurableGuardSnapshotBundle {
    /// Bundle schema version.
    pub schema_version: u16,
    /// Delivery guard snapshot payload.
    pub delivery_guard: DeliveryGuardSnapshot,
    /// Channel policy snapshot payload.
    pub channel_policy: ChannelPolicySnapshot,
}

impl DurableGuardSnapshotBundle {
    /// Captures a snapshot bundle from live guard engines.
    pub fn capture(
        delivery_guards: &MessageDeliveryGuards,
        channel_permissions: &ChannelPermissionEngine,
    ) -> Self {
        Self {
            schema_version: DURABLE_GUARD_BUNDLE_SCHEMA_VERSION,
            delivery_guard: delivery_guards.export_snapshot(),
            channel_policy: channel_permissions.export_snapshot(),
        }
    }

    /// Restores snapshot bundle into live guard engines.
    pub fn restore_into(
        self,
        delivery_guards: &mut MessageDeliveryGuards,
        channel_permissions: &mut ChannelPermissionEngine,
    ) -> Result<(), DurableGuardSnapshotStoreError> {
        if self.schema_version != DURABLE_GUARD_BUNDLE_SCHEMA_VERSION {
            return Err(
                DurableGuardSnapshotStoreError::BundleSchemaVersionMismatch {
                    expected: DURABLE_GUARD_BUNDLE_SCHEMA_VERSION,
                    found: self.schema_version,
                },
            );
        }
        delivery_guards
            .restore_snapshot(self.delivery_guard)
            .map_err(DurableGuardSnapshotStoreError::DeliverySnapshot)?;
        channel_permissions
            .restore_snapshot(self.channel_policy)
            .map_err(DurableGuardSnapshotStoreError::ChannelPolicySnapshot)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Durable snapshot store error taxonomy.
pub enum DurableGuardSnapshotStoreError {
    /// Bundle schema version does not match current runtime schema.
    BundleSchemaVersionMismatch {
        /// Expected schema version.
        expected: u16,
        /// Found schema version in payload.
        found: u16,
    },
    /// I/O failure while reading or writing snapshot payload.
    Io(String),
    /// Payload failed structural or semantic validation.
    InvalidPayload(String),
    /// Delivery snapshot payload could not be restored.
    DeliverySnapshot(DeliveryGuardSnapshotError),
    /// Channel-policy snapshot payload could not be restored.
    ChannelPolicySnapshot(ChannelPolicySnapshotError),
}

impl fmt::Display for DurableGuardSnapshotStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BundleSchemaVersionMismatch { expected, found } => write!(
                f,
                "durable guard bundle schema version mismatch, expected {expected}, found {found}"
            ),
            Self::Io(value) => write!(f, "durable guard snapshot store I/O error: {value}"),
            Self::InvalidPayload(value) => {
                write!(f, "durable guard snapshot store invalid payload: {value}")
            }
            Self::DeliverySnapshot(error) => write!(f, "{error}"),
            Self::ChannelPolicySnapshot(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for DurableGuardSnapshotStoreError {}

impl From<DeliveryGuardSnapshotError> for DurableGuardSnapshotStoreError {
    fn from(value: DeliveryGuardSnapshotError) -> Self {
        Self::DeliverySnapshot(value)
    }
}

impl From<ChannelPolicySnapshotError> for DurableGuardSnapshotStoreError {
    fn from(value: ChannelPolicySnapshotError) -> Self {
        Self::ChannelPolicySnapshot(value)
    }
}

/// Store interface for whole durable guard bundles.
pub trait DurableGuardBundleSnapshotStore {
    /// Saves a durable guard bundle.
    fn save_bundle(
        &mut self,
        bundle: DurableGuardSnapshotBundle,
    ) -> Result<(), DurableGuardSnapshotStoreError>;

    /// Loads a durable guard bundle, if present.
    fn load_bundle(
        &self,
    ) -> Result<Option<DurableGuardSnapshotBundle>, DurableGuardSnapshotStoreError>;
}

/// Store interface for delivery-guard snapshot lane.
pub trait DeliveryGuardSnapshotStore {
    /// Saves only delivery-guard snapshot payload.
    fn save_delivery_guard(
        &mut self,
        snapshot: DeliveryGuardSnapshot,
    ) -> Result<(), DurableGuardSnapshotStoreError>;

    /// Loads only delivery-guard snapshot payload, if present.
    fn load_delivery_guard(
        &self,
    ) -> Result<Option<DeliveryGuardSnapshot>, DurableGuardSnapshotStoreError>;
}

/// Store interface for channel-policy snapshot lane.
pub trait ChannelPolicySnapshotStore {
    /// Saves only channel-policy snapshot payload.
    fn save_channel_policy(
        &mut self,
        snapshot: ChannelPolicySnapshot,
    ) -> Result<(), DurableGuardSnapshotStoreError>;

    /// Loads only channel-policy snapshot payload, if present.
    fn load_channel_policy(
        &self,
    ) -> Result<Option<ChannelPolicySnapshot>, DurableGuardSnapshotStoreError>;
}

#[derive(Debug, Clone, Default)]
/// In-memory durable snapshot store implementation for tests and local runtime.
pub struct InMemoryDurableGuardSnapshotStore {
    bundle: Option<DurableGuardSnapshotBundle>,
}

impl DurableGuardBundleSnapshotStore for InMemoryDurableGuardSnapshotStore {
    fn save_bundle(
        &mut self,
        bundle: DurableGuardSnapshotBundle,
    ) -> Result<(), DurableGuardSnapshotStoreError> {
        validate_bundle(&bundle)?;
        self.bundle = Some(bundle);
        Ok(())
    }

    fn load_bundle(
        &self,
    ) -> Result<Option<DurableGuardSnapshotBundle>, DurableGuardSnapshotStoreError> {
        if let Some(bundle) = &self.bundle {
            validate_bundle(bundle)?;
        }
        Ok(self.bundle.clone())
    }
}

impl DeliveryGuardSnapshotStore for InMemoryDurableGuardSnapshotStore {
    fn save_delivery_guard(
        &mut self,
        snapshot: DeliveryGuardSnapshot,
    ) -> Result<(), DurableGuardSnapshotStoreError> {
        let mut bundle = self.bundle.clone().unwrap_or_else(default_bundle);
        bundle.delivery_guard = snapshot;
        self.save_bundle(bundle)
    }

    fn load_delivery_guard(
        &self,
    ) -> Result<Option<DeliveryGuardSnapshot>, DurableGuardSnapshotStoreError> {
        Ok(self.load_bundle()?.map(|bundle| bundle.delivery_guard))
    }
}

impl ChannelPolicySnapshotStore for InMemoryDurableGuardSnapshotStore {
    fn save_channel_policy(
        &mut self,
        snapshot: ChannelPolicySnapshot,
    ) -> Result<(), DurableGuardSnapshotStoreError> {
        let mut bundle = self.bundle.clone().unwrap_or_else(default_bundle);
        bundle.channel_policy = snapshot;
        self.save_bundle(bundle)
    }

    fn load_channel_policy(
        &self,
    ) -> Result<Option<ChannelPolicySnapshot>, DurableGuardSnapshotStoreError> {
        Ok(self.load_bundle()?.map(|bundle| bundle.channel_policy))
    }
}

#[derive(Debug, Clone)]
/// File-backed durable snapshot store implementation.
pub struct FileDurableGuardSnapshotStore {
    path: PathBuf,
}

impl FileDurableGuardSnapshotStore {
    /// Creates a file-backed snapshot store rooted at `path`.
    pub fn new(path: PathBuf) -> Result<Self, DurableGuardSnapshotStoreError> {
        if path.is_dir() {
            return Err(DurableGuardSnapshotStoreError::InvalidPayload(
                "snapshot file path must not be a directory".to_owned(),
            ));
        }
        Ok(Self { path })
    }

    fn load_or_default_bundle(
        &self,
    ) -> Result<DurableGuardSnapshotBundle, DurableGuardSnapshotStoreError> {
        Ok(self.load_bundle()?.unwrap_or_else(default_bundle))
    }
}

impl DurableGuardBundleSnapshotStore for FileDurableGuardSnapshotStore {
    fn save_bundle(
        &mut self,
        bundle: DurableGuardSnapshotBundle,
    ) -> Result<(), DurableGuardSnapshotStoreError> {
        validate_bundle(&bundle)?;
        let payload = serialize_bundle(&bundle);
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&self.path)
            .map_err(|error| DurableGuardSnapshotStoreError::Io(error.to_string()))?;
        file.write_all(payload.as_bytes())
            .map_err(|error| DurableGuardSnapshotStoreError::Io(error.to_string()))
    }

    fn load_bundle(
        &self,
    ) -> Result<Option<DurableGuardSnapshotBundle>, DurableGuardSnapshotStoreError> {
        if !self.path.exists() {
            return Ok(None);
        }
        let payload = fs::read_to_string(&self.path)
            .map_err(|error| DurableGuardSnapshotStoreError::Io(error.to_string()))?;
        if payload.trim().is_empty() {
            return Ok(None);
        }
        let bundle = deserialize_bundle(&payload)?;
        Ok(Some(bundle))
    }
}

impl DeliveryGuardSnapshotStore for FileDurableGuardSnapshotStore {
    fn save_delivery_guard(
        &mut self,
        snapshot: DeliveryGuardSnapshot,
    ) -> Result<(), DurableGuardSnapshotStoreError> {
        let mut bundle = self.load_or_default_bundle()?;
        bundle.delivery_guard = snapshot;
        self.save_bundle(bundle)
    }

    fn load_delivery_guard(
        &self,
    ) -> Result<Option<DeliveryGuardSnapshot>, DurableGuardSnapshotStoreError> {
        Ok(self.load_bundle()?.map(|bundle| bundle.delivery_guard))
    }
}

impl ChannelPolicySnapshotStore for FileDurableGuardSnapshotStore {
    fn save_channel_policy(
        &mut self,
        snapshot: ChannelPolicySnapshot,
    ) -> Result<(), DurableGuardSnapshotStoreError> {
        let mut bundle = self.load_or_default_bundle()?;
        bundle.channel_policy = snapshot;
        self.save_bundle(bundle)
    }

    fn load_channel_policy(
        &self,
    ) -> Result<Option<ChannelPolicySnapshot>, DurableGuardSnapshotStoreError> {
        Ok(self.load_bundle()?.map(|bundle| bundle.channel_policy))
    }
}

fn default_bundle() -> DurableGuardSnapshotBundle {
    DurableGuardSnapshotBundle::capture(
        &MessageDeliveryGuards::new(),
        &ChannelPermissionEngine::new(),
    )
}

fn validate_bundle(
    bundle: &DurableGuardSnapshotBundle,
) -> Result<(), DurableGuardSnapshotStoreError> {
    if bundle.schema_version != DURABLE_GUARD_BUNDLE_SCHEMA_VERSION {
        return Err(
            DurableGuardSnapshotStoreError::BundleSchemaVersionMismatch {
                expected: DURABLE_GUARD_BUNDLE_SCHEMA_VERSION,
                found: bundle.schema_version,
            },
        );
    }
    MessageDeliveryGuards::from_snapshot(bundle.delivery_guard.clone())
        .map_err(DurableGuardSnapshotStoreError::DeliverySnapshot)?;
    ChannelPermissionEngine::from_snapshot(bundle.channel_policy.clone())
        .map_err(DurableGuardSnapshotStoreError::ChannelPolicySnapshot)?;
    Ok(())
}

fn serialize_bundle(bundle: &DurableGuardSnapshotBundle) -> String {
    let mut lines = vec![
        format!("bundle_schema|{}", bundle.schema_version),
        format!("delivery_schema|{}", bundle.delivery_guard.schema_version),
    ];

    for (sender, nonce) in &bundle.delivery_guard.next_nonce_by_sender {
        lines.push(format!("delivery_nonce|{}|{nonce}", encode_hex(sender)));
    }
    for message_id in &bundle.delivery_guard.seen_message_ids {
        lines.push(format!("delivery_seen|{}", encode_hex(message_id)));
    }
    lines.push("delivery_end|".to_owned());

    lines.push(format!(
        "channel_schema|{}",
        bundle.channel_policy.schema_version
    ));
    for channel in &bundle.channel_policy.channels {
        lines.push(format!("channel_begin|{}", encode_hex(&channel.channel_id)));
        for member in &channel.members {
            lines.push(format!("channel_member|{}", encode_hex(member)));
        }
        for admin in &channel.admins {
            lines.push(format!("channel_admin|{}", encode_hex(admin)));
        }
        lines.push(format!(
            "channel_perm_send|{}",
            encode_permission_rule(&channel.permissions.send)
        ));
        lines.push(format!(
            "channel_perm_read|{}",
            encode_permission_rule(&channel.permissions.read)
        ));
        lines.push(format!(
            "channel_perm_invite|{}",
            encode_permission_rule(&channel.permissions.invite)
        ));
        lines.push(format!(
            "channel_perm_remove|{}",
            encode_permission_rule(&channel.permissions.remove)
        ));
        lines.push(format!(
            "channel_perm_configure|{}",
            encode_permission_rule(&channel.permissions.configure)
        ));
        lines.push(format!(
            "channel_retention|{}",
            encode_retention_policy(&channel.permissions.retention)
        ));
        lines.push("channel_end|".to_owned());
    }
    lines.push("channel_end_all|".to_owned());
    lines.push("bundle_end|".to_owned());

    lines.join("\n")
}

fn deserialize_bundle(
    payload: &str,
) -> Result<DurableGuardSnapshotBundle, DurableGuardSnapshotStoreError> {
    let mut lines = payload.lines();

    let bundle_schema = parse_schema_line(&mut lines, "bundle_schema|", "bundle_schema")?;
    let delivery_schema = parse_schema_line(&mut lines, "delivery_schema|", "delivery_schema")?;

    let mut next_nonce_by_sender = BTreeMap::new();
    let mut seen_message_ids = BTreeSet::new();
    loop {
        let line = lines.next().ok_or_else(|| {
            DurableGuardSnapshotStoreError::InvalidPayload("missing delivery_end marker".to_owned())
        })?;
        if line == "delivery_end|" {
            break;
        }
        if let Some(value) = line.strip_prefix("delivery_nonce|") {
            let mut parts = value.splitn(2, '|');
            let sender_hex = parts
                .next()
                .ok_or_else(|| DurableGuardSnapshotStoreError::InvalidPayload(line.to_owned()))?;
            let nonce_raw = parts
                .next()
                .ok_or_else(|| DurableGuardSnapshotStoreError::InvalidPayload(line.to_owned()))?;
            let sender = decode_hex(sender_hex)?;
            let nonce = nonce_raw
                .parse::<u64>()
                .map_err(|_| DurableGuardSnapshotStoreError::InvalidPayload(line.to_owned()))?;
            if next_nonce_by_sender.insert(sender, nonce).is_some() {
                return Err(DurableGuardSnapshotStoreError::InvalidPayload(
                    line.to_owned(),
                ));
            }
            continue;
        }
        if let Some(value) = line.strip_prefix("delivery_seen|") {
            let message_id = decode_hex(value)?;
            if !seen_message_ids.insert(message_id) {
                return Err(DurableGuardSnapshotStoreError::InvalidPayload(
                    line.to_owned(),
                ));
            }
            continue;
        }
        return Err(DurableGuardSnapshotStoreError::InvalidPayload(
            line.to_owned(),
        ));
    }

    let channel_schema = parse_schema_line(&mut lines, "channel_schema|", "channel_schema")?;
    let mut channels = Vec::new();
    let mut builder: Option<ChannelPolicyBuilder> = None;
    loop {
        let line = lines.next().ok_or_else(|| {
            DurableGuardSnapshotStoreError::InvalidPayload(
                "missing channel_end_all marker".to_owned(),
            )
        })?;

        if line == "channel_end_all|" {
            if builder.is_some() {
                return Err(DurableGuardSnapshotStoreError::InvalidPayload(
                    "unterminated channel block".to_owned(),
                ));
            }
            break;
        }
        if let Some(value) = line.strip_prefix("channel_begin|") {
            if builder.is_some() {
                return Err(DurableGuardSnapshotStoreError::InvalidPayload(
                    "nested channel_begin marker".to_owned(),
                ));
            }
            builder = Some(ChannelPolicyBuilder::new(decode_hex(value)?));
            continue;
        }
        if line == "channel_end|" {
            let value = builder.take().ok_or_else(|| {
                DurableGuardSnapshotStoreError::InvalidPayload(
                    "channel_end marker without channel_begin".to_owned(),
                )
            })?;
            channels.push(value.build()?);
            continue;
        }

        let active = builder.as_mut().ok_or_else(|| {
            DurableGuardSnapshotStoreError::InvalidPayload(
                "channel field found without channel_begin".to_owned(),
            )
        })?;

        if let Some(value) = line.strip_prefix("channel_member|") {
            active.members.push(decode_hex(value)?);
            continue;
        }
        if let Some(value) = line.strip_prefix("channel_admin|") {
            active.admins.push(decode_hex(value)?);
            continue;
        }
        if let Some(value) = line.strip_prefix("channel_perm_send|") {
            active.set_send(decode_permission_rule(value)?)?;
            continue;
        }
        if let Some(value) = line.strip_prefix("channel_perm_read|") {
            active.set_read(decode_permission_rule(value)?)?;
            continue;
        }
        if let Some(value) = line.strip_prefix("channel_perm_invite|") {
            active.set_invite(decode_permission_rule(value)?)?;
            continue;
        }
        if let Some(value) = line.strip_prefix("channel_perm_remove|") {
            active.set_remove(decode_permission_rule(value)?)?;
            continue;
        }
        if let Some(value) = line.strip_prefix("channel_perm_configure|") {
            active.set_configure(decode_permission_rule(value)?)?;
            continue;
        }
        if let Some(value) = line.strip_prefix("channel_retention|") {
            active.set_retention(decode_retention_policy(value)?)?;
            continue;
        }

        return Err(DurableGuardSnapshotStoreError::InvalidPayload(
            line.to_owned(),
        ));
    }

    let bundle_end = lines.next().ok_or_else(|| {
        DurableGuardSnapshotStoreError::InvalidPayload("missing bundle_end marker".to_owned())
    })?;
    if bundle_end != "bundle_end|" {
        return Err(DurableGuardSnapshotStoreError::InvalidPayload(
            bundle_end.to_owned(),
        ));
    }
    if lines.next().is_some() {
        return Err(DurableGuardSnapshotStoreError::InvalidPayload(
            "extra payload lines after bundle_end marker".to_owned(),
        ));
    }

    let bundle = DurableGuardSnapshotBundle {
        schema_version: bundle_schema,
        delivery_guard: DeliveryGuardSnapshot {
            schema_version: delivery_schema,
            next_nonce_by_sender,
            seen_message_ids,
        },
        channel_policy: ChannelPolicySnapshot {
            schema_version: channel_schema,
            channels,
        },
    };
    validate_bundle(&bundle)?;
    Ok(bundle)
}

#[derive(Debug, Clone)]
struct ChannelPolicyBuilder {
    channel_id: String,
    members: Vec<String>,
    admins: Vec<String>,
    send: Option<PermissionRule>,
    read: Option<PermissionRule>,
    invite: Option<PermissionRule>,
    remove: Option<PermissionRule>,
    configure: Option<PermissionRule>,
    retention: Option<RetentionPolicy>,
}

impl ChannelPolicyBuilder {
    fn new(channel_id: String) -> Self {
        Self {
            channel_id,
            members: Vec::new(),
            admins: Vec::new(),
            send: None,
            read: None,
            invite: None,
            remove: None,
            configure: None,
            retention: None,
        }
    }

    fn set_send(&mut self, value: PermissionRule) -> Result<(), DurableGuardSnapshotStoreError> {
        if self.send.replace(value).is_some() {
            return Err(DurableGuardSnapshotStoreError::InvalidPayload(
                "duplicate channel_perm_send field".to_owned(),
            ));
        }
        Ok(())
    }

    fn set_read(&mut self, value: PermissionRule) -> Result<(), DurableGuardSnapshotStoreError> {
        if self.read.replace(value).is_some() {
            return Err(DurableGuardSnapshotStoreError::InvalidPayload(
                "duplicate channel_perm_read field".to_owned(),
            ));
        }
        Ok(())
    }

    fn set_invite(&mut self, value: PermissionRule) -> Result<(), DurableGuardSnapshotStoreError> {
        if self.invite.replace(value).is_some() {
            return Err(DurableGuardSnapshotStoreError::InvalidPayload(
                "duplicate channel_perm_invite field".to_owned(),
            ));
        }
        Ok(())
    }

    fn set_remove(&mut self, value: PermissionRule) -> Result<(), DurableGuardSnapshotStoreError> {
        if self.remove.replace(value).is_some() {
            return Err(DurableGuardSnapshotStoreError::InvalidPayload(
                "duplicate channel_perm_remove field".to_owned(),
            ));
        }
        Ok(())
    }

    fn set_configure(
        &mut self,
        value: PermissionRule,
    ) -> Result<(), DurableGuardSnapshotStoreError> {
        if self.configure.replace(value).is_some() {
            return Err(DurableGuardSnapshotStoreError::InvalidPayload(
                "duplicate channel_perm_configure field".to_owned(),
            ));
        }
        Ok(())
    }

    fn set_retention(
        &mut self,
        value: RetentionPolicy,
    ) -> Result<(), DurableGuardSnapshotStoreError> {
        if self.retention.replace(value).is_some() {
            return Err(DurableGuardSnapshotStoreError::InvalidPayload(
                "duplicate channel_retention field".to_owned(),
            ));
        }
        Ok(())
    }

    fn build(self) -> Result<ChannelPolicySnapshotChannel, DurableGuardSnapshotStoreError> {
        let send = self.send.ok_or_else(|| {
            DurableGuardSnapshotStoreError::InvalidPayload(
                "missing channel_perm_send field".to_owned(),
            )
        })?;
        let read = self.read.ok_or_else(|| {
            DurableGuardSnapshotStoreError::InvalidPayload(
                "missing channel_perm_read field".to_owned(),
            )
        })?;
        let invite = self.invite.ok_or_else(|| {
            DurableGuardSnapshotStoreError::InvalidPayload(
                "missing channel_perm_invite field".to_owned(),
            )
        })?;
        let remove = self.remove.ok_or_else(|| {
            DurableGuardSnapshotStoreError::InvalidPayload(
                "missing channel_perm_remove field".to_owned(),
            )
        })?;
        let configure = self.configure.ok_or_else(|| {
            DurableGuardSnapshotStoreError::InvalidPayload(
                "missing channel_perm_configure field".to_owned(),
            )
        })?;
        let retention = self.retention.ok_or_else(|| {
            DurableGuardSnapshotStoreError::InvalidPayload(
                "missing channel_retention field".to_owned(),
            )
        })?;

        Ok(ChannelPolicySnapshotChannel {
            channel_id: self.channel_id,
            members: self.members,
            admins: self.admins,
            permissions: ChannelPermissions {
                send,
                read,
                invite,
                remove,
                configure,
                retention,
            },
        })
    }
}

fn parse_schema_line<'a, I>(
    lines: &mut I,
    prefix: &str,
    field_name: &'static str,
) -> Result<u16, DurableGuardSnapshotStoreError>
where
    I: Iterator<Item = &'a str>,
{
    let line = lines.next().ok_or_else(|| {
        DurableGuardSnapshotStoreError::InvalidPayload(format!("missing {field_name} field"))
    })?;
    let value = line
        .strip_prefix(prefix)
        .ok_or_else(|| DurableGuardSnapshotStoreError::InvalidPayload(line.to_owned()))?;
    value
        .parse::<u16>()
        .map_err(|_| DurableGuardSnapshotStoreError::InvalidPayload(line.to_owned()))
}

fn encode_permission_rule(rule: &PermissionRule) -> String {
    match rule {
        PermissionRule::All => "all".to_owned(),
        PermissionRule::Members => "members".to_owned(),
        PermissionRule::Admins => "admins".to_owned(),
        PermissionRule::Allowlist(values) => {
            let encoded_values = values
                .iter()
                .map(|value| encode_hex(value))
                .collect::<Vec<String>>()
                .join(",");
            format!("allowlist:{encoded_values}")
        }
    }
}

fn decode_permission_rule(value: &str) -> Result<PermissionRule, DurableGuardSnapshotStoreError> {
    match value {
        "all" => Ok(PermissionRule::All),
        "members" => Ok(PermissionRule::Members),
        "admins" => Ok(PermissionRule::Admins),
        _ => {
            let encoded = value
                .strip_prefix("allowlist:")
                .ok_or_else(|| DurableGuardSnapshotStoreError::InvalidPayload(value.to_owned()))?;
            let mut entries = BTreeSet::new();
            if !encoded.is_empty() {
                for token in encoded.split(',') {
                    entries.insert(decode_hex(token)?);
                }
            }
            Ok(PermissionRule::Allowlist(entries))
        }
    }
}

fn encode_retention_policy(policy: &RetentionPolicy) -> String {
    match policy {
        RetentionPolicy::Forever => "forever".to_owned(),
        RetentionPolicy::MaxAgeSeconds(value) => format!("max_age:{value}"),
        RetentionPolicy::MaxMessageCount(value) => format!("max_count:{value}"),
    }
}

fn decode_retention_policy(value: &str) -> Result<RetentionPolicy, DurableGuardSnapshotStoreError> {
    if value == "forever" {
        return Ok(RetentionPolicy::Forever);
    }
    if let Some(raw) = value.strip_prefix("max_age:") {
        let parsed = raw
            .parse::<u64>()
            .map_err(|_| DurableGuardSnapshotStoreError::InvalidPayload(value.to_owned()))?;
        return Ok(RetentionPolicy::MaxAgeSeconds(parsed));
    }
    if let Some(raw) = value.strip_prefix("max_count:") {
        let parsed = raw
            .parse::<usize>()
            .map_err(|_| DurableGuardSnapshotStoreError::InvalidPayload(value.to_owned()))?;
        return Ok(RetentionPolicy::MaxMessageCount(parsed));
    }
    Err(DurableGuardSnapshotStoreError::InvalidPayload(
        value.to_owned(),
    ))
}

fn encode_hex(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len() * 2);
    for byte in value.as_bytes() {
        encoded.push(char::from(HEX_CHARS[(byte >> 4) as usize]));
        encoded.push(char::from(HEX_CHARS[(byte & 0x0f) as usize]));
    }
    encoded
}

fn decode_hex(value: &str) -> Result<String, DurableGuardSnapshotStoreError> {
    if !value.len().is_multiple_of(2) {
        return Err(DurableGuardSnapshotStoreError::InvalidPayload(
            value.to_owned(),
        ));
    }
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len() / 2);
    let mut index = 0;
    while index < bytes.len() {
        let high = decode_hex_nibble(bytes[index])?;
        let low = decode_hex_nibble(bytes[index + 1])?;
        decoded.push((high << 4) | low);
        index += 2;
    }
    String::from_utf8(decoded)
        .map_err(|_| DurableGuardSnapshotStoreError::InvalidPayload(value.to_owned()))
}

fn decode_hex_nibble(value: u8) -> Result<u8, DurableGuardSnapshotStoreError> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err(DurableGuardSnapshotStoreError::InvalidPayload(
            "invalid hex character".to_owned(),
        )),
    }
}

const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

#[cfg(test)]
mod tests {
    use super::{
        decode_hex, decode_permission_rule, decode_retention_policy, deserialize_bundle,
        encode_hex, encode_permission_rule, encode_retention_policy, serialize_bundle,
        DurableGuardBundleSnapshotStore, DurableGuardSnapshotBundle,
        DurableGuardSnapshotStoreError, InMemoryDurableGuardSnapshotStore,
    };
    use crate::{
        ChannelPermissionEngine, ChannelPermissions, DeliveryGuardInput, DeliveryValidationResult,
        MessageDeliveryGuards, PermissionRule, RetentionPolicy,
    };
    use std::collections::BTreeSet;

    fn delivery_input(message_id: &str, nonce: u64, received_at: &str) -> DeliveryGuardInput {
        DeliveryGuardInput {
            message_id: message_id.to_owned(),
            sender: "kamn:did:agent:sender-1".to_owned(),
            recipient: "kamn:did:agent:recipient-1".to_owned(),
            nonce,
            created: "2026-02-09T00:00:00.000Z".to_owned(),
            expires: "2026-02-09T00:30:00.000Z".to_owned(),
            received_at: received_at.to_owned(),
        }
    }

    #[test]
    fn hex_encoding_roundtrip() {
        let value = "kamn:did:agent:sender-1|nonce";
        let encoded = encode_hex(value);
        let decoded = decode_hex(&encoded).expect("hex decoding should pass");
        assert_eq!(decoded, value);
    }

    #[test]
    fn permission_rule_encoding_roundtrip() {
        let rule = PermissionRule::Allowlist(BTreeSet::from([
            "kamn:did:agent:a".to_owned(),
            "kamn:did:agent:b".to_owned(),
        ]));
        let encoded = encode_permission_rule(&rule);
        let decoded = decode_permission_rule(&encoded).expect("rule decode should pass");
        assert_eq!(decoded, rule);
    }

    #[test]
    fn retention_policy_encoding_roundtrip() {
        let policy = RetentionPolicy::MaxMessageCount(64);
        let encoded = encode_retention_policy(&policy);
        let decoded = decode_retention_policy(&encoded).expect("policy decode should pass");
        assert_eq!(decoded, policy);
    }

    #[test]
    fn bundle_serialization_roundtrip() {
        let mut guards = MessageDeliveryGuards::new();
        let mut channels = ChannelPermissionEngine::new();
        channels
            .register_channel(
                "channel:group:bundle-roundtrip",
                vec![
                    "kamn:did:agent:owner".to_owned(),
                    "kamn:did:agent:member-1".to_owned(),
                ],
                vec!["kamn:did:agent:owner".to_owned()],
                ChannelPermissions {
                    send: PermissionRule::Members,
                    read: PermissionRule::Members,
                    invite: PermissionRule::Admins,
                    remove: PermissionRule::Admins,
                    configure: PermissionRule::Admins,
                    retention: RetentionPolicy::MaxMessageCount(2),
                },
            )
            .expect("channel registration should pass");

        assert_eq!(
            guards.validate(delivery_input(
                "urn:uuid:bundle-roundtrip-1",
                1,
                "2026-02-09T00:10:00.000Z"
            )),
            DeliveryValidationResult::Accepted
        );

        let bundle = DurableGuardSnapshotBundle::capture(&guards, &channels);
        let payload = serialize_bundle(&bundle);
        let decoded = deserialize_bundle(&payload).expect("bundle decode should pass");
        assert_eq!(decoded, bundle);
    }

    #[test]
    fn in_memory_store_rejects_invalid_bundle_schema() {
        let guards = MessageDeliveryGuards::new();
        let channels = ChannelPermissionEngine::new();
        let mut bundle = DurableGuardSnapshotBundle::capture(&guards, &channels);
        bundle.schema_version = bundle.schema_version.saturating_add(1);
        let mut store = InMemoryDurableGuardSnapshotStore::default();
        assert_eq!(
            store.save_bundle(bundle),
            Err(
                DurableGuardSnapshotStoreError::BundleSchemaVersionMismatch {
                    expected: 1,
                    found: 2
                }
            )
        );
    }
}
