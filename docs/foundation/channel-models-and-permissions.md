# Channel Models and Permissions Contract Rules

This document defines the bounded contract lane used to enforce channel-model
and channel-permission safety checks in fast CI.

## Scope
- Channel lifecycle model contracts from `channel_models`.
- Membership and permission fail-closed contracts from `channel_policies`.
- Retention policy behavior checks from `channel_permissions_retention`.

## Contract Lane Commands
Run from repository root:

```bash
bash scripts/channel/run_channel_policy_contract_lane.sh
bash scripts/channel/test_run_channel_policy_contract_lane.sh
bash scripts/channel/run_channel_retention_redaction_contract_lane.sh
bash scripts/channel/test_run_channel_retention_redaction_contract_lane.sh
bash scripts/channel/run_channel_lifecycle_contract_lane.sh
bash scripts/channel/test_run_channel_lifecycle_contract_lane.sh
cargo test -p kamn-core --test channel_permissions_retention
cargo test -p kamn-core --test channel_permissions_retention_docs
cargo test -p kamn-core --test channel_models_and_permissions_docs
```

## Deterministic Safety Rules
- Unauthorized actor actions must be rejected with deterministic rule context.
- Empty/invalid allowlist policies must fail at registration.
- Retention pruning must be deterministic for equal timestamps.
- CI lane wiring must stay scoped to channel docs/scripts without broad fan-out.

## Regression Marker
- unauthorized channel policy bypass is rejected (`Regression: #929`)
