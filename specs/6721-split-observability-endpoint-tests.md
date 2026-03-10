# Objective

Split `crates/kamn-node/src/main_tests/observability_endpoint_tests.rs` into bounded sibling modules organized by observability endpoint concern while preserving the existing `kamn-node` main test wiring and current endpoint behavior.

# Inputs/Outputs

## Inputs
- `crates/kamn-node/src/main_tests/observability_endpoint_tests.rs` at 1564 LOC on current `origin/main`
- Existing `kamn-node` observability endpoint coverage spanning payload contract checks, runtime paths, TLS behavior, readiness semantics, stream behavior, and negative-path handling
- Existing touched-Rust size policy and `main_tests` shell budget policy

## Outputs
- `crates/kamn-node/src/main_tests/observability_endpoint_tests.rs` reduced to a bounded root or staged shell cap enforced by contract
- New bounded sibling modules under `crates/kamn-node/src/main_tests/observability_endpoint_tests/` grouped by concern
- Extracted support helpers for shared HTTP/TLS/request orchestration where needed
- Contract coverage that fails if the root file regresses above its staged cap or the extracted layout disappears
- Updated spec evidence showing focused observability endpoint coverage still runs through the real `kamn-node` test surface

# Boundaries/Non-goals

- Do not change observability endpoint runtime behavior, payload schema, or public API semantics
- Do not redesign unrelated `main_tests` domains
- Do not add new dependencies
- Do not weaken existing endpoint assertions to satisfy file-size policy

# Failure modes

- `observability_endpoint_tests.rs` remains an oversized monolith
- extracted modules are arbitrary slices instead of concern-based seams
- shared helpers remain embedded in the root file and keep it oversized
- observability endpoint runtime, TLS, or stream regression coverage is lost or disconnected from the real test wiring
- touched-Rust size policy fails on the issue write set

# Acceptance criteria

- [ ] `crates/kamn-node/src/main_tests/observability_endpoint_tests.rs` is reduced to a bounded root or staged shell cap enforced by contract
- [ ] observability endpoint tests are extracted into bounded sibling modules organized by coherent concerns
- [ ] extracted files created by this issue stay within the active touched-Rust size policy on the issue write set
- [ ] a contract test fails if the root file regresses above its staged cap or the extracted module layout disappears
- [ ] focused observability endpoint coverage still passes through the real `kamn-node` test surface after extraction
- [ ] touched-Rust size policy returns `policy_decision=GO` on the issue write set

# Files to touch

- `specs/6721-split-observability-endpoint-tests.md`
- `crates/kamn-node/src/main_tests/observability_endpoint_tests.rs`
- new files under `crates/kamn-node/src/main_tests/observability_endpoint_tests/`
- `crates/kamn-node/src/main_tests/*contract*` as needed for extraction enforcement

# Error semantics

- Preserve current hard-fail assertions for payload contract violations, TLS failures, readiness failures, stream handling, and negative-path behavior
- Contract tests fail hard with exact missing-path, staged-root-cap, and file-budget details
- No fallback to inline helper bodies or alternate observability endpoint wiring

# Test plan

1. Add a red contract asserting the extracted module layout and staged root cap.
2. Extract the root by concern, expected seams:
   - shared transport/TLS support
   - payload contract checker tests
   - runtime health/metrics/readiness tests
   - TLS and negative-path tests
   - stream/concurrency/reconnect tests
3. Run focused `kamn-node` observability endpoint tests that exercise the real endpoint path.
4. Run the extraction contract.
5. Run touched-Rust size policy on the issue write set.
