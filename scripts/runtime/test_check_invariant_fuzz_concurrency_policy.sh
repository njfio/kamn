#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/runtime/check_invariant_fuzz_concurrency_policy.sh"
LANE_SCRIPT="$ROOT_DIR/scripts/runtime/run_invariant_fuzz_concurrency_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected invariant/fuzz/concurrency policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected invariant/fuzz/concurrency contract lane script to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/invariant-fuzz-concurrency-contract-report.json"
bash "$LANE_SCRIPT" --output-json "$report_file" >/dev/null

go_output="$(bash "$CHECKER" --report-file "$report_file")"
if ! printf '%s\n' "$go_output" | grep -Fq "status=ok"; then
  echo "expected invariant/fuzz/concurrency policy checker success status" >&2
  exit 1
fi

tampered_report="$TMP_DIR/invariant-fuzz-concurrency-contract-report.tampered.json"
cp "$report_file" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["concurrency_replay_artifact_key"] = "tampered_concurrency_artifact_key"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(bash "$CHECKER" --report-file "$tampered_report" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered invariant/fuzz/concurrency report to fail policy checker" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -Fq "concurrency_replay_artifact_key mismatch"; then
  echo "expected explicit concurrency replay artifact mismatch policy error" >&2
  exit 1
fi

# Regression: #1363
if ! printf '%s\n' "$tampered_output" | grep -Fq "concurrency_mutation_replay:v1"; then
  echo "expected required concurrency replay artifact key marker in policy regression path" >&2
  exit 1
fi

echo "invariant/fuzz/concurrency evidence policy checker tests passed."
