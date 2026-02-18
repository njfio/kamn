# Issue #5037 Spec

- Title: Subtask: M8 crypto-shred and retention-policy legal-hold conformance suite
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Parent task `#5024` requires deterministic M8 compliance guarantees around
retention windows, legal-hold precedence, and crypto-shredding behavior. The
current suite covers core lifecycle paths but is missing explicit fail-closed
validation for duplicate wrapped-key recipient identities, which weakens CEK
integrity guarantees.

## Acceptance Criteria
- AC-1: Crypto-shred replaces wrapped keys with tombstone marker while
  preserving content/hash-chain integrity markers and stable shred reason code.
- AC-2: Retention due projection remains deterministic across classes and legal
  hold blocks due candidates/shredding until released.
- AC-3: Wrapped-key registration fails closed when duplicate recipient DIDs are
  provided.
- AC-4: Cross-owner operations remain denied fail-closed with stable reason
  markers.
- AC-5: Shell/workflow/python/template LOC remains unchanged
  (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- Add duplicate wrapped-key recipient fail-closed validation in
  `data_layer_m8_compliance_lifecycle`.
- Extend M8 conformance tests for duplicate recipient rejection.
- Validate scoped/full regression and shell guardrail evidence.

Out of scope:
- New dependencies/protocol/wire-format changes.
- CI workflow or shell-script changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Crypto-shred eligible message | Wrapped keys replaced with tombstone marker, content/hash markers preserved, shred reason marker stable |
| C-02 | AC-2 | Conformance | Mixed retention classes and legal-hold toggle path | Due candidates deterministic; legal hold blocks shred/due until released |
| C-03 | AC-3 | Conformance | Message registration with duplicate wrapped-key recipient DID | Registration fails with typed duplicate-recipient error |
| C-04 | AC-4 | Regression | Cross-owner due/shred/legal-hold operations | Owner-scope violation error with stable reason marker |
| C-05 | AC-5 | Regression | Shell/rust guardrail checks + diff audit | No shell surface growth; guardrails GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m8_compliance_lifecycle`
- `cargo test -p kamn-core`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5037.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5037.json`

## Success Metrics
- M8 suite includes deterministic duplicate wrapped-key recipient rejection.
- All mapped conformance tests pass with stable reason/error markers.
- Shell-to-Rust ratio remains in-go and shell LOC remains below hard ceiling.
