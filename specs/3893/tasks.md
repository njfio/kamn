# Issue #3893 Tasks

- Issue: #3893
- Status: Completed

## Ordered Tasks
- T1 (Red): added failing activation-closure docs-contract assertions in `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`.
- T2 (Green): updated `docs/plans/2026-02-14-production-service-next-steps.md` with deterministic activation-closure summary marker declarations.
- T3 (Refactor): extracted docs-contract validation into reusable shell helper `check_activation_closure_docs_contract` within the harness.
- T4 (Regression): added explicit docs-drift tamper path and deterministic failure reason assertion (`activation_closure_docs_missing_marker:<marker>`).
- T5 (Docs): documented activation closure summary marker key tuple and summary marker semantics in next-steps plan doc.
- T6 (Verify): ran:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - `cargo test -p kamn-core --test kolme_devnet_ops_docs`

## Completion Evidence
- Milestone activation closure docs and summary marker parity checks now fail closed on drift and remain deterministic across go/no-go evidence generation and docs contracts.
