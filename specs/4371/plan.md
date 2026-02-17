# Plan — #4371

## Approach
- Extend local KAMN runtime integration policy checker with deterministic in-memory provider detection
  for production-mode command surfaces.
- Add RED tamper assertions in the contract-lane shell test, then implement GREEN mapping.
- Update operator docs and docs contract tests for marker parity.

## Affected Modules
- `scripts/kolme/check_local_kamn_live_runtime_integration_policy.py`
- `scripts/kolme/test_run_local_kamn_live_runtime_integration_contract_lane.sh`
- `docs/ops/configuration.md`
- `docs/foundation/release-gonogo-checklist.md`
- docs contract tests under `crates/kamn-core/tests/*_docs.rs`

## Risks and Mitigations
- Risk: Duplicate/overlapping reason semantics with real-node profile checker.
  - Mitigation: Reuse reason names already used by real-node guard paths.
- Risk: Docs/test drift.
  - Mitigation: Keep docs parity assertions in shell + Rust docs tests.

## Interfaces/Contracts
- New deterministic failure reason: `runtime_commit_in_memory_provider_reference_detected`.
- Policy output must include the reason in `reason_codes` when marker drift is present.

## ADR
- Not required (no architectural or dependency change).
