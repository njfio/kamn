# Objective

Split `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_fixtures.rs` into bounded sibling modules that preserve the existing live-postgres daemon fixture behavior while reducing the root shell under the active size policy.

# Inputs/Outputs

## Inputs
- `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_fixtures.rs` at 1590 LOC on current `origin/main`
- Existing live-postgres fixture helpers used by `daemon_tests`
- Current `kamn-node` daemon test entrypoint wiring for the live-postgres fixture surface
- Active touched-Rust size policy

## Outputs
- `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_fixtures.rs` reduced to a bounded root shell
- New bounded sibling modules under `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_fixtures/` grouped by fixture concern
- Contract coverage that fails if the fixture root shell regresses or the extracted module layout disappears
- Updated spec evidence showing the extracted live-postgres fixture surface still supports the real daemon tests and passes touched-Rust size checks

# Boundaries/Non-goals

- Do not change daemon runtime behavior
- Do not redesign live-postgres fixture semantics
- Do not weaken daemon test coverage to satisfy file-size policy
- Do not add new dependencies

# Failure modes

- `live_postgres_fixtures.rs` remains an oversized monolith
- extracted modules are arbitrary slices rather than matching the fixture concern seams
- matrix, topology, or multi-host fixture behavior is lost during extraction
- the root contract no longer enforces the fixture module layout or root shell budget
- touched-Rust size policy fails on the issue write set

# Acceptance criteria (testable booleans)

- [ ] `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_fixtures.rs` is reduced to a bounded root shell under the active size policy
- [ ] live-postgres fixture helpers are split into coherent sibling modules that reflect the existing concern seams
- [ ] extracted files added by this issue remain within the active touched-Rust size policy on the issue write set
- [ ] a contract fails if the fixture root shell regresses or the extracted module layout disappears
- [ ] daemon tests that rely on the live-postgres fixture surface continue to pass after extraction
- [ ] touched-Rust size policy returns `policy_decision=GO` on the issue write set

# Files to touch

- `specs/6731-split-live-postgres-fixtures.md`
- `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_fixtures.rs`
- new files under `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_fixtures/`
- `crates/kamn-node/tests/` extraction contract file(s) as needed

# Error semantics

- Preserve the current hard-fail assertions around live-postgres gate decisions, matrix projections, topology fingerprints, and multi-host prerequisite handling
- New extraction contracts fail with exact missing-path, missing-module-marker, or root-budget details
- No fallback path to inline fixture helpers or partial topology/multi-host coverage

# Test plan

1. Add a red extraction contract requiring a bounded live-postgres fixture root shell and a concern-based `live_postgres_fixtures/` layout.
2. Extract the fixture surface into sibling modules grouped by constants/taxonomy, gate helpers, matrix profiles, topology projections, and multi-host execution helpers.
3. Run the targeted daemon tests that depend on the live-postgres fixture surface.
4. Run the extraction contract.
5. Run the touched-Rust size policy on the issue write set.
6. Record final evidence and any deviations in this spec before opening the PR.
