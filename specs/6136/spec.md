# Spec: Issue #6136 - Document payload-hash length identity decision

Status: Accepted
Issue: #6136
Milestone: r68-r59-swarm-remediation-and-full-gap-closure

## Problem Statement
R59 S-13 called out that Kolme runtime request identity currently uses `payload_hash.len()` while parameters are named `payload_hash`, which can be misread as value-based hashing. The current compatibility behavior is intentional (`Regression: #1777`) but undocumented in canonical runtime commit docs.

## Scope
In scope:
- Add explicit documentation in `docs/foundation/kolme-runtime-commit-client.md` describing current length-based identity behavior.
- Document the compatibility rationale and collision caveat.
- Add contract coverage ensuring the documentation marker remains present.

Out of scope:
- Changing runtime identity algorithm implementation.
- Renaming public API fields or wire fields.

## Acceptance Criteria
- AC-1: Canonical runtime commit docs explicitly state that commit/idempotency identity currently uses payload-hash length compatibility behavior.
- AC-2: Docs include rationale (`Regression: #1777`) and an explicit migration caveat.
- AC-3: Conformance test fails closed if required documentation markers are removed.

## Conformance Cases
- C-01 (AC-1): Docs include marker that `payload_hash.trim().len()` (length) is the current compatibility identity input.
- C-02 (AC-2): Docs include `Regression: #1777` and collision caveat/migration note.
- C-03 (AC-3): Contract test validates marker presence in `kolme-runtime-commit-client.md`.

## Success Metrics
- `cargo test -p kamn-kolme spec_c13_runtime_request_identity_policy_docs_describe_payload_hash_length_design_decision`
- `cargo test -p kamn-kolme runtime_request_identity_policy_contracts`
- `cargo fmt --check`
