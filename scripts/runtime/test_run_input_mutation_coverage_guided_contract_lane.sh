#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/run_input_mutation_coverage_guided_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected runtime input mutation coverage-guided contract lane script to be executable" >&2
  exit 1
fi

if ! grep -q "unit_input_mutation_coverage_guided_envelope_seed_corpus_covers_boundary_classes" "$CONTRACT_LANE"; then
  echo "expected coverage-guided lane to include envelope boundary class coverage" >&2
  exit 1
fi

if ! grep -q "unit_input_mutation_coverage_guided_did_seed_corpus_covers_boundary_classes" "$CONTRACT_LANE"; then
  echo "expected coverage-guided lane to include did boundary class coverage" >&2
  exit 1
fi

if ! grep -q "minimal_failing_seed_prefix" "$CONTRACT_LANE"; then
  echo "expected coverage-guided lane to emit minimizer contract marker" >&2
  exit 1
fi

if ! grep -q "KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_MAX_SECONDS" "$CONTRACT_LANE"; then
  echo "expected coverage-guided lane to enforce deterministic runtime budget" >&2
  exit 1
fi

report_file="$TMP_DIR/input-mutation-coverage-guided-contract-report.json"
lane_output="$(bash "$CONTRACT_LANE" --output-json "$report_file")"
if ! printf '%s\n' "$lane_output" | grep -q "runtime input mutation coverage-guided contract lane tests passed."; then
  echo "expected coverage-guided contract lane success marker" >&2
  exit 1
fi

if ! printf '%s\n' "$lane_output" | grep -q "runtime_input_mutation_coverage_guided_contract_report=$report_file"; then
  echo "expected coverage-guided contract lane report marker" >&2
  exit 1
fi

python3 - "$report_file" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
assert report["schema_version"] == "kamn.runtime.input-mutation-coverage-guided-contract-report.v1"
assert report["status"] == "pass"
assert report["target"] == "all"
assert report["replay_schema_version"] == "kamn.runtime.input-mutation-coverage-guided-replay-metadata.v1"
assert report["replay_artifact_key"] == "input_mutation_coverage_guided_replay:v1"
assert report["seed_corpus_keys"]["envelope"] == "input_mutation_coverage_guided_envelope_seed:v1"
assert report["seed_corpus_keys"]["did"] == "input_mutation_coverage_guided_did_seed:v1"
assert report["minimizer"] == "minimal_failing_seed_prefix"
assert report["reason_codes"] == ["none"]
assert report["envelope_test_count"] >= 1
assert report["did_test_count"] >= 1
assert report["shared_test_count"] >= 4
PY

envelope_report_file="$TMP_DIR/input-mutation-coverage-guided-envelope-report.json"
bash "$CONTRACT_LANE" --target envelope --output-json "$envelope_report_file" >/dev/null
python3 - "$envelope_report_file" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
assert report["target"] == "envelope"
assert report["envelope_test_count"] >= 1
assert report["did_test_count"] == 0
assert report["shared_test_count"] == 0
PY

did_report_file="$TMP_DIR/input-mutation-coverage-guided-did-report.json"
bash "$CONTRACT_LANE" --target did --output-json "$did_report_file" >/dev/null
python3 - "$did_report_file" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
assert report["target"] == "did"
assert report["envelope_test_count"] == 0
assert report["did_test_count"] >= 1
assert report["shared_test_count"] == 0
PY

echo "runtime input mutation coverage-guided contract lane script tests passed."
