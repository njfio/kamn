# Channel Permissions and Retention Policies (Issue #120)

This document defines the first implementation slice for channel permission
evaluation and retention policy behavior.

## Core Types
- `ChannelPermissionEngine`: deterministic in-memory permission/retention engine.
- `ChannelPermissions`: per-channel rules for:
  - `send`
  - `read`
  - `invite`
  - `remove`
  - `configure`
  - `retention`
- `PermissionRule`:
  - `All`
  - `Members`
  - `Admins`
  - `Allowlist(BTreeSet<String>)`
- `RetentionPolicy`:
  - `Forever`
  - `MaxAgeSeconds(u64)`
  - `MaxMessageCount(usize)`
- `RetentionMessage`: message metadata for retention candidate evaluation.

## Permission Evaluation
- `authorize(channel_id, actor, action)` validates actor DID and applies rule for action.
- Unauthorized access returns `ChannelPolicyError::Unauthorized` with actor/action/rule context.
- Channel registration enforces:
  - non-empty channel ID
  - non-empty members/admins
  - valid `kamn:did:agent:*` identifiers
  - admins must be members

## Retention Evaluation
- `retention_candidates(channel_id, now_secs, messages)` returns message IDs eligible for pruning.
- `Forever` returns no candidates.
- `MaxAgeSeconds` prunes messages where `now_secs - created_at_secs > max_age`.
- `MaxMessageCount` keeps newest `N` messages and prunes oldest deterministically
  (sort by `created_at_secs`, then message ID).
- Invalid policy guard:
  - `MaxMessageCount(0)` is rejected at registration.

## Local Validation
Run from repository root:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core --test channel_permissions_retention
cargo test -p kamn-core
```

## Notes
This slice is intentionally dependency-free and deterministic to keep CI fast
and low-cost while establishing channel safety controls.
