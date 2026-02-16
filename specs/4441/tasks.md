# Tasks: Issue #4441

Status: InProgress
Issue: #4441

## Ordered Tasks

T1 (RED):
- Add RED fixtures/assertions for:
  - tampered live milestone lineage acceptance
  - partial live evidence acceptance
  - deterministic mismatch failure surface
- Run:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- Expect RED.

T2 (Verify in GREEN):
- Re-run:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- Ensure deterministic pass with live mismatch/tamper regression coverage.

## TDD Evidence

- RED command/output: Pending
- GREEN command/output: Pending
