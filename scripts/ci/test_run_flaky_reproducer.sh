#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/ci/run_flaky_reproducer.sh"

if [ ! -x "$SCRIPT" ]; then
  echo "expected flaky reproducer script to be executable" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

stable_artifact_dir="$TMP_DIR/stable-artifacts"
stable_report="$TMP_DIR/stable-report.json"
stable_output="$(
  bash "$SCRIPT" \
    --seed 17 \
    --attempts 3 \
    --max-seconds 30 \
    --artifact-dir "$stable_artifact_dir" \
    --label stable-seed-check \
    --output-json "$stable_report" \
    -- bash -c 'printf "seed=%s attempt=%s\n" "$KAMN_FLAKY_REPRODUCER_SEED" "$KAMN_FLAKY_REPRODUCER_ATTEMPT"; exit 0'
)"
if ! printf '%s\n' "$stable_output" | grep -q '^flaky_reproducer_status=pass$'; then
  echo "expected stable flaky reproducer run status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$stable_output" | grep -q '^flaky_reproducer_reason_code=stable_success$'; then
  echo "expected stable flaky reproducer reason marker" >&2
  exit 1
fi

python3 - "$stable_report" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.ci.flaky-reproducer-report.v1":
    raise SystemExit("unexpected flaky reproducer schema")
if report.get("status") != "pass":
    raise SystemExit("expected stable reproducer status=pass")
if report.get("final_decision") != "GO":
    raise SystemExit("expected stable reproducer final_decision=GO")
if report.get("seed") != 17:
    raise SystemExit("expected stable reproducer seed=17")
attempts = report.get("attempt_results")
if not isinstance(attempts, list) or len(attempts) != 3:
    raise SystemExit("expected three stable attempt results")
if any(item.get("status") != "pass" for item in attempts):
    raise SystemExit("expected all stable attempt results to pass")
PY

flaky_artifact_dir="$TMP_DIR/flaky-artifacts"
flaky_report="$TMP_DIR/flaky-report.json"
set +e
flaky_output="$(
  bash "$SCRIPT" \
    --seed 17 \
    --attempts 3 \
    --max-seconds 30 \
    --artifact-dir "$flaky_artifact_dir" \
    --label flaky-seed-check \
    --output-json "$flaky_report" \
    -- bash -c 'if [ "$KAMN_FLAKY_REPRODUCER_ATTEMPT" -eq 2 ]; then printf "flaky-fail\n" >&2; exit 1; fi; printf "flaky-pass\n"; exit 0' 2>&1
)"
flaky_code=$?
set -e
if [ "$flaky_code" -eq 0 ]; then
  echo "expected flaky reproducer run to fail closed when mixed outcomes are observed" >&2
  exit 1
fi
if ! printf '%s\n' "$flaky_output" | grep -q '^flaky_reproducer_reason_code=flaky_pattern_observed$'; then
  echo "expected flaky reproducer mixed-outcome reason marker" >&2
  exit 1
fi

python3 - "$flaky_report" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.ci.flaky-reproducer-report.v1":
    raise SystemExit("unexpected flaky reproducer schema")
if report.get("status") != "fail":
    raise SystemExit("expected flaky reproducer status=fail")
if report.get("final_decision") != "NO-GO":
    raise SystemExit("expected flaky reproducer final_decision=NO-GO")
if report.get("reason_code") != "flaky_pattern_observed":
    raise SystemExit("expected flaky reproducer reason_code=flaky_pattern_observed")
attempts = report.get("attempt_results")
if not isinstance(attempts, list) or len(attempts) != 3:
    raise SystemExit("expected three flaky attempt results")
statuses = [item.get("status") for item in attempts]
if statuses.count("pass") == 0 or statuses.count("fail") == 0:
    raise SystemExit("expected mixed flaky attempt outcomes")
PY

set +e
invalid_attempt_output="$(
  bash "$SCRIPT" \
    --seed 17 \
    --attempts 0 \
    --max-seconds 30 \
    --artifact-dir "$TMP_DIR/invalid-artifacts" \
    --label invalid-attempt \
    --output-json "$TMP_DIR/invalid-report.json" \
    -- bash -c 'exit 0' 2>&1
)"
invalid_attempt_code=$?
set -e
if [ "$invalid_attempt_code" -eq 0 ]; then
  echo "expected attempts=0 to fail argument validation" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_attempt_output" | grep -q 'attempts must be greater than zero'; then
  echo "expected deterministic attempts validation failure marker" >&2
  exit 1
fi

echo "run_flaky_reproducer tests passed."
