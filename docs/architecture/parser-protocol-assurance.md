# Parser And Protocol Assurance

schema_version=kamn.docs.parser-protocol-assurance.v1
last_updated=2026-02-25

## Objective

Keep parser/protocol boundaries fail-closed with deterministic fuzz corpora and
property-based invariant checks.

## Fuzz Boundaries

- `fuzz/fuzz_targets/message_envelope_parser.rs`
- `fuzz/fuzz_targets/did_parser.rs`
- `fuzz/fuzz_targets/signature_profile_parser.rs`
- `fuzz/fuzz_targets/kolme_api_codec_parser.rs`

Corpus metadata is tracked in:

- `fuzz/corpus/replay-metadata/cargo-fuzz-seed-corpus-v1.json`

## Property Invariant Boundaries

- `crates/kamn-core/tests/parser_protocol_proptest_invariants.rs`
- `crates/kamn-core/tests/peer_lifecycle_proptest_invariants.rs`
- `crates/kamn-core/tests/task_escrow_proptest_invariants.rs`

## Deterministic Replay Artifacts

- `crates/kamn-core/proptest-regressions/tests/parser_protocol_proptest_invariants.txt`
- `crates/kamn-core/proptest-regressions/tests/peer_lifecycle_proptest_invariants.txt`
- `crates/kamn-core/proptest-regressions/tests/task_escrow_proptest_invariants.txt`
