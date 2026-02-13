#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

cargo test -p kamn-core --test zk_witness_fuzz_smoke fuzz_smoke_zk_witness_mutation_lane_is_panic_free_and_deterministic -- --exact >/dev/null
cargo test -p kamn-core --test zk_witness_fuzz_smoke fuzz_smoke_zk_witness_error_corpus_covers_expected_rejection_classes -- --exact >/dev/null
cargo test -p kamn-core --test zk_witness_fuzz_smoke functional_zk_witness_mutation_suite_covers_malformed_missing_and_tampered_classes -- --exact >/dev/null
cargo test -p kamn-core --test zk_witness_fuzz_smoke integration_zk_witness_mutation_fail_closed_reasons_are_explicit_and_deterministic -- --exact >/dev/null
cargo test -p kamn-core --test zk_witness_fuzz_smoke regression_zk_witness_mutation_reason_signatures_remain_stable -- --exact >/dev/null
cargo test -p kamn-core --test zk_witness_fuzz_smoke performance_zk_witness_mutation_contract_lane_stays_within_budget -- --exact >/dev/null

echo "runtime zk witness mutation contract lane tests passed."
