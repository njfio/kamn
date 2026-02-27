# Issue 6224 Spec

Status: Reviewed
Priority: P1
Milestone: R59 Swarm Gap Closure
Parent: #6223

## Problem Statement
`quota_policy.rs` and `fairness_policy.rs` currently lack direct, explicit contract tests at the module level for fail-closed reason selection and boundary behavior. This leaves security policy behavior under-specified in executable form and weakens regression protection.

## Scope
In scope:
- Add direct tests in `crates/kamn-runtime-guards/src/quota_policy.rs` covering allow/reject and fail-closed reason mapping.
- Add direct tests in `crates/kamn-runtime-guards/src/fairness_policy.rs` covering allow/reject and fail-closed reason mapping.
- Add direct tests in `crates/kamn-core/src/quota_policy.rs` and `crates/kamn-core/src/fairness_policy.rs` validating compatibility re-export contracts remain executable.

Out of scope:
- Policy algorithm redesign.
- Runtime behavior changes in unrelated modules.

## Acceptance Criteria
- AC-1: Runtime-guards quota policy tests cover unknown scope, zero window, zero limit, over-limit reject, and boundary allow (`observed_count == limit`).
- AC-2: Runtime-guards fairness policy tests cover unknown scope, zero window, zero max-gap, weighted-share exceed reject, and boundary allow (`active_weighted_share == max_weighted_share_gap`).
- AC-3: Core compatibility modules include direct tests asserting quota/fairness APIs are callable through `kamn-core` re-exports and preserve deterministic reason markers.
- AC-4: `cargo test -p kamn-runtime-guards` and targeted `cargo test -p kamn-core` for the added policy tests both pass.

## Conformance Cases
- C-01 (AC-1, Unit/Conformance): `evaluate_quota_policy` rejects `scope="unknown"` with `quota_scope_unknown`.
- C-02 (AC-1, Unit/Conformance): `evaluate_quota_policy` rejects zero window with `quota_window_non_positive`.
- C-03 (AC-1, Unit/Conformance): `evaluate_quota_policy` rejects zero limit with `quota_limit_non_positive`.
- C-04 (AC-1, Unit/Conformance): `evaluate_quota_policy` rejects `observed_count > limit` with `quota_limit_exceeded` and allows `observed_count == limit`.
- C-05 (AC-2, Unit/Conformance): `evaluate_fairness_policy` rejects `scope="unknown"` with `fairness_scope_unknown`.
- C-06 (AC-2, Unit/Conformance): `evaluate_fairness_policy` rejects zero window with `fairness_window_non_positive`.
- C-07 (AC-2, Unit/Conformance): `evaluate_fairness_policy` rejects zero max-gap with `fairness_max_gap_non_positive`.
- C-08 (AC-2, Unit/Conformance): `evaluate_fairness_policy` rejects `active_weighted_share > max_weighted_share_gap` with `fairness_weighted_share_exceeds_gap` and allows equality boundary.
- C-09 (AC-3, Unit/Conformance): `kamn-core` quota re-export tests invoke helpers and decision APIs successfully with deterministic marker assertions.
- C-10 (AC-3, Unit/Conformance): `kamn-core` fairness re-export tests invoke helpers and decision APIs successfully with deterministic marker assertions.
- C-11 (AC-4, Functional): targeted crate test commands pass.

## Success Metrics
- Zero uncovered fail-closed branches for quota/fairness policy modules introduced by this issue.
- Deterministic reason-marker compatibility remains validated in both runtime-guards and core compatibility surfaces.
