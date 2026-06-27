# 7029-repair-kamn-core-rustdoc-bridge-adapter-link

## Objective
Restore the Fast Gate kamn-core rustdoc artifact contract lane by repairing
broken intra-doc links from bridge adapter envelope docs to the real
`BridgeAdapter` trait.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-core/src/bridge_adapter/models/envelopes.rs`
  - `crates/kamn-core/src/bridge_adapter/models/adapters.rs`
  - `scripts/framework/manifests/ci_kamn_core_rustdoc_artifact_contract_lane.json`
  - `scripts/ci/check_kamn_core_rustdoc_artifact_policy.sh`
- Outputs:
  - `RUSTDOCFLAGS=-D warnings cargo doc -p kamn-core --no-deps` succeeds.
  - The kamn-core rustdoc artifact contract lane succeeds and produces a policy
    report accepted by the policy checker.
  - Bridge adapter envelope docs keep a real link to `BridgeAdapter`; the link is
    not escaped into plain text.

## Boundaries/Non-goals
- Do not relax `-D warnings`, rustdoc broken-link checks, or rustdoc artifact
  policy.
- Do not remove the `BridgeAdapter` references from the public docs.
- Do not broaden this issue into missing-docs policy, bridge runtime behavior, or
  MVP demo feature work.

## Failure Modes
- `NormalizedInboundMessage` docs link to an unqualified `BridgeAdapter` symbol
  that rustdoc cannot resolve from `models::envelopes`.
- `BridgeOutboundEnvelope` docs link to an unqualified `BridgeAdapter` symbol
  that rustdoc cannot resolve from `models::envelopes`.
- A future edit escapes the link instead of preserving a valid rustdoc target.

## Acceptance Criteria
- [x] Red evidence captures the rustdoc artifact lane failing with unresolved
      links to `BridgeAdapter`.
- [x] A focused source contract fails when bridge envelope docs use an
      unqualified `BridgeAdapter` intra-doc link.
- [x] Bridge envelope docs link to the actual `BridgeAdapter` trait target.
- [x] `RUSTDOCFLAGS=-D warnings cargo doc -p kamn-core --no-deps` passes.
- [x] The kamn-core rustdoc artifact contract lane and policy checker pass.
- [x] `cargo fmt --check`, strict workspace clippy, and `make check` remain
      green.

## Files To Touch
- `crates/kamn-core/src/bridge_adapter/models/envelopes.rs`
- A focused contract test under `crates/kamn-core/tests/`
- This spec file

## Error Semantics
- Rustdoc broken intra-doc links remain hard failures under `-D warnings`.
- Rustdoc artifact policy failures remain hard failures.
- Source contract failures remain hard failures and should identify the stale
  unqualified link.

## Test Plan
- Red: run the rustdoc artifact lane and record the unresolved
  `BridgeAdapter` links.
- Red: add a source contract requiring bridge envelope docs to link to
  `crate::bridge_adapter::BridgeAdapter` and observe it fail before the doc fix.
- Green: update bridge envelope docs to use a resolvable `BridgeAdapter` link.
- Integration: rerun rustdoc, the rustdoc artifact contract lane, and its policy
  checker.

## Completion Evidence
- Red: `bash scripts/framework/run_manifest_lane.sh --manifest scripts/framework/manifests/ci_kamn_core_rustdoc_artifact_contract_lane.json --phase contract --artifact-dir <tmp> --output-json /tmp/kamn-core-rustdoc-artifact-report-before-7029.json`
  failed with unresolved links to `BridgeAdapter` in
  `crates/kamn-core/src/bridge_adapter/models/envelopes.rs`.
- Red: `cargo test -p kamn-core --test rustdoc_bridge_adapter_link_contract`
  failed with `bridge envelope docs must not use unresolved BridgeAdapter link
  sentence`.
- Green: `cargo test -p kamn-core --test rustdoc_bridge_adapter_link_contract`
  passed with 1 test.
- Green: `RUSTDOCFLAGS=-D warnings cargo doc -p kamn-core --no-deps` passed.
- Integration: `bash scripts/framework/run_manifest_lane.sh --manifest scripts/framework/manifests/ci_kamn_core_rustdoc_artifact_contract_lane.json --phase contract --artifact-dir <tmp> --output-json /tmp/kamn-core-rustdoc-artifact-report-after-7029.json`
  passed with `kamn_core_rustdoc_artifact_status=pass`.
- Integration: `bash scripts/ci/check_kamn_core_rustdoc_artifact_policy.sh --report-file /tmp/kamn-core-rustdoc-artifact-report-after-7029.json`
  passed with `kamn_core_rustdoc_artifact_policy=ok`.
- Integration: `bash scripts/ci/test_run_kamn_core_rustdoc_artifact_contract_lane.sh`
  passed.
- Integration: `bash scripts/ci/test_check_kamn_core_rustdoc_artifact_policy.sh`
  passed.
- Full gates: `cargo fmt --check` passed.
- Full gates: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  passed in 13m 59s.
- Full gates: `make check` passed.
- Governance: `python3 scripts/ci/check_governance_feature_commit_ratio.py --repo-root . --base-sha d2c2fe1b901a1d53ea419f31778e1d836f2b1323 --head-sha HEAD --window-size 50 --max-governance-ratio 0.20 --output-json /tmp/kamn-governance-feature-commit-ratio-after-7029.json`
  passed with `governance_ratio=0.2` and `feature_ratio=0.8`.
- Telemetry: `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --threshold-file .ci/shell-rust-ratio-guardrail.env --output-json /tmp/kamn-shell-rust-ratio-after-7029.json`
  passed with `shell_to_rust_ratio=0.421487`.
- Telemetry: `bash scripts/ci/collect_shell_rust_loc_telemetry.sh --output-json /tmp/kamn-shell-rust-loc-telemetry-after-7029.json`
  passed with `delta_shell_line_total=229`, `delta_rust_line_total=166408`,
  and `delta_shell_to_rust_ratio=-0.181627`.

## Shell-Surface Metrics
- `shell_loc_delta_estimate: +20`
- `rust_loc_delta_estimate: +80`
- `shell_to_rust_ratio_delta_estimate: -0.0001`
- `shell_surface_mitigation_issue: #7029`
- `shell_loc_delta_actual: +229`
- `rust_loc_delta_actual: +166408`
- `shell_to_rust_ratio_delta_actual: -0.181627`
- `shell_surface_ratio_target_status: improved`
