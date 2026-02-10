#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ZK_WITNESS_MUTATION_LANE="$ROOT_DIR/scripts/runtime/run_zk_witness_mutation_contract_lane.sh"
ZK_WITNESS_MUTATION_DEEP_LANE="$ROOT_DIR/scripts/runtime/run_zk_witness_mutation_deep_lane.sh"
cd "$ROOT_DIR"

# Keep deterministic mutation coverage bounded for fast, low-cost PR validation.
cargo test -p kamn-core --test message_envelope_fuzz_smoke fuzz_smoke_envelope_mutation_lane_is_panic_free_and_deterministic -- --exact >/dev/null
cargo test -p kamn-core --test message_envelope_fuzz_smoke functional_envelope_mutation_suite_covers_malformed_truncated_and_tampered_classes -- --exact >/dev/null
cargo test -p kamn-core --test message_envelope_fuzz_smoke integration_envelope_mutation_fail_closed_reasons_are_explicit_and_deterministic -- --exact >/dev/null
cargo test -p kamn-core --test message_envelope_fuzz_smoke regression_envelope_mutation_reason_signatures_remain_stable -- --exact >/dev/null
cargo test -p kamn-core --test message_envelope_fuzz_smoke performance_envelope_mutation_contract_lane_stays_within_budget -- --exact >/dev/null

cargo test -p kamn-core --test did_fuzz_smoke fuzz_smoke_did_parse_mutations_are_panic_free_and_deterministic -- --exact >/dev/null
cargo test -p kamn-core --test did_fuzz_smoke functional_did_mutation_suite_covers_normalization_encoding_and_method_mismatch_classes -- --exact >/dev/null
cargo test -p kamn-core --test did_fuzz_smoke integration_did_mutation_fail_closed_reasons_are_explicit_and_deterministic -- --exact >/dev/null
cargo test -p kamn-core --test did_fuzz_smoke regression_did_mutation_reason_signatures_remain_stable -- --exact >/dev/null
cargo test -p kamn-core --test did_fuzz_smoke performance_did_mutation_contract_lane_stays_within_budget -- --exact >/dev/null

if [ "${KAMN_RUNTIME_ZK_WITNESS_MUTATION_DEEP:-false}" = "true" ]; then
  bash "$ZK_WITNESS_MUTATION_DEEP_LANE" >/dev/null
else
  bash "$ZK_WITNESS_MUTATION_LANE" >/dev/null
fi

cargo test -p kamn-core --test runtime_network_docs doc_contains_mutation_fail_closed_contract_rules -- --exact >/dev/null

echo "runtime input mutation contract lane tests passed."
