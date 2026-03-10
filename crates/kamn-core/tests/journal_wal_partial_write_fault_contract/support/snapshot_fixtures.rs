use kamn_core::{
    ChannelMetadata, ChannelRecordSnapshot, ChannelSnapshot, ChannelType, MessageLifecycleSnapshot,
    MessageRecordSnapshot, MessageStatus, TaskOperationNoticeKind, TaskOperationRecordSnapshot,
    TaskOperationSnapshot, TaskState, CHANNEL_SNAPSHOT_SCHEMA_VERSION,
    MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION, TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION,
};

pub(crate) fn channel_snapshots() -> (ChannelSnapshot, ChannelSnapshot) {
    let first = ChannelSnapshot {
        schema_version: CHANNEL_SNAPSHOT_SCHEMA_VERSION,
        records: vec![channel_record("channel-fixture-1", "member_1")],
    };
    let second = ChannelSnapshot {
        schema_version: CHANNEL_SNAPSHOT_SCHEMA_VERSION,
        records: vec![
            first.records[0].clone(),
            channel_record("channel-fixture-2", "member_2"),
        ],
    };
    (first, second)
}

pub(crate) fn message_snapshots() -> (MessageLifecycleSnapshot, MessageLifecycleSnapshot) {
    let first = MessageLifecycleSnapshot {
        schema_version: MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION,
        records: vec![message_record("message-fixture-1", "sender_1", "recipient_1", 0)],
    };
    let second = MessageLifecycleSnapshot {
        schema_version: MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION,
        records: vec![
            first.records[0].clone(),
            message_record("message-fixture-2", "sender_2", "recipient_2", 2),
        ],
    };
    (first, second)
}

pub(crate) fn task_snapshots() -> (TaskOperationSnapshot, TaskOperationSnapshot) {
    let first = TaskOperationSnapshot {
        schema_version: TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION,
        tasks: vec![task_record("task-fixture-1", "requester_1", "first fixture task")],
    };
    let second = TaskOperationSnapshot {
        schema_version: TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION,
        tasks: vec![
            first.tasks[0].clone(),
            task_record("task-fixture-2", "requester_2", "second fixture task"),
        ],
    };
    (first, second)
}

fn channel_record(channel_id: &str, member_id: &str) -> ChannelRecordSnapshot {
    ChannelRecordSnapshot {
        channel_id: channel_id.to_owned(),
        channel_type: ChannelType::Group,
        metadata: ChannelMetadata::Group,
        members: vec![did("owner"), did(member_id)],
        admins: vec![did("owner")],
    }
}

fn message_record(
    message_id: &str,
    sender_id: &str,
    recipient_id: &str,
    second_offset: u8,
) -> MessageRecordSnapshot {
    MessageRecordSnapshot {
        message_id: message_id.to_owned(),
        sender: did(sender_id),
        recipients: vec![did(recipient_id)],
        created: format!("2026-01-01T00:00:0{second_offset}Z"),
        expires: format!("2026-01-01T00:00:0{}Z", second_offset + 1),
        status: MessageStatus::Created,
        history: vec![MessageStatus::Created],
    }
}

fn task_record(task_id: &str, requester_id: &str, description: &str) -> TaskOperationRecordSnapshot {
    TaskOperationRecordSnapshot {
        task_id: task_id.to_owned(),
        requester: did(requester_id),
        assignee: None,
        description: description.to_owned(),
        lifecycle_history: vec![TaskState::Submitted],
        dependencies: Vec::new(),
        notices: vec![TaskOperationNoticeKind::Submitted],
    }
}

fn did(id: &str) -> String {
    format!("kamn:did:agent:{id}")
}
