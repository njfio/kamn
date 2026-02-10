#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
POLICY_CHECKER="$ROOT_DIR/scripts/kolme/check_nonce_broadcast_parity_policy.py"
MATRIX_RUNNER="$ROOT_DIR/scripts/kolme/run_nonce_broadcast_parity_matrix.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/kolme_commit/nonce_broadcast_parity_cases.json"
ROADMAP_DOC="$ROOT_DIR/docs/planning/kolme-integration-roadmap.md"
CI_STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected nonce/broadcast parity policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$MATRIX_RUNNER" ]; then
  echo "expected nonce/broadcast parity matrix runner to be executable" >&2
  exit 1
fi

if [ ! -f "$FIXTURE_FILE" ]; then
  echo "expected nonce/broadcast parity fixture file to exist" >&2
  exit 1
fi

if [ ! -f "$ROADMAP_DOC" ] || [ ! -f "$CI_STRATEGY_DOC" ]; then
  echo "expected Kolme roadmap and CI strategy docs to exist" >&2
  exit 1
fi

max_seconds="${KAMN_KOLME_NONCE_BROADCAST_PARITY_MAX_SECONDS:-60}"
if ! [[ "$max_seconds" =~ ^[0-9]+$ ]] || [ "$max_seconds" -le 0 ]; then
  echo "KAMN_KOLME_NONCE_BROADCAST_PARITY_MAX_SECONDS must be a positive integer" >&2
  exit 1
fi

start_epoch="$(date +%s)"

go_output="$(
  python3 "$POLICY_CHECKER" \
    --case-id "broadcast-duplicate-go-lane-001" \
    --operation "broadcast" \
    --http-status 409 \
    --nonce-value 0 \
    --broadcast-accepted false \
    --duplicate-detected true \
    --payload-valid true \
    --authorization-present true \
    --ci-fast-gate PASS \
    --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$go_output" | grep -q '^final_decision=GO$'; then
  echo "expected duplicate broadcast parity case to produce GO" >&2
  exit 1
fi

set +e
no_go_output="$(
  python3 "$POLICY_CHECKER" \
    --case-id "broadcast-unauthorized-no-go-lane-001" \
    --operation "broadcast" \
    --http-status 401 \
    --nonce-value 0 \
    --broadcast-accepted false \
    --duplicate-detected false \
    --payload-valid true \
    --authorization-present false \
    --ci-fast-gate PASS \
    --output-json "$TMP_REPORT" 2>&1
)"
no_go_code=$?
set -e
if [ "$no_go_code" -eq 0 ]; then
  echo "expected unauthorized broadcast parity case to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$no_go_output" | grep -q '^final_decision=NO-GO$'; then
  echo "expected unauthorized broadcast parity case to produce NO-GO" >&2
  exit 1
fi
if ! printf '%s\n' "$no_go_output" | grep -q 'unauthorized_status'; then
  echo "expected unauthorized broadcast parity case to emit unauthorized_status reason code" >&2
  exit 1
fi

matrix_output="$(
  python3 "$MATRIX_RUNNER" \
    --fixture "$FIXTURE_FILE" \
    --max-cases 5 \
    --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$matrix_output" | grep -q '^status=pass;'; then
  echo "expected nonce/broadcast parity matrix to pass for fixture cases" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("schema_version") != "kamn.kolme.nonce-broadcast-parity-matrix-report.v1":
    raise SystemExit("unexpected nonce/broadcast parity matrix report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected nonce/broadcast parity matrix report to pass")
cases = payload.get("cases", [])
if not any(case.get("case_id") == "broadcast_duplicate_idempotent_go" for case in cases):
    raise SystemExit("expected broadcast_duplicate_idempotent_go case in parity matrix report")
PY

if ! grep -q "check_nonce_broadcast_parity_policy.py" "$ROADMAP_DOC"; then
  echo "expected Kolme roadmap doc to reference nonce/broadcast parity policy checker command" >&2
  exit 1
fi

if ! grep -q "run_nonce_broadcast_parity_matrix.py" "$ROADMAP_DOC"; then
  echo "expected Kolme roadmap doc to reference nonce/broadcast parity matrix command" >&2
  exit 1
fi

if ! grep -q "run_nonce_broadcast_parity_contract_lane.sh" "$ROADMAP_DOC"; then
  echo "expected Kolme roadmap doc to reference nonce/broadcast parity contract lane command" >&2
  exit 1
fi

if ! grep -q "fixtures/kolme_commit/nonce_broadcast_parity_cases.json" "$ROADMAP_DOC"; then
  echo "expected Kolme roadmap doc to reference nonce/broadcast parity fixture path" >&2
  exit 1
fi

if ! grep -q "test_run_nonce_broadcast_parity_contract_lane.sh" "$CI_STRATEGY_DOC"; then
  echo "expected CI strategy doc to reference nonce/broadcast parity contract lane test command" >&2
  exit 1
fi

if ! grep -q "KAMN_KOLME_NONCE_BROADCAST_PARITY_MAX_SECONDS=60" "$CI_STRATEGY_DOC"; then
  echo "expected CI strategy doc to include nonce/broadcast parity runtime budget marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "Kolme nonce/broadcast parity contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "Kolme nonce/broadcast parity contract lane tests passed."
