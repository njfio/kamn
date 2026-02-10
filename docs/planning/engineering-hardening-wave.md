# Engineering Hardening Wave (Issues #894 / #895 / #896)

This wave focuses on low-cost, fail-closed quality contracts that keep the
default development loop green while tightening missing-doc policy controls for
`kamn-core`.

## Scope

- Keep baseline local and CI checks deterministic and reproducible.
- Enforce explicit `kamn-core` missing-doc policy allowlist drift checks.
- Keep policy checks cheap enough for docs-only and script-only changes.

## Commands

- Baseline local checks:
  - `cargo fmt --check`
  - `cargo clippy -- -D warnings`
  - `cargo test`
- Missing-doc policy contract checker:
  - `bash scripts/ci/check_kamn_core_missing_docs_policy.sh`
- Missing-doc policy checker regression tests:
  - `bash scripts/ci/test_check_kamn_core_missing_docs_policy.sh`
- CI helper regression suite:
  - `bash scripts/ci/test_ci_tools.sh`
- Contract framework helper unit tests:
  - `bash scripts/framework/test_contract_framework.sh`

## Missing-Docs Policy Contract

- `crates/kamn-core/src/lib.rs` must declare `#![warn(missing_docs)]`.
- Crate-wide `#![allow(missing_docs)]` is prohibited.
- Legacy `#[allow(missing_docs)] pub mod ...` exemptions are tracked in:
  - `fixtures/ci/kamn_core_missing_docs_allowlist.txt`
- Any allowlist drift fails closed via:
  - `scripts/ci/check_kamn_core_missing_docs_policy.sh`

## Cost and Runtime Policy

- The policy checker is shell-based (`grep`/`awk` + fixture diff) and avoids
  full Rust builds when only documentation/policy files change.
- The framework extraction pilot keeps legacy shell command surfaces stable while
  moving reusable validation logic into shared Python helpers.
  - migrated lanes: token launch handoff, treasury disbursement approvals, post-cutover SLO canary gate, escrow settlement reconciliation, cutover rollback evidence, channel retention/redaction, bridge replay/redaction, bridge adapter conformance, cross-chain outbound intent, localhost bridge demo evidence, DID lifecycle operator-binding, federated delegation settlement, SOC2 control evidence, DSAR legal-hold, governance simulation, governance stake/slash, reputation weighted decay, reputation dispute, reputation signal quarantine, reputation recovery reversal.
- CI scope routing only enables the checker for relevant files:
  - `crates/kamn-core/src/lib.rs`
  - `crates/kamn-core/tests/missing_docs_policy.rs`
  - `crates/kamn-core/tests/engineering_hardening_wave_docs.rs`
  - `fixtures/ci/kamn_core_missing_docs_allowlist.txt`
  - `scripts/ci/check_kamn_core_missing_docs_policy.sh`
  - `scripts/ci/test_check_kamn_core_missing_docs_policy.sh`
  - `scripts/framework/contract_framework.py`
  - `scripts/framework/test_contract_framework.sh`
  - `scripts/framework/test_contract_framework.py`
  - `docs/foundation/treasury-disbursement-policy.md`
  - `docs/foundation/observability-slo-dashboards.md`
  - `docs/foundation/escrow-lifecycle.md`
  - `docs/foundation/mainnet-cutover-runbook.md`
  - `docs/foundation/task-operations.md`
  - `docs/foundation/release-gonogo-checklist.md`
  - `docs/foundation/audit-export-interfaces.md`
  - `docs/foundation/governance-proposal-vote-execution.md`
  - `docs/foundation/reputation-signal-routing.md`
  - `docs/planning/engineering-hardening-wave.md`
  - `README.md`

## Regression Marker

- `Regression: #896` — protect against missing-doc policy drift and undocumented
  checker/documentation command changes.
