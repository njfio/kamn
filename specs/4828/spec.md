# Spec — Issue #4828

- Title: Subtask: migrate first 60 eligible `*_contract.py` checks and wrapper entrypoints
- Parent: Parent task `#4815`
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Implemented
- Priority: P1

## Objective

Route the first deterministic cohort of 60 eligible check-wrapper entrypoints through the declarative checker gateway while preserving legacy checker behavior and output compatibility.

## Problem Statement

Eligible wrapper entrypoints still dispatch directly to legacy checker targets, preventing phased migration governance and making it hard to enforce a stable cohort boundary for declarative migration progress tracking.

## Scope

In scope:
- eligibility gate in `scripts/lib/exec_dispatch.py` for migration cohort v1
- declarative checker legacy delegation mode with transparent output forwarding
- deterministic contract test asserting cohort size (`60`) and delegation wiring behavior
- unit test coverage for legacy delegation execution path

Out of scope:
- rewriting legacy checker logic for migrated cohort in this subtask
- migrating wrappers outside cohort v1 eligibility boundary

## Acceptance Criteria

- AC-1: Eligible wrapper cohort v1 is deterministically defined as exactly 60 wrappers and enforced by regression tests.
- AC-2: Wrapper behavior remains compatibility-safe after routing through declarative checker legacy delegation (same forwarded args, same exit behavior, same output stream shape).
- AC-3: Declarative checker supports compatibility inputs (`--bundle-file` alias, legacy target delegation options) without regressing existing policy-file mode behavior.

## Conformance Cases

- C-01 (AC-1, Conformance): `scripts/lib/test_exec_dispatch_registry.sh` asserts cohort size equals 60 using deterministic eligibility rules.
- C-02 (AC-2, Functional/Integration): `scripts/lib/test_exec_dispatch_registry.sh` sandbox delegation check verifies wrapper dispatch sets `KAMN_DECLARATIVE_POLICY_CHECKER_DELEGATE=1` and preserves prefixed + passthrough args.
- C-03 (AC-3, Unit/Functional): `scripts/framework/test_declarative_policy_checker.py` covers legacy delegate mode and existing policy-file mode; representative wrapper policy tests remain green.
- C-04 (Regression): `bash scripts/ci/test_ci_tools.sh` passes after migration routing changes.

## Success Metrics / Signals

- Deterministic cohort migration marker: `60` wrappers enforced in test contract.
- No regression in wrapper contract suites and CI tools regression matrix.
- Declarative checker can serve both direct-policy and compatibility-delegate paths.
