# Issue 6258 Spec

Status: Reviewed
Priority: P1
Milestone: R59 Swarm Gap Closure
Parent: #6256

## Objective
Reduce unintended root-level public API exposure from `kamn-core` by removing
deprecated compatibility shims and restricting internal implementation modules,
while preserving intended consumable API through curated re-exports.

## Inputs/Outputs
Inputs:
- Current root module declarations in `crates/kamn-core/src/lib.rs`.
- Existing curated `pub use` export list in `crates/kamn-core/src/lib.rs`.
- Existing in-workspace consumers (`kamn-types`, `kamn-node`, and other crates).
- Public API policy fixtures under `fixtures/ci/` and thresholds under `.ci/`.

Outputs:
- Tightened root module visibility in `crates/kamn-core/src/lib.rs`.
- Updated downstream usage to curated exports where needed.
- Updated API-surface baseline fixture aligned to post-tightening surface.
- Passing conformance/test evidence for issue-scope checks.

## Boundaries/Non-goals
In scope:
- Remove root `pub mod` declarations for deprecated shim modules.
- Convert internal adapter/state/helper modules from `pub mod` to private `mod`.
- Keep consumer-facing API available via explicit `pub use` re-exports.
- Update in-workspace callsites relying on removed module-path exports.

Out of scope:
- Runtime behavior changes.
- Cross-repo consumer migration and external release notes.
- Reworking unrelated performance-budget tests.

## Failure Modes
- FM-1: Missing re-export causes downstream compile failure.
- FM-2: Policy fixture mismatch (`module_count` / module entries / totals) causes
  `public_api_surface_policy` test failure.
- FM-3: Hidden module needed by external callers causes compatibility break
  beyond intended scope.

## Acceptance Criteria (Testable Booleans)
- AC-1: `crates/kamn-core/src/lib.rs` no longer root-exports the targeted
  internal and deprecated shim modules.
- AC-2: Workspace crates that consume `kamn-core` compile and their tests pass
  under issue-scope verification.
- AC-3: In-workspace consumers use curated root exports instead of
  `kamn_core::<module>::...` internal module paths.

## Files to Touch
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-types/src/lib.rs`
- `fixtures/ci/kamn_core_public_api_surface_baseline.env`
- `specs/6258/spec.md`
- `specs/6258/plan.md`
- `specs/6258/tasks.md`

## Error Semantics
- Preserve existing hard-fail behavior in policy checks.
- No silent fallback for missing baseline keys or module mismatches.
- Compile/test failures are authoritative and require explicit fixture or export
  corrections.

## Test Plan
Conformance commands:
- C-01 (AC-1): `rg -n '^pub mod (anti_spam|cross_chain_receipt|fairness_policy|live_probe_matrix|message_delivery_guards|quota_policy|retention_engine|watchdog|data_layer_postgres_execution_adapter|data_layer_postgres_repository_bridge|migrations|namespaces|smoke|sqlite_store_backend|state);' crates/kamn-core/src/lib.rs` reports `0`.
- C-02 (AC-2): `cargo test -p kamn-core --test public_api_surface_policy`
  passes; `cargo test -p kamn-types` passes; `cargo test -p kamn-node` passes.
- C-03 (AC-3): `rg -n 'kamn_core::[a-z][a-z0-9_]*::' crates --glob '!crates/kamn-core/**'`
  reports `0`.

Regression coverage:
- `cargo test -p kamn-core -p kamn-types -p kamn-node` (noting unrelated local
  perf-budget sensitivity in `performance_signer_emulator_contract_lane_stays_within_budget`).
