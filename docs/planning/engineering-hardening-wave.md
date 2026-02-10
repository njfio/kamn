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
- Bounded rustdoc generation command (kamn-core only):
  - `RUSTDOCFLAGS="-D warnings" cargo doc -p kamn-core --no-deps`
- Rustdoc artifact contract lane and policy checker:
  - `bash scripts/ci/run_kamn_core_rustdoc_artifact_contract_lane.sh --output-json /tmp/kamn-core-rustdoc-artifact-report.json`
  - `bash scripts/ci/check_kamn_core_rustdoc_artifact_policy.sh --report-file /tmp/kamn-core-rustdoc-artifact-report.json`
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
- Architecture/runtime flow and rustdoc publication docs are required:
  - `docs/architecture/kamn-core-module-map.md`
  - `docs/developer/rustdoc-publishing.md`

## Cost and Runtime Policy

- The policy checker is shell-based (`grep`/`awk` + fixture diff) and avoids
  full Rust builds when only documentation/policy files change.
- The framework extraction pilot keeps legacy shell command surfaces stable while
  moving reusable validation logic into shared Python helpers.
  - migrated lanes: token launch handoff, token launch handoff contract lane runner, treasury disbursement approvals, treasury disbursement contract lane runner, launch canary contract lane runner, post-cutover SLO canary gate, post-cutover SLO contract lane runner, escrow settlement reconciliation, go/no-go evidence, staging rehearsal evidence, DR evidence, deployment SLO rollback policy checker, deployment SLO rollback lane runner, dashboard backend session policy checker, dashboard backend session lane runner, dashboard backend session contract lane runner, dashboard stale/error policy checker, dashboard stale/error lane runner, dashboard stale/error contract lane runner, cutover rollback evidence, channel retention/redaction, channel lifecycle contract lane runner, channel policy contract lane runner, bridge replay/redaction, bridge adapter conformance, cross-chain outbound intent, localhost bridge demo evidence, DID lifecycle operator-binding, key lifecycle invariant, group sender replay/ratchet, DIDComm envelope compatibility policy checker, A2A/MCP conformance policy checker, live-network smoke lane runner, live-network pilot artifact summary generator, live-network pilot artifact summary policy checker, live-network pilot deep lane runner, localhost signed integration harness runner, localhost signed integration contract lane runner, localhost signed integration evidence policy checker, live transport parity fast lane runner, live transport smoke parity lane runner, live transport smoke parity policy checker, live transport smoke parity contract lane runner, SDK schema compatibility, SDK schema compatibility contract lane runner, SDK example fixture drift policy checker, SDK example fixture drift contract lane runner, federated DID handshake, federated delegation settlement, SOC2 control evidence, SOC2 control evidence contract lane runner, DSAR legal-hold, DSAR legal-hold contract lane runner, governance simulation, governance simulation contract lane runner, governance lifecycle rollback policy checker, governance lifecycle rollback lane runner, governance lifecycle rollback contract lane runner, governance quorum attestation policy checker, governance quorum attestation lane runner, governance quorum attestation contract lane runner, governance stake/slash, governance stake/slash contract lane runner, reputation weighted decay, reputation dispute, reputation signal quarantine, reputation recovery reversal, frontend shell determinism matrix policy checker, frontend shell determinism matrix lane runner, frontend shell determinism matrix contract lane runner, classification redaction policy checker, classification redaction lane runner, classification redaction contract lane runner, durable guard recovery contract lane runner.
  - dashboard/compliance pilot contract wrappers now dispatch through `scripts/framework/run_manifest_lane.sh` using manifests in `scripts/framework/manifests/` for backend session/auth freshness, stale/error budget, SOC2 evidence, DSAR legal-hold, and classification/redaction.
  - governance/reputation pilot wrappers now dispatch through `scripts/framework/run_manifest_lane.sh` using manifests in `scripts/framework/manifests/` for governance simulation, governance lifecycle rollback, governance quorum replay, governance stake/slash risk, and reputation dispute.
  - lifecycle verification command and budget contracts are documented in `docs/testing/invariant-and-fuzz-strategy.md` and enforced by `scripts/runtime/run_invariant_fuzz_concurrency_contract_lane.sh`.
- CI scope routing only enables the checker for relevant files:
  - `crates/kamn-core/src/lib.rs`
  - `crates/kamn-core/tests/missing_docs_policy.rs`
  - `crates/kamn-core/tests/engineering_hardening_wave_docs.rs`
  - `fixtures/ci/kamn_core_missing_docs_allowlist.txt`
  - `scripts/ci/check_kamn_core_missing_docs_policy.sh`
  - `scripts/ci/test_check_kamn_core_missing_docs_policy.sh`
  - `scripts/ci/run_kamn_core_rustdoc_artifact_contract_lane.sh`
  - `scripts/ci/test_run_kamn_core_rustdoc_artifact_contract_lane.sh`
  - `scripts/ci/check_kamn_core_rustdoc_artifact_policy.sh`
  - `scripts/ci/test_check_kamn_core_rustdoc_artifact_policy.sh`
  - `scripts/framework/contract_framework.py`
  - `scripts/framework/contract_lane_helpers.py` (compliance/governance/reputation routing)
  - `scripts/framework/test_contract_lane_helpers.py` (compliance/governance/reputation routing)
  - `scripts/framework/lane_manifest.py`
  - `scripts/framework/run_lane_from_manifest.py`
  - `scripts/framework/run_manifest_lane.sh`
  - `scripts/framework/test_pilot_lane_manifests.py`
  - `scripts/framework/manifests/dashboard_*`
  - `scripts/framework/manifests/compliance_soc2_*`
  - `scripts/framework/manifests/compliance_dsar_*`
  - `scripts/framework/manifests/compliance_classification_redaction_*`
  - `scripts/framework/manifests/governance_simulation_*`
  - `scripts/framework/manifests/governance_lifecycle_rollback_*`
  - `scripts/framework/manifests/governance_quorum_attestation_replay_*`
  - `scripts/framework/manifests/governance_stake_slash_risk_*`
  - `scripts/framework/manifests/reputation_dispute_*`
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
  - `docs/architecture/kamn-core-module-map.md`
  - `docs/developer/rustdoc-publishing.md`
  - `docs/testing/invariant-and-fuzz-strategy.md`
  - `docs/planning/engineering-hardening-wave.md`
  - `README.md`
- Selector lock for framework helper migration:
  - `scripts/framework/contract_lane_helpers.py` and
    `scripts/framework/test_contract_lane_helpers.py` route SOC2, DSAR,
    governance simulation, governance stake/slash, and reputation dispute
    contract lanes in `scripts/ci/select_targets.sh`.
  - Broad framework fan-out remains pinned to `scripts/framework/contract_framework.py`
    and its framework test harness files.

## Regression Marker

- `Regression: #896` — protect against missing-doc policy drift and undocumented
  checker/documentation command changes.
