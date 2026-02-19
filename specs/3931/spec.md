# Issue #3931 Spec

- Title: Subtask: add cargo-fuzz targets for message envelope and DID parsing with reproducible seed corpus
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Problem Statement
Parser robustness needs dedicated `cargo fuzz` targets with deterministic seed-corpus governance and fail-closed documentation contracts.

## Acceptance Criteria
- AC-1: Cargo-fuzz target package exists with message-envelope and DID parser targets bound to production parsing APIs.
- AC-2: Reproducible seed corpus and replay metadata exist for both fuzz targets.
- AC-3: CI strategy documents CI-smoke vs local-heavy fuzz boundaries and command surface for these targets.
- AC-4: Contract tests fail closed on fuzz target/corpus/docs marker drift.
- AC-5: Targeted lint/tests and shell guardrails pass.

## Scope
In scope:
- `fuzz/Cargo.toml`
- `fuzz/fuzz_targets/{message_envelope_parser.rs,did_parser.rs}`
- `fuzz/corpus/**` (seed inputs + replay metadata)
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/{ci_strategy_docs.rs,cargo_fuzz_target_contract.rs}`
- `specs/3931/{spec.md,plan.md,tasks.md}`

Out of scope:
- Enabling heavy fuzz execution in `ci-fast-gate`
- New shell scripts or workflow file changes
- Production runtime logic changes

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Fuzz package manifest + target source files | Both parser targets compile-reference production APIs |
| C-02 | AC-2 | Regression | Corpus seed files + replay metadata | Seed artifacts exist and remain deterministic |
| C-03 | AC-3 | Functional | CI strategy section markers | CI-smoke/local-heavy command boundaries are documented |
| C-04 | AC-4 | Conformance | Rust contract tests for fuzz package/docs | Missing markers/files fail closed |
| C-05 | AC-5 | Regression | fmt/clippy/tests/shell guardrails | All green with no shell-surface regression |

## Test Mapping
- `cargo test -p kamn-core --test cargo_fuzz_target_contract`
- `cargo test -p kamn-core --test ci_strategy_docs`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`
- `scripts/ci/check_shell_loc_hard_ceiling.sh --output-json /tmp/shell-loc-hard-ceiling-3931.json`
- `scripts/ci/check_shell_rust_ratio_guardrail.sh --output-json /tmp/shell-rust-ratio-3931.json`
- `scripts/ci/check_shell_surface_threshold_ratchet.sh --output-json /tmp/shell-threshold-ratchet-3931.json`

## Success Metrics
- Fuzz target package and deterministic corpora are tracked in git.
- CI/docs contracts pin command surface and local-heavy boundaries.
- Shell LOC remains unchanged.
