# Plan — Issue #4185

## Approach

1. Add `scripts/deploy/check_upgrade_rehearsal_lineage_policy.py` to evaluate milestone-review
   bundle lineage and promotion-gate mappings with deterministic reasons.
2. Integrate checker invocation and tamper fixtures into
   `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`.
3. Update release/planning docs with checker command and taxonomy markers.
4. Update Rust docs tests to enforce marker parity.

## Affected Modules

- `scripts/deploy/check_upgrade_rehearsal_lineage_policy.py` (new)
- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `docs/planning/kolme-devnet-ops.md`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`

## Risks / Mitigations

- Risk: duplicated validation logic diverges from source generator semantics.
  Mitigation: verifier consumes milestone bundle output and validates deterministic observed/contracts
  surface rather than re-implementing full generation path.

## Interfaces / Contracts

- checker output contract markers:
  - reason taxonomy version
  - reason codes csv
  - reason codes value
  - final decision/status

## ADR

- Not required.
