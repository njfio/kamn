#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
RUNNER="$ROOT_DIR/scripts/ci/run_critical_path_mutation_gate.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$RUNNER" "expected critical-path mutation gate script to be executable"

PASS_REPORT="$TMP_DIR/mutation-pass.json"
FAIL_REPORT="$TMP_DIR/mutation-fail.json"

pass_output="$(
  KAMN_MUTATION_GATE_STUB=true \
  bash "$RUNNER" \
    --output-json "$PASS_REPORT" \
    --timeout-seconds 60
)"
if ! printf '%s\n' "$pass_output" | grep -q '^status=ok$'; then
  echo "expected status=ok for stubbed mutation pass run" >&2
  printf '%s\n' "$pass_output" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^reason_codes_csv=none$'; then
  echo "expected reason_codes_csv=none for stubbed mutation pass run" >&2
  printf '%s\n' "$pass_output" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^tested_mutants=10$'; then
  echo "expected tested_mutants=10 in stubbed mutation pass run" >&2
  printf '%s\n' "$pass_output" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^caught_mutants=10$'; then
  echo "expected caught_mutants=10 in stubbed mutation pass run" >&2
  printf '%s\n' "$pass_output" >&2
  exit 1
fi
python3 - "$PASS_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.ci.critical-path-mutation-report.v1":
    raise SystemExit("unexpected schema_version")
if payload.get("slice_count") != 6:
    raise SystemExit("expected 6 mutation slices")
if payload.get("totals", {}).get("expected_mutants") != 10:
    raise SystemExit("expected expected_mutants=10")
PY

if KAMN_MUTATION_GATE_STUB=true \
  KAMN_MUTATION_GATE_STUB_FAIL_SLICE=node-signer \
  bash "$RUNNER" \
    --output-json "$FAIL_REPORT" \
    --timeout-seconds 60 \
    >"$TMP_DIR/fail.out" \
    2>"$TMP_DIR/fail.err"
then
  echo "expected stubbed mutation run to fail when a slice is forced to escape" >&2
  cat "$TMP_DIR/fail.out" >&2 || true
  cat "$TMP_DIR/fail.err" >&2 || true
  exit 1
fi
if ! grep -q '^reason_codes_csv=critical_path_mutation_slice_exit_nonzero,critical_path_mutation_slice_escape_detected$' "$TMP_DIR/fail.out"; then
  echo "expected deterministic nonzero+escape reason codes for forced-fail mutation slice" >&2
  cat "$TMP_DIR/fail.out" >&2 || true
  exit 1
fi
python3 - "$FAIL_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("final_decision") != "NO-GO":
    raise SystemExit("expected final_decision=NO-GO")
if payload.get("totals", {}).get("missed_mutants") != 1:
    raise SystemExit("expected exactly one missed mutant in forced-fail run")
PY

echo "critical-path mutation gate script tests passed."
