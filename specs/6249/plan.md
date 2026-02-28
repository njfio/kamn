# Issue 6249 Plan

Status: Reviewed

1. Inventory compatibility shim/facade modules in `crates/kamn-core/src` and classify each as remove-now or keep-temporary.
2. Replace shim module wrappers in `kamn-core` with direct extracted-crate re-exports in `lib.rs`.
3. Retire shim modules from primary export wiring, mark remaining module wrappers as deprecated compatibility surfaces, and adjust tests that validated shim-module paths.
4. Migrate `kamn-node` anti-spam imports to `kamn_runtime_guards::anti_spam` direct usage.
5. Update docs:
   - `docs/planning/r59-followup.md` shim inventory + keep/remove + timeline.
   - `docs/architecture/adr-003-kamn-core-wave2-shim-retirement.md` extraction boundary decision.
6. Verify with scoped tests for `kamn-core`, `kamn-runtime-guards`, and `kamn-node` service API paths.

## Risks / Mitigations
- Risk: removing module wrappers breaks path-based imports unexpectedly.
  - Mitigation: migrate in-repo imports first and keep temporary root re-exports where needed.
- Risk: compatibility contract drift between docs and code.
  - Mitigation: include explicit shim table and ADR with removal timeline.

## Interfaces / Contracts
- `crates/kamn-core/src/lib.rs` public exports.
- Extracted crates: `kamn-runtime-guards`, `kamn-live-probe-matrix`, `kamn-bridges`.
- Consumer: `crates/kamn-node/src/service_api_endpoint*.rs` anti-spam path.
