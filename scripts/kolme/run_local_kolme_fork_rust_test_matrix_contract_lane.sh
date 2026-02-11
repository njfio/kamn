#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_rust_test_matrix_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_fork_rust_test_matrix_policy.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
OUTPUT_JSON="/tmp/kolme-local-fork-rust-test-matrix-summary.json"
POLICY_OUTPUT_JSON="/tmp/kolme-local-fork-rust-test-matrix-policy.json"
MAX_SECONDS="${KAMN_KOLME_LOCAL_FORK_RUST_MATRIX_MAX_SECONDS:-120}"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-json)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --output-json" >&2
        exit 1
      fi
      OUTPUT_JSON="$2"
      shift 2
      ;;
    --policy-output-json)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --policy-output-json" >&2
        exit 1
      fi
      POLICY_OUTPUT_JSON="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_kolme_fork_rust_test_matrix_contract_lane.sh [options]

Options:
  --output-json <path>         Matrix summary output.
  --policy-output-json <path>  Policy report output.
USAGE
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if ! [[ "$MAX_SECONDS" =~ ^[0-9]+$ ]] || [ "$MAX_SECONDS" -le 0 ]; then
  echo "KAMN_KOLME_LOCAL_FORK_RUST_MATRIX_MAX_SECONDS must be a positive integer" >&2
  exit 1
fi

if [ ! -x "$RUNNER" ]; then
  echo "expected local fork rust test matrix lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local fork rust test matrix policy checker to be executable" >&2
  exit 1
fi

if [ ! -f "$DOC_FILE" ]; then
  echo "expected Kolme devnet ops documentation to exist" >&2
  exit 1
fi

start_epoch="$(date +%s)"

bash "$RUNNER" \
  --mode dry-run \
  --checkout-path /tmp/kolme_fork \
  --max-seconds "$MAX_SECONDS" \
  --output-json "$OUTPUT_JSON" \
  >/dev/null

python3 "$CHECKER" \
  --report-file "$OUTPUT_JSON" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code dry_run_no_commands_executed \
  --output-json "$POLICY_OUTPUT_JSON" \
  >/dev/null

KAMN_KOLME_LOCAL_HEAVY=1 \
  bash "$RUNNER" \
    --mode run \
    --checkout-path /tmp/kolme_fork \
    --matrix-command "printf 'matrix_contract_ok_1\\n'" \
    --matrix-command "printf 'matrix_contract_ok_2\\n'" \
    --max-seconds "$MAX_SECONDS" \
    --output-json "$OUTPUT_JSON" \
    >/dev/null

python3 "$CHECKER" \
  --report-file "$OUTPUT_JSON" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code fork_rust_test_matrix_passed \
  --output-json "$POLICY_OUTPUT_JSON" \
  >/dev/null

if ! grep -q "run_local_kolme_fork_rust_test_matrix_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork rust test matrix lane runner" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_fork_rust_test_matrix_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork rust test matrix policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_rust_test_matrix_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork rust test matrix contract lane" >&2
  exit 1
fi

if ! grep -q "Regression: #1541" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include local fork rust test matrix regression marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$MAX_SECONDS" ]; then
  echo "local fork rust test matrix contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "local fork rust test matrix contract lane tests passed."
