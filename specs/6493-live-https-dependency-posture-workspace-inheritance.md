## Objective
Teach the kamn-core live HTTPS dependency posture checker to evaluate workspace-inherited dependency posture correctly so the current `crates/kamn-core/Cargo.toml` passes while real posture drift still fails closed.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-core/Cargo.toml`
  - root `Cargo.toml` `[workspace.dependencies]`
  - `README.md`
  - `docs/architecture/adr-kamn-core-live-tls-transport.md`
  - `docs/ci/strategy.md`
  - `scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh`
- Outputs:
  - updated checker behavior for workspace-inherited dependencies
  - updated regression coverage for inherited posture and local drift overrides
  - passing `scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh`

## Boundaries/Non-goals
- Do not change the accepted live HTTPS dependency set.
- Do not change runtime TLS behavior or feature composition.
- Do not add dependencies.
- Do not broaden this issue into unrelated CI workflow edits.

## Failure modes
- Checker still treats workspace-inherited `rustls` posture as missing and reports `rustls_default_features_not_disabled`.
- Checker stops failing when a local manifest override re-enables `rustls` default features.
- Regression tests only pass on the repository manifest and do not cover a drift fixture.
- Documentation marker checks regress while updating the checker/test harness.

## Acceptance criteria
- [ ] Running `bash scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh` passes on the current workspace-inherited `crates/kamn-core/Cargo.toml`.
- [ ] A regression fixture that overrides local `rustls` posture to `default-features = true` fails with `reason_codes_csv=rustls_default_features_not_disabled`.
- [ ] The checker resolves workspace-inherited `rustls`, `rustls-pemfile`, and `webpki-roots` declarations without weakening existing fail-closed checks.
- [ ] `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` passes locally after the fix.

## Files to touch
- `scripts/ci/check_kamn_core_live_https_dependency_posture.py`
- `scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh`
- `specs/6493-live-https-dependency-posture-workspace-inheritance.md`

## Error semantics
- The checker must continue to exit non-zero on any posture violation.
- Violation reports must keep the existing deterministic reason taxonomy and reason code names.
- Workspace inheritance resolution errors must fail closed rather than silently defaulting to pass.

## Test plan
- Red:
  - run `bash scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh` and confirm failure on the current workspace-inherited manifest
- Green:
  - rerun `bash scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh`
  - rerun `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
- Refactor:
  - rerun the focused checker regression after simplifying any duplicated manifest-resolution logic
