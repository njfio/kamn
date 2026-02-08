# Retention Policy Engine Slice (Issues #154, #155)

This document describes the first implementation slice for configurable retention policy enforcement.

## Scope Delivered
- Added `RetentionPolicyEngine` with configurable default retention class plus domain overrides.
- Added deterministic expiration evaluation across domains:
  - Stable sort order for evaluation and expiration output.
  - Domain override precedence over default policy class.
- Added operator status surface:
  - `status_for(...)` returns class + deterministic expiry timestamp per record.
- Added resurfacing regression guard:
  - Expired record IDs are blocked from reappearing in later evaluations.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test retention_policy_engine
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```

## Follow-up
- Add richer retention classes (class tiers + legal hold semantics).
- Integrate retention evaluation output with audit export and redaction workflows.
