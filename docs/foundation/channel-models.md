# Channel Models (Issues #118, #228, #229)

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

## Validation and Safety Rules
- Channel IDs must be non-empty and unique.
- DIDs must parse as `kamn:did:agent:*`.
- Non-admin actors cannot mutate group membership/admin state.
- Direct channels reject unsupported group-style mutations.
- Specialized channels enforce typed metadata and per-type minimum member thresholds.

## Local Validation
Run from repository root:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core --test channel_models
cargo test -p kamn-core
```

## Notes
This slice is deterministic and dependency-light (`BTreeMap`/`BTreeSet`) to keep
CI fast and low-cost while establishing channel authorization semantics.
