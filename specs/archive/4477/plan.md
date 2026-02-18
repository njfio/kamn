# Issue #4477 Plan

- Issue: `#4477`
- Status: `Completed`

## Approach
- Extend go/no-go evidence contract generation/checking with optional TLS evidence gate convergence payload.
- Enforce deterministic TLS evidence reason taxonomy for completeness and freshness violations.
- Add shell contract coverage for valid, stale, missing, and tampered TLS evidence gate cases.
- Update release checklist docs with TLS convergence gate markers and parity test assertions.

## Affected Modules
- `scripts/deploy/gonogo_evidence_contract.py`
- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks and Mitigations
- Risk: tightening go/no-go policy could break existing generator/checker workflows.
- Mitigation: keep TLS gate input optional; enforce deterministic checks only when TLS evidence gate is present.
- Risk: non-deterministic freshness checks.
- Mitigation: compute freshness from file mtime with explicit max-age input and deterministic reason codes.

## Interface Contract
- Additive CLI options for go/no-go generator:
  - `--tls-evidence-report-file`
  - `--tls-evidence-max-age-seconds`
- Additive payload section:
  - `tls_evidence_gate`

## ADR
- Not required.
