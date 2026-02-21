# Issue #5422 Spec — Multi-Host Live-Postgres Validation With Batched Coherence Bundles

- Status: Reviewed
- Issue: #5422
- Parent: #3812
- Milestone: R27 Program: operational hardening and live validation

## Problem Statement
Same-host daemon live-postgres validation is contract-complete, but production integration remains incomplete without multi-host distributed-lane validation. Prior one-dimension-per-issue granularity created governance overhead and branch churn.

## Scope
In scope:
- Deliver multi-host distributed daemon live-postgres validation lane(s) with deterministic conformance contracts.
- Group remaining coherence dimensions into batched implementation bundles (target 5-8 bundles).
- Preserve stable selector paths and fail-closed taxonomy markers.

Out of scope:
- New protocol economics.
- Cross-region rollout.

## Coherence Bundle Map (Batched Delivery)
- B-01 Runtime Matrix Bundle: env gate + load-profile + role-profile + role-pair distributed projections.
- B-02 Parallel Lane Bundle: bounded/asymmetric parallel role-pair distributed lane invariance.
- B-03 Topology Mapping Bundle: topology scope + host-pair + lane-set/lane-count/host-mode mappings.
- B-04 Topology Coherence Bundle: host-cardinality, host-pair-cardinality, host-mode-host-pair coherence classes.
- B-05 Fingerprint Stability Bundle: lane fingerprint hash coherence, order normalization, digest stability.
- B-06 Multi-Host Execution Bundle: distributed host orchestration harness, runtime evidence projection, CI run-mode policy.

## Acceptance Criteria
- AC-1: Multi-host distributed daemon live-postgres lane exists with deterministic pass/fail projection and stable reason-taxonomy markers.
- AC-2: Remaining coherence dimensions are tracked/delivered through the six bundled groups above (no new one-dimension-per-issue fan-out).
- AC-3: Docs/specs publish bundle-to-conformance mapping and selector command surface.
- AC-4: CI/local validation commands remain reproducible and fail closed when distributed-lane prerequisites are missing.

## Conformance Cases
- C-01 (Integration, AC-1): multi-host distributed lane baseline executes with stable decision payload.
- C-02 (Functional, AC-1/AC-4): missing distributed-lane prerequisites fail closed with deterministic reason codes.
- C-03 (Conformance, AC-2): bundle map contract test enforces six-bundle grouping markers.
- C-04 (Conformance, AC-3): docs contract enforces bundle-to-command mapping markers.
- C-05 (Regression, AC-1): selector path compatibility remains `main_tests::daemon_tests::*`.
- C-06 (Performance, AC-4): distributed lane stays within bounded fast/deep execution envelope.

## Success Metrics
- Multi-host distributed lane is executable with deterministic outcomes.
- Coherence scope is represented by six bundles and does not re-fragment into dozens of micro issues.
- Contract markers prevent taxonomy/selector drift.

## AC -> Tests Mapping (initial)
- AC-1: distributed lane integration selector (to be introduced in implementation tasks).
- AC-2: bundle-map contract test (new `kamn-core` docs/contract test target).
- AC-3: docs contract assertions in ops/review planning docs.
- AC-4: CI policy and fail-closed guard tests for distributed-lane prerequisites.
