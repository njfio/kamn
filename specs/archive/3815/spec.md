# Spec - Issue #3815

- Title: Epic: R27.3 deliver live libp2p+Kolme proof and governance budget closure
- Parent: #3812
- Milestone: R27.3 Live libp2p+Kolme proof and governance budgets
- Status: Implemented
- Priority: P1

## Problem Statement

R27.3 required deterministic proof that live libp2p+Kolme readiness and governance budget controls converge with release-evidence automation.

## Objective

Close the epic with explicit AC/conformance mapping across completed stories:
- `#3844` triadic local-heavy convergence/finality and signature realism coverage.
- `#3851` governance budget unification and release-evidence closure automation.

## Scope

In scope:
- Triadic topology and live-proof bundle contracts.
- Signature parity realism policy contracts.
- Unified governance budget policy and release evidence bundle automation.
- Epic-level lifecycle closure artifacts.

Out of scope:
- Mainnet cutover and unrelated protocol feature work.

## Acceptance Criteria

- AC-1: Triadic local-heavy libp2p+Kolme validation lanes remain deterministic.
- AC-2: Signature realism/parity policy contracts remain deterministic and fail closed.
- AC-3: Budget governance and release-evidence bundle gates remain deterministic and fail closed.
- AC-4: Epic-level conformance evidence remains auditable and passing.

## Conformance Cases

- C-01 (AC-1): `bash scripts/kolme/test_run_triadic_devnet_smoke_contract_lane.sh` passes.
- C-02 (AC-2): `bash scripts/kolme/test_run_signature_parity_contract_lane.sh` passes.
- C-03 (AC-3): `bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh` and `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh` pass.
- C-04 (AC-4): consolidated checks above pass in epic closure run.

## Success Metrics

- R27.3 readiness and governance surfaces remain deterministic, fail-closed, and traceable at epic level.
