use super::super::{ChannelMetadata, ChannelModelError, ChannelStore, ChannelType};

#[test]
fn group_creator_must_be_member() {
    let mut store = ChannelStore::new();
    assert_eq!(
        store.create_group(
            "channel:group:1",
            "kamn:did:agent:owner",
            vec!["kamn:did:agent:member-1".to_owned()],
            vec!["kamn:did:agent:member-1".to_owned()],
        ),
        Err(ChannelModelError::CreatorNotMember(
            "kamn:did:agent:owner".to_owned()
        ))
    );
}

#[test]
fn direct_channels_require_distinct_participants() {
    let mut store = ChannelStore::new();
    assert_eq!(
        store.create_direct(
            "channel:direct:1",
            "kamn:did:agent:alice",
            "kamn:did:agent:alice",
        ),
        Err(ChannelModelError::InvalidDirectParticipants)
    );
}

#[test]
fn governance_channels_require_three_members() {
    let mut store = ChannelStore::new();
    assert_eq!(
        store.create_governance_channel(
            "channel:gov:1",
            "kamn:did:agent:owner",
            "core-protocol",
            vec![
                "kamn:did:agent:owner".to_owned(),
                "kamn:did:agent:validator-1".to_owned(),
            ],
            vec!["kamn:did:agent:owner".to_owned()],
        ),
        Err(ChannelModelError::InsufficientMembers {
            channel_type: ChannelType::Governance,
            minimum: 3,
            actual: 2,
        })
    );
}

#[test]
fn broadcast_metadata_requires_non_empty_topic() {
    let mut store = ChannelStore::new();
    assert_empty_topic_rejected(&mut store);
    assert_broadcast_metadata_persisted(&mut store);
}

fn assert_empty_topic_rejected(store: &mut ChannelStore) {
    assert_eq!(
        store.create_broadcast(
            "channel:broadcast:1",
            "kamn:did:agent:owner",
            "",
            vec!["kamn:did:agent:owner".to_owned()],
            vec!["kamn:did:agent:owner".to_owned()],
        ),
        Err(ChannelModelError::InvalidMetadata(
            "topic must not be empty".to_owned()
        ))
    );
}

fn assert_broadcast_metadata_persisted(store: &mut ChannelStore) {
    store
        .create_broadcast(
            "channel:broadcast:2",
            "kamn:did:agent:owner",
            "announcements",
            vec!["kamn:did:agent:owner".to_owned()],
            vec!["kamn:did:agent:owner".to_owned()],
        )
        .expect("broadcast should be created");
    assert_eq!(
        store
            .metadata("channel:broadcast:2")
            .expect("metadata should resolve"),
        ChannelMetadata::Broadcast {
            topic: "announcements".to_owned(),
        }
    );
}
