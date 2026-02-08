# Direct and Group Channel Models (Issue #118)

This document defines the first implementation slice for direct/group channel
models with membership and admin operations.

## Core Models
- `ChannelType`:
  - `Direct`
  - `Group`
- `ChannelStore`: in-memory channel registry with deterministic member-to-channel indexing.

## Supported Operations
- `create_direct(channel_id, participant_a, participant_b)`
  - requires two distinct valid `kamn:did:agent:*` participants.
  - both participants become members and admins.
- `create_group(channel_id, creator, members, admins)`
  - requires non-empty members/admins.
  - creator must be both member and admin.
  - admins must be a subset of members.
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
  - `members(channel_id)`
  - `admins(channel_id)`
  - `is_member(channel_id, did)`
  - `channels_for_member(did)`

## Validation and Safety Rules
- Channel IDs must be non-empty and unique.
- DIDs must parse as `kamn:did:agent:*`.
- Non-admin actors cannot mutate group membership/admin state.
- Direct channels reject unsupported group-style mutations.

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
