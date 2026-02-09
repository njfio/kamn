#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
POLICY_CHECKER="$ROOT_DIR/scripts/kolme/check_runtime_commit_replay_policy.py"
MATRIX_RUNNER="$ROOT_DIR/scripts/kolme/run_runtime_commit_replay_tamper_matrix.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/kolme_commit/runtime_commit_replay_tamper_cases.json"
ROADMAP_DOC="$ROOT_DIR/docs/planning/kolme-integration-roadmap.md"
GONOGO_DOC="$ROOT_DIR/docs/foundation/release-gonogo-checklist.md"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected runtime commit replay policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$MATRIX_RUNNER" ]; then
  echo "expected runtime commit replay matrix runner to be executable" >&2
  exit 1
fi

if [ ! -f "$FIXTURE_FILE" ]; then
  echo "expected runtime commit replay fixture file to exist" >&2
  exit 1
fi

if [ ! -f "$ROADMAP_DOC" ] || [ ! -f "$GONOGO_DOC" ]; then
  echo "expected Kolme roadmap and release go/no-go docs to exist" >&2
  exit 1
fi

start_epoch="$(date +%s)"

go_output="$(
  python3 "$POLICY_CHECKER" \
    --operation-id "op-go-lane-001" \
    --idempotency-key "kolme-runtime-commit:op-go-lane-001:state:agent:1:12" \
    --receipt-provider "kolme-local" \
    --expected-receipt-provider "kolme-local" \
    --receipt-commit-id "kolme-commit:op-go-lane-001:agent:1:12" \
    --expected-receipt-commit-id "kolme-commit:op-go-lane-001:agent:1:12" \
    --nonce-monotonic true \
    --replay-detected false \
    --payload-hash-match true \
    --receipt-finality FINAL \
    --ci-fast-gate PASS \
    --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$go_output" | grep -q '^final_decision=GO$'; then
  echo "expected GO replay policy case to produce GO" >&2
  exit 1
fi

set +e
no_go_output="$(
  python3 "$POLICY_CHECKER" \
    --operation-id "op-no-go-lane-001" \
    --idempotency-key "kolme-runtime-commit:op-no-go-lane-001:state:agent:2:12" \
    --receipt-provider "kolme-local" \
    --expected-receipt-provider "kolme-local" \
    --receipt-commit-id "kolme-commit:op-no-go-lane-001:agent:2:12" \
    --expected-receipt-commit-id "kolme-commit:op-no-go-lane-001:agent:2:12" \
    --nonce-monotonic false \
    --replay-detected true \
    --payload-hash-match true \
    --receipt-finality FINAL \
    --ci-fast-gate PASS \
    --output-json "$TMP_REPORT" 2>&1
)"
no_go_code=$?
set -e
if [ "$no_go_code" -eq 0 ]; then
  echo "expected replay-detected policy case to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$no_go_output" | grep -q '^final_decision=NO-GO$'; then
  echo "expected replay-detected policy case to produce NO-GO" >&2
  exit 1
fi

matrix_output="$(
  python3 "$MATRIX_RUNNER" \
    --fixture "$FIXTURE_FILE" \
    --max-cases 3 \
    --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$matrix_output" | grep -q '^status=pass;'; then
  echo "expected runtime commit replay matrix to pass for fixture cases" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("schema_version") != "kamn.kolme.runtime-commit-replay-matrix.v1":
    raise SystemExit("unexpected runtime commit replay matrix report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected runtime commit replay matrix report to pass")
cases = payload.get("cases", [])
if not any(case.get("case_id") == "no_go_replay_detected" for case in cases):
    raise SystemExit("expected no_go_replay_detected case in runtime commit replay matrix report")
PY

if ! grep -q "check_runtime_commit_replay_policy.py" "$ROADMAP_DOC"; then
  echo "expected Kolme integration roadmap to reference runtime commit replay policy checker command" >&2
  exit 1
fi

if ! grep -q "run_runtime_commit_replay_tamper_matrix.py" "$ROADMAP_DOC"; then
  echo "expected Kolme integration roadmap to reference runtime commit replay matrix command" >&2
  exit 1
fi

if ! grep -q "run_runtime_commit_replay_contract_lane.sh" "$ROADMAP_DOC"; then
  echo "expected Kolme integration roadmap to reference runtime commit replay contract lane command" >&2
  exit 1
fi

if ! grep -q "run_runtime_commit_replay_tamper_matrix.py" "$GONOGO_DOC"; then
  echo "expected release go/no-go doc to reference runtime commit replay matrix command" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt 60 ]; then
  echo "Kolme runtime commit replay contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "Kolme runtime commit replay contract lane tests passed."
