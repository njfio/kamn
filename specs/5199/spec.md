# Issue #5199 Spec

- Title: Task: harden signer test env lock against mutex poisoning cascade
- Status: Reviewed
- Priority: P1
- Milestone: specs/milestones/r27-46-r42-gap-remediation-and-maintainability-closure/index.md

## Problem Statement
`main_tests` uses a shared process-wide env lock for signer/runtime env mutation. Any panic while holding this lock poisons the mutex. A subset of tests still acquire the lock via `.expect(...)`, which can trigger deterministic cascade failures after a single panic.

## Scope
In:
- Remove poison-propagating `signer_env_lock().lock().expect(...)` acquisition patterns from `main_tests` modules.
- Route all signer env lock acquisition through `lock_signer_env_guard()` poison-recovery semantics.
- Add regression coverage that intentionally poisons the lock and verifies subsequent lock acquisition still succeeds.

Out:
- Production signer/runtime behavior changes.
- New dependencies, CI workflow changes, or protocol changes.

## Acceptance Criteria
- AC-1: No `main_tests` callsite acquires `signer_env_lock()` with `.expect(...)`; all callsites use poison-recovery guard acquisition.
- AC-2: A regression test intentionally poisons the signer env lock and verifies subsequent lock acquisition succeeds without panicking.
- AC-3: Signer and CLI contract test paths that rely on signer env lock continue to pass after poison-recovery migration.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Conformance | `main_tests` lock-acquisition source scan | Zero remaining `signer_env_lock().lock().expect(...)` callsites |
| C-02 | AC-2 | Regression | Intentional panic while holding signer env lock | Next `lock_signer_env_guard()` call succeeds and test process remains stable |
| C-03 | AC-3 | Functional | signer + CLI test subsets | Selected signer and CLI tests pass under normal and poisoned-lock orderings |

## Test Mapping
- C-01 -> source assertions via updated lock callsites in:
  - `crates/kamn-node/src/main_tests/cli_contract_tests.rs`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- C-02 -> `crates/kamn-node/src/main_tests/cli_contract_tests.rs::regression_signer_env_lock_recovers_after_poison`
- C-03 -> targeted runs:
  - `cargo test -p kamn-node main_tests::cli_contract_tests:: -- --nocapture`
  - `cargo test -p kamn-node main_tests::signer_tests:: -- --nocapture`

## Success Metrics
- `signer_env_lock()` acquisition in `main_tests` is poison-recovery-only.
- No deterministic lock-poison cascade observed in targeted signer/CLI test runs.
- Shell LOC delta stays neutral (`0`).
