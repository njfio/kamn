#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/runtime/run_concurrency_state_mutation_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/runtime/concurrency_state_mutation_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/runtime_concurrency_state_mutation_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/runtime/run_concurrency_state_mutation_deep_lane.sh"
DEEP_IMPL="$ROOT_DIR/scripts/runtime/run_concurrency_state_mutation_deep_lane_impl.sh"
DEEP_MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/runtime_concurrency_state_mutation_deep_lane.json"
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

if [ ! -x "$DEEP_IMPL" ]; then
  echo "expected runtime concurrency state mutation deep lane implementation script to be executable" >&2
  exit 1
fi

if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected runtime concurrency state mutation shared contract script to be executable" >&2
  exit 1
fi

if ! grep -q "functional_task_accept_concurrency_replay_fixture_preserves_invariants" "$SHARED_CONTRACT"; then
  echo "expected concurrency contract lane to include functional replay fixture coverage" >&2
  exit 1
fi

if ! grep -q "integration_peer_lifecycle_concurrency_replay_is_deterministic_across_rounds" "$SHARED_CONTRACT"; then
  echo "expected concurrency contract lane to include integration replay determinism coverage" >&2
  exit 1
fi

if ! grep -q "functional_escrow_dispute_refund_concurrency_replay_fixture_preserves_terminal_snapshot" "$SHARED_CONTRACT"; then
  echo "expected concurrency contract lane to include escrow dispute/refund functional replay coverage" >&2
  exit 1
fi

if ! grep -q "integration_escrow_dispute_refund_concurrency_replay_is_deterministic_across_rounds" "$SHARED_CONTRACT"; then
  echo "expected concurrency contract lane to include escrow dispute/refund integration replay determinism coverage" >&2
  exit 1
fi

if ! grep -q "regression_concurrency_accept_race_never_allows_multiple_winners" "$SHARED_CONTRACT"; then
  echo "expected concurrency contract lane to include regression winner exclusivity coverage" >&2
  exit 1
fi

if ! grep -q "regression_escrow_refund_race_never_allows_multiple_refund_winners" "$SHARED_CONTRACT"; then
  echo "expected concurrency contract lane to include escrow refund race regression coverage" >&2
  exit 1
fi

if ! grep -q "performance_concurrency_state_mutation_contract_lane_stays_within_budget" "$SHARED_CONTRACT"; then
  echo "expected concurrency contract lane to include performance budget coverage" >&2
  exit 1
fi

if ! grep -q "performance_escrow_dispute_refund_concurrency_lane_stays_within_budget" "$SHARED_CONTRACT"; then
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

if [ ! -L "$FAST_SCRIPT" ]; then
  echo "expected runtime concurrency state mutation contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$FAST_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected runtime concurrency state mutation wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$FAST_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected runtime concurrency state mutation wrapper to resolve runtime manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "concurrency_state_mutation_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected runtime concurrency state mutation manifest to dispatch shared contract module" >&2
  exit 1
fi

if [ ! -L "$DEEP_SCRIPT" ]; then
  echo "expected runtime concurrency state mutation deep lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$DEEP_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected runtime concurrency state mutation deep lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_deep_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$DEEP_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_deep_manifest" != "$DEEP_MANIFEST_FILE" ]; then
  echo "expected runtime concurrency state mutation deep lane wrapper to resolve runtime deep manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "run_concurrency_state_mutation_deep_lane_impl.sh" "$DEEP_MANIFEST_FILE"; then
  echo "expected runtime concurrency state mutation deep manifest to dispatch implementation module" >&2
  exit 1
fi

if ! grep -Fq "run_concurrency_state_mutation_contract_lane.sh" "$DEEP_IMPL"; then
  echo "expected deep lane implementation to execute concurrency contract lane baseline first" >&2
  exit 1
fi

if ! grep -q "performance_concurrency_state_mutation_deep_lane_stress -- --ignored" "$DEEP_IMPL"; then
  echo "expected deep lane implementation to execute ignored concurrency stress test" >&2
  exit 1
fi

echo "runtime concurrency state mutation contract lane script tests passed."
