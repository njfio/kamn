# Plan: Issue #6033

## Approach
1. Build compact registry fixtures to register deterministic monthly partitions.
2. Add RED tests for archival due filtering/sorting and invalid reattach transition behavior.
3. Add deterministic recovery-readiness assertions across active/archived states.
4. Keep production code unchanged unless tests expose a contract mismatch.
5. Run targeted M10 slices and adjacent module regressions.

## Affected Modules
- `crates/kamn-core/src/data_layer_m10_partition_archival/registry.rs`

## Risks / Mitigations
- Risk: date/month arithmetic assumptions could invalidate test expectations.
  Mitigation: use explicit fixed month IDs and retention-window values with straightforward distances.
- Risk: archival filtering may be misattributed to one predicate.
  Mitigation: fixture includes partitions that independently exercise each predicate (shredded, retention age, lifecycle status).

## Interfaces / Contracts
- No public API changes.
- Test-only additions validating existing M10 registry contracts.
