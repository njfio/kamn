# Issue #5336 Spec

- Title: Task: close remaining R45 review gaps for signer flake, branch hygiene, and docs-contract wave 4
- Status: Implemented (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
R46 review (`docs/review/gaps-and-issues-r45.md`) flags three residual operational gaps:
1. signer env guard tests remain intermittently flaky under parallel load,
2. branch hygiene re-accumulation requires active closure evidence,
3. docs-contract include_str suite-file surface remains higher than desired after wave 3.

## Acceptance Criteria
- AC-1: `kamn-node` managed-backend signer tests use the same crate-wide signer env lock domain as `main_tests` and `signer` tests.
- AC-2: signer mismatch regression and adjacent signer env tests remain deterministic under parallel execution stress.
- AC-3: branch hygiene gap is closed with current measured remote branch count and fresh cleanup evidence recorded in repository review docs.
- AC-4: docs-contract wave-4 tranche reduces include_str test-file count by at least 10 from baseline while preserving migrated assertion semantics.
- AC-5: all touched suites remain rustfmt/clippy/test clean in targeted verification.

## Scope
In scope:
- Test-only lock unification in `crates/kamn-node/src/signer/managed_backend.rs` tests.
- Review/measurement updates for branch hygiene evidence in `docs/review/gaps-and-issues-r45.md`.
- New docs-contract wave-4 harness and migration of a tranche of low-coupling docs-contract suites.

Out of scope:
- Production signer behavior changes.
- New dependencies.
- Unrelated CI workflow redesign.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Regression | lock-domain alias regression test | managed-backend test lock aliases crate signer lock |
| C-02 | AC-2 | Integration | parallel signer regression commands | deterministic pass for targeted signer env suites |
| C-03 | AC-3 | Functional | branch-count measurement + cleanup run evidence | branch count under budget and evidence captured in review doc |
| C-04 | AC-4 | Structural | include_str file count before/after + harness tests | count reduced by >=10 with migrated assertions preserved |
| C-05 | AC-5 | Quality | fmt/clippy/targeted tests | no formatting, lint, or regression failures |

## Test Mapping
- `cargo test -p kamn-node signer::managed_backend::tests::regression_managed_backend_env_lock_aliases_shared_signer_lock -- --exact`
- `cargo test -p kamn-node main_tests::signer_tests::regression_kolme_live_managed_external_backend_response_rejects_signer_public_key_mismatch -- --exact --test-threads=16`
- `cargo test -p kamn-node signer::tests::regression_signer_secret_source_precedence_failure_zeroizes_env_secret_buffer -- --exact --test-threads=16`
- `cargo test -p kamn-node main_tests::runtime_tests::unit_kolme_live_local_signer_override_marker_parses_boolean_values -- --exact --test-threads=16`
- `git ls-remote --heads origin | wc -l`
- `gh run list --workflow branch-cleanup.yml --limit 5 --json createdAt,conclusion,event`
- `rg -l "include_str!\(" crates/kamn-core/tests crates/kamn-node/tests crates/kamn-sdk/tests | wc -l`
- `cargo test -p kamn-core --test docs_contract_wave4_harness`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`

## Success Metrics
- No residual lock-domain split for signer env tests.
- include_str suite-file count reduced by >=10 from current baseline.
- branch hygiene marker in review docs reflects measured post-cleanup state (<100 remote branches).
