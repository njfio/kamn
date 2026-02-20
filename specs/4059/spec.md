# Issue #4059 Spec

- Title: Task: implement audit-evidence integrity checker and ci dry-run release-governance contracts
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-13-authorization-tenant-isolation-and-audit-integrity-governance/index.md

## Problem Statement
R27.13 requires fail-closed audit-evidence integrity checks in release governance without introducing heavy/local-only execution into CI fast-gate. Existing go/no-go bundle scripts include audit-integrity gate logic, but this task lacks milestone-bound spec artifacts and deterministic CI strategy parity coverage.

## Scope
In scope:
- Add issue-level spec/plan/tasks artifacts for #4059.
- Add deterministic Rust contract tests that exercise audit-integrity gate generate/check flows in CI-safe dry-run mode.
- Add CI strategy documentation markers for audit-integrity checker commands, taxonomy, and fail-closed reasons.
- Add docs-contract tests that enforce the new CI strategy markers.

Out of scope:
- New shell wrapper script bodies.
- External audit systems/SIEM integration.
- Heavy runtime lane execution in CI fast-gate.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 320
- shell_to_rust_ratio_delta_estimate: -0.0015
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Audit-integrity evidence generation/check path is covered by deterministic Rust contract tests that validate GO behavior and marker schema in CI-safe dry-run mode.
- AC-2: Tampered audit-integrity gate payloads fail closed with deterministic mismatch reasons.
- AC-3: `docs/ci/strategy.md` documents audit-integrity dry-run checker commands and taxonomy/reason-code markers, and docs-contract tests enforce parity.
- AC-4: Verification commands for targeted tests and linting pass with no new shell script body growth.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | valid sqlite crash-recovery policy report + `generate_gonogo_evidence_bundle.sh` | GO bundle with deterministic audit-integrity gate markers |
| C-02 | AC-1 | Integration | generated bundle + `check_gonogo_evidence_policy.sh` | policy checker returns pass with `audit_integrity_gate_final_decision=GO` |
| C-03 | AC-2 | Regression | tampered `audit_integrity_gate.observed` payload | checker fails closed with `audit integrity gate convergence mismatch` |
| C-04 | AC-3 | Functional | CI strategy docs section content | required command/taxonomy/reason markers exist |
| C-05 | AC-4 | Performance | dry-run generate+check execution in Rust test | bounded runtime and deterministic outputs |

## Test Mapping
- `cargo test -p kamn-core --test audit_evidence_integrity_contract spec_c01_audit_integrity_generate_bundle_emits_deterministic_go_markers -- --exact`
- `cargo test -p kamn-core --test audit_evidence_integrity_contract spec_c02_audit_integrity_policy_checker_accepts_converged_bundle -- --exact`
- `cargo test -p kamn-core --test audit_evidence_integrity_contract regression_spec_c03_audit_integrity_policy_checker_rejects_tampered_gate_payload -- --exact`
- `cargo test -p kamn-core --test audit_evidence_integrity_contract performance_spec_c05_audit_integrity_generate_and_check_dry_run_stays_within_budget -- --exact`
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_audit_integrity_dry_run_governance_markers -- --exact`

## Success Metrics
- #4059 has repository-level spec artifacts and deterministic implementation evidence.
- Audit-integrity governance behavior is enforced by Rust tests and CI strategy docs parity, not ad hoc/manual review.
- Shell surface remains unchanged while coverage and governance fidelity increase.
