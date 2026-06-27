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
- [ ] Red evidence captures the rustdoc artifact lane failing with unresolved
      links to `BridgeAdapter`.
- [ ] A focused source contract fails when bridge envelope docs use an
      unqualified `BridgeAdapter` intra-doc link.
- [ ] Bridge envelope docs link to the actual `BridgeAdapter` trait target.
- [ ] `RUSTDOCFLAGS=-D warnings cargo doc -p kamn-core --no-deps` passes.
- [ ] The kamn-core rustdoc artifact contract lane and policy checker pass.
- [ ] `cargo fmt --check`, strict workspace clippy, and `make check` remain
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

## Shell-Surface Metrics
- `shell_loc_delta_estimate: +20`
- `rust_loc_delta_estimate: +80`
- `shell_to_rust_ratio_delta_estimate: -0.0001`
- `shell_surface_mitigation_issue: #7029`
