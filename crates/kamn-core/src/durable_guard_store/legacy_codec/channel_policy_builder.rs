use crate::{ChannelPermissions, ChannelPolicySnapshotChannel, PermissionRule, RetentionPolicy};

use super::super::DurableGuardSnapshotStoreError;

#[derive(Debug, Clone)]
pub(super) struct ChannelPolicyBuilder {
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
    pub(super) fn new(channel_id: String) -> Self {
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

    pub(super) fn set_send(
        &mut self,
        value: PermissionRule,
    ) -> Result<(), DurableGuardSnapshotStoreError> {
        set_once(&mut self.send, value, "duplicate channel_perm_send field")
    }

    pub(super) fn set_read(
        &mut self,
        value: PermissionRule,
    ) -> Result<(), DurableGuardSnapshotStoreError> {
        set_once(&mut self.read, value, "duplicate channel_perm_read field")
    }

    pub(super) fn set_invite(
        &mut self,
        value: PermissionRule,
    ) -> Result<(), DurableGuardSnapshotStoreError> {
        set_once(
            &mut self.invite,
            value,
            "duplicate channel_perm_invite field",
        )
    }

    pub(super) fn set_remove(
        &mut self,
        value: PermissionRule,
    ) -> Result<(), DurableGuardSnapshotStoreError> {
        set_once(
            &mut self.remove,
            value,
            "duplicate channel_perm_remove field",
        )
    }

    pub(super) fn set_configure(
        &mut self,
        value: PermissionRule,
    ) -> Result<(), DurableGuardSnapshotStoreError> {
        set_once(
            &mut self.configure,
            value,
            "duplicate channel_perm_configure field",
        )
    }

    pub(super) fn set_retention(
        &mut self,
        value: RetentionPolicy,
    ) -> Result<(), DurableGuardSnapshotStoreError> {
        set_once(
            &mut self.retention,
            value,
            "duplicate channel_retention field",
        )
    }

    pub(super) fn build(
        self,
    ) -> Result<ChannelPolicySnapshotChannel, DurableGuardSnapshotStoreError> {
        Ok(ChannelPolicySnapshotChannel {
            channel_id: self.channel_id,
            members: self.members,
            admins: self.admins,
            permissions: ChannelPermissions {
                send: require(self.send, "missing channel_perm_send field")?,
                read: require(self.read, "missing channel_perm_read field")?,
                invite: require(self.invite, "missing channel_perm_invite field")?,
                remove: require(self.remove, "missing channel_perm_remove field")?,
                configure: require(self.configure, "missing channel_perm_configure field")?,
                retention: require(self.retention, "missing channel_retention field")?,
            },
        })
    }

    pub(super) fn members_mut(&mut self) -> &mut Vec<String> {
        &mut self.members
    }

    pub(super) fn admins_mut(&mut self) -> &mut Vec<String> {
        &mut self.admins
    }
}

fn set_once<T>(
    slot: &mut Option<T>,
    value: T,
    message: &'static str,
) -> Result<(), DurableGuardSnapshotStoreError> {
    if slot.replace(value).is_some() {
        return Err(DurableGuardSnapshotStoreError::InvalidPayload(
            message.to_owned(),
        ));
    }
    Ok(())
}

fn require<T>(
    value: Option<T>,
    message: &'static str,
) -> Result<T, DurableGuardSnapshotStoreError> {
    value.ok_or_else(|| DurableGuardSnapshotStoreError::InvalidPayload(message.to_owned()))
}
