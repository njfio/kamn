# Tasks: Issue 6209 - Correct FNV-1a Ordering in Name-Seed Derivation

- Issue: #6209
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add regression test proving hash-round ordering is XOR-then-multiply and not multiply-then-XOR.
- [x] T2 (GREEN): extract explicit FNV-1a round helper and route name-seed derivation through it.
- [x] T3 (REGRESSION): preserve deterministic identity key test vectors.
- [x] T4 (VERIFY): run `cargo fmt --check`, `cargo clippy -p kamn-agent-lib -- -D warnings`, and `cargo test -p kamn-agent-lib`.
