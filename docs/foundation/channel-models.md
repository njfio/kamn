# Channel Models and Snapshot Persistence Contracts (Issues #118, #228, #229, #617)

This document defines the implemented channel model slices for direct/group
channels and specialized broadcast/task/marketplace/governance channels.
For marketplace listing/discovery workflow controls built on marketplace channels, see `docs/foundation/service-marketplace-discovery.md`.

## Core Models
- `ChannelType`:
  - `Direct`
  - `Group`
  - `Broadcast`
  - `Task`
  - `Marketplace`
  - `Governance`
- `ChannelMetadata`:
  - `Direct`
  - `Group`
  - `Broadcast { topic }`
  - `Task { task_id }`
  - `Marketplace { market_scope }`
  - `Governance { proposal_scope }`
- `ChannelStore`: in-memory channel registry with deterministic member-to-channel indexing.
- Snapshot contracts:
  - `ChannelRecordSnapshot`
  - `ChannelSnapshot`
  - `ChannelSnapshotStore`
  - `InMemoryChannelSnapshotStore`
  - `FileChannelSnapshotStore`

## Supported Operations
- `create_direct(channel_id, participant_a, participant_b)`
  - requires two distinct valid `kamn:did:agent:*` participants.
  - both participants become members and admins.
- `create_group(channel_id, creator, members, admins)`
  - requires non-empty members/admins.
  - creator must be both member and admin.
  - admins must be a subset of members.
- `create_broadcast(channel_id, creator, topic, members, admins)`
  - validates non-empty `topic`.
- `create_task_channel(channel_id, creator, task_id, members, admins)`
  - validates non-empty `task_id`.
  - requires at least 2 members.
- `create_marketplace_channel(channel_id, creator, market_scope, members, admins)`
  - validates non-empty `market_scope`.
  - requires at least 2 members.
- `create_governance_channel(channel_id, creator, proposal_scope, members, admins)`
  - validates non-empty `proposal_scope`.
  - requires at least 3 members.
- `invite_member(channel_id, actor, new_member)`
  - actor must be admin (group channel only).
- `remove_member(channel_id, actor, member)`
  - actor must be admin (group channel only).
  - removing the last admin is blocked.
- `add_admin(channel_id, actor, member)` / `remove_admin(channel_id, actor, member)`
  - actor must be admin (group channel only).
  - removing the last admin is blocked.
- Query APIs:
  - `channel_type(channel_id)`
  - `metadata(channel_id)`
  - `members(channel_id)`
  - `admins(channel_id)`
  - `is_member(channel_id, did)`
  - `channels_for_member(did)`
- Snapshot APIs:
  - `export_snapshot()`
  - `restore_snapshot(snapshot)`
  - `ChannelSnapshotStore::write(snapshot)`
  - `ChannelSnapshotStore::read_latest()`
  - `FileChannelSnapshotStore::recover_latest_and_repair()`

## Validation and Safety Rules
- Channel IDs must be non-empty and unique.
- DIDs must parse as `kamn:did:agent:*`.
- Non-admin actors cannot mutate group membership/admin state.
- Direct channels reject unsupported group-style mutations.
- Specialized channels enforce typed metadata and per-type minimum member thresholds.

## Snapshot Persistence and Restore Contract Rules
- Snapshot schema is versioned by `CHANNEL_SNAPSHOT_SCHEMA_VERSION`.
- Restore validation enforces channel ID, DID, metadata/type compatibility, and per-type member/admin invariants.
- Restore guards reject:
  - duplicate channel IDs
  - duplicate members/admins within one channel record
  - admin/member mismatch (admin must be a member)
  - direct-channel snapshot state that does not preserve exactly two distinct participants/admins
- File-backed persistence uses deterministic payload lines:
  - `schema|<version>`
  - `record|<channel_id>|<type_code>|<metadata_value>|<members_csv>|<admins_csv>`
- Delimiter poisoning (`|`, newline, `,`) is rejected for scalar fields during serialization.
- Corrupt payload recovery truncates invalid data and returns `latest=None` with `repaired=true`.
- Regression contract:
  - duplicate channel IDs on restore are rejected (`Regression: #617`)
  - admin/member mismatch on restore is rejected (`Regression: #617`)

## Fast and Cost-Effective Validation
Run targeted checks from repository root:

```bash
cargo test -p kamn-core --lib channel_models::tests::
cargo test -p kamn-core --test channel_models
cargo test -p kamn-core --test channel_models_docs
bash scripts/channel/run_channel_lifecycle_contract_lane.sh
```

Scheduled deep-lane command:

```bash
cargo test -p kamn-core --lib channel_models::tests::performance_channel_snapshot_deep_lane_stress -- --ignored
bash scripts/channel/run_channel_lifecycle_deep_lane.sh
```

Then run strict gates:

```bash
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```
