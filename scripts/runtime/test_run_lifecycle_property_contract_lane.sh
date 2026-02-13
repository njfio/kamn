#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/run_lifecycle_property_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/runtime/lifecycle_property_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/runtime_lifecycle_property_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected runtime lifecycle property contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected runtime lifecycle property shared contract module to be executable" >&2
  exit 1
fi

if ! grep -q "task_lifecycle_property_generated_sequences_preserve_transition_contracts" "$SHARED_CONTRACT"; then
  echo "expected lifecycle property contract lane to cover task lifecycle generated sequence invariants" >&2
  exit 1
fi

if ! grep -q "escrow_property_generated_action_sequences_preserve_amount_and_status_invariants" "$SHARED_CONTRACT"; then
  echo "expected lifecycle property contract lane to cover escrow lifecycle invariants" >&2
  exit 1
fi

if ! grep -q "peer_lifecycle_property_generated_event_sequences_match_transition_contract" "$SHARED_CONTRACT"; then
  echo "expected lifecycle property contract lane to cover peer lifecycle invariants" >&2
  exit 1
fi

if ! grep -q "dispute_refund_transition_contracts" "$SHARED_CONTRACT"; then
  echo "expected lifecycle property contract lane to cover dispute/refund property contracts" >&2
  exit 1
fi

if ! grep -q "integration_dispute_refund_replay_traces_are_deterministic" "$SHARED_CONTRACT"; then
  echo "expected lifecycle property contract lane to cover dispute/refund replay determinism integration test" >&2
  exit 1
fi

if ! grep -q "performance_dispute_refund_property_contract_lane_stays_within_budget" "$SHARED_CONTRACT"; then
  echo "expected lifecycle property contract lane to enforce dispute/refund property runtime budget contract" >&2
  exit 1
fi

report_file="$TMP_DIR/lifecycle-property-contract-report.json"
lane_output="$(bash "$CONTRACT_LANE" --output-json "$report_file")"
if ! printf '%s\n' "$lane_output" | grep -q "runtime lifecycle property contract lane tests passed."; then
  echo "expected runtime lifecycle property contract lane success marker" >&2
  exit 1
fi

if ! printf '%s\n' "$lane_output" | grep -q "runtime_lifecycle_property_contract_report=$report_file"; then
  echo "expected lifecycle property contract lane to emit report path marker" >&2
  exit 1
fi

python3 - "$report_file" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
assert report["schema_version"] == "kamn.runtime.lifecycle-property-contract-report.v1"
assert report["status"] == "pass"
assert report["replay_schema_version"] == "kamn.runtime.lifecycle-property-replay-metadata.v1"
assert report["replay_artifact_key"] == "lifecycle_property_replay:v1"
assert report["reason_codes"] == ["none"]
assert len(report["executed_tests"]) >= 12
assert len(report["executed_cases"]) == len(report["executed_tests"])
assert all("target" in case and "name" in case for case in report["executed_cases"])
assert report["generated_sequence_bounds"]["task"]["alphabet_size"] == 8
assert report["generated_sequence_bounds"]["task"]["max_sequence_length"] == 4
assert report["generated_sequence_bounds"]["escrow"]["alphabet_size"] == 5
assert report["generated_sequence_bounds"]["escrow"]["max_sequence_length"] == 4
assert report["generated_sequence_bounds"]["peer"]["alphabet_size"] == 6
assert report["generated_sequence_bounds"]["peer"]["max_sequence_length"] == 4
assert "integration_dispute_refund_replay_traces_are_deterministic" in report["executed_tests"]
assert "peer_lifecycle_property_sequence_replay_is_deterministic" in report["executed_tests"]
PY

if [ ! -L "$CONTRACT_LANE" ]; then
  echo "expected runtime lifecycle property wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$CONTRACT_LANE")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected runtime lifecycle property wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$CONTRACT_LANE")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected runtime lifecycle property wrapper to resolve runtime manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "lifecycle_property_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected runtime lifecycle property manifest to dispatch shared contract module" >&2
  exit 1
fi

echo "runtime lifecycle property contract lane script tests passed."
