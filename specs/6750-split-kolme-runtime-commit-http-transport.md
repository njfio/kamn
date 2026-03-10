# 6750-split-kolme-runtime-commit-http-transport

## Objective

Split `crates/kamn-core/tests/kolme_runtime_commit_http_transport.rs` into bounded concern-based modules while preserving the real `kolme_runtime_commit_http_transport` test target and bringing the touched surface under the active size ratchet.

## Inputs/Outputs

### Inputs
- Existing test root `crates/kamn-core/tests/kolme_runtime_commit_http_transport.rs`
- Current `kamn-core` runtime commit HTTP transport test target
- Active touched-Rust size policy and current `origin/main` baseline

### Outputs
- A thin root shell file for `kolme_runtime_commit_http_transport.rs`
- Concern-based sibling modules under `crates/kamn-core/tests/kolme_runtime_commit_http_transport/`
- An external extraction contract that enforces the root-shell budget and required module layout
- Preserved green behavior for the real `kolme_runtime_commit_http_transport` test target

## Boundaries/Non-goals

- Do not change production runtime commit, HTTP transport, TLS, or provider behavior
- Do not rewrite unrelated `kamn-core` test files
- Do not modify CI policy beyond running the existing touched-Rust checker against the split write set
- Do not weaken or delete existing coverage to satisfy the split

## Failure Modes

- Extraction contract does not detect a regressed module layout or oversized root shell
- Real test target loses coverage or fails because helper wiring changes semantics
- Shared helper extraction leaves touched files or functions above active size limits
- Live-smoke gating or TLS/env handling becomes disconnected during the split

## Acceptance Criteria

- [ ] `crates/kamn-core/tests/kolme_runtime_commit_http_transport.rs` becomes a thin root shell within the active touched-file budget
- [ ] Extracted sibling modules exist for the planned concerns and each touched file remains within the active size policy
- [ ] An external extraction contract enforces the root-shell budget and required module layout
- [ ] `cargo test -p kamn-core --test kolme_runtime_commit_http_transport -- --nocapture` passes after the split
- [ ] The touched-Rust size checker returns `policy_decision=GO` for the staged write set

## Files To Touch

- `specs/6750-split-kolme-runtime-commit-http-transport.md`
- `crates/kamn-core/tests/kolme_runtime_commit_http_transport.rs`
- `crates/kamn-core/tests/kolme_runtime_commit_http_transport/**`
- `crates/kamn-core/tests/kolme_runtime_commit_http_transport_extraction_contract.rs`

## Error Semantics

- Test helpers remain fail-closed via `expect!`, `assert!`, and explicit panic paths already used by this test target
- Extraction contract failures must identify the missing module or budget regression directly
- No silent fallback helper paths are introduced during the split

## Test Plan

1. Add an external extraction contract asserting:
   - root shell budget
   - required module declarations
   - moved test bodies removed from the root
   - extracted module files exist
2. Run the extraction contract and confirm it fails on current `main`
3. Extract the root into bounded sibling modules with minimal behavior change
4. Run:
   - `cargo test -p kamn-core --test kolme_runtime_commit_http_transport -- --nocapture`
   - `cargo test -p kamn-core --test kolme_runtime_commit_http_transport_extraction_contract -- --nocapture`
   - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6748 --base-ref origin/main --output-json /tmp/6750-touched-size.json`
