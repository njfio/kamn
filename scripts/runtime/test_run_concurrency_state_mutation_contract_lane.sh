#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/runtime/run_concurrency_state_mutation_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/runtime/run_concurrency_state_mutation_deep_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected runtime concurrency state mutation contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected runtime concurrency state mutation deep lane script to be executable" >&2
  exit 1
fi

if ! grep -q "functional_task_accept_concurrency_replay_fixture_preserves_invariants" "$FAST_SCRIPT"; then
  echo "expected concurrency contract lane to include functional replay fixture coverage" >&2
  exit 1
fi

if ! grep -q "integration_peer_lifecycle_concurrency_replay_is_deterministic_across_rounds" "$FAST_SCRIPT"; then
  echo "expected concurrency contract lane to include integration replay determinism coverage" >&2
  exit 1
fi

if ! grep -q "functional_escrow_dispute_refund_concurrency_replay_fixture_preserves_terminal_snapshot" "$FAST_SCRIPT"; then
  echo "expected concurrency contract lane to include escrow dispute/refund functional replay coverage" >&2
  exit 1
fi

if ! grep -q "integration_escrow_dispute_refund_concurrency_replay_is_deterministic_across_rounds" "$FAST_SCRIPT"; then
  echo "expected concurrency contract lane to include escrow dispute/refund integration replay determinism coverage" >&2
  exit 1
fi

if ! grep -q "regression_concurrency_accept_race_never_allows_multiple_winners" "$FAST_SCRIPT"; then
  echo "expected concurrency contract lane to include regression winner exclusivity coverage" >&2
  exit 1
fi

if ! grep -q "regression_escrow_refund_race_never_allows_multiple_refund_winners" "$FAST_SCRIPT"; then
  echo "expected concurrency contract lane to include escrow refund race regression coverage" >&2
  exit 1
fi

if ! grep -q "performance_concurrency_state_mutation_contract_lane_stays_within_budget" "$FAST_SCRIPT"; then
  echo "expected concurrency contract lane to include performance budget coverage" >&2
  exit 1
fi

if ! grep -q "performance_escrow_dispute_refund_concurrency_lane_stays_within_budget" "$FAST_SCRIPT"; then
  echo "expected concurrency contract lane to include escrow dispute/refund performance budget coverage" >&2
  exit 1
fi

report_file="$TMP_DIR/concurrency-mutation-contract-report.json"
lane_output="$(bash "$FAST_SCRIPT" --output-json "$report_file")"
if ! printf '%s\n' "$lane_output" | grep -q "runtime concurrency state mutation contract lane tests passed."; then
  echo "expected runtime concurrency state mutation contract lane success marker" >&2
  exit 1
fi

if ! printf '%s\n' "$lane_output" | grep -q "runtime_concurrency_mutation_contract_report=$report_file"; then
  echo "expected runtime concurrency lane report path marker" >&2
  exit 1
fi

python3 - "$report_file" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
assert report["schema_version"] == "kamn.runtime.concurrency-mutation-contract-report.v1"
assert report["status"] == "pass"
assert report["replay_artifact_key"] == "concurrency_mutation_replay:v1"
assert report["reason_codes"] == ["none"]
assert report["test_count"] >= 12
PY

if ! grep -Fq "run_concurrency_state_mutation_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected deep lane to execute concurrency contract lane baseline first" >&2
  exit 1
fi

if ! grep -q "performance_concurrency_state_mutation_deep_lane_stress -- --ignored" "$DEEP_SCRIPT"; then
  echo "expected deep lane to execute ignored concurrency stress test" >&2
  exit 1
fi

echo "runtime concurrency state mutation contract lane script tests passed."
