#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_native_api_parity_live_proof_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_native_api_parity_live_proof_policy.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
OUTPUT_JSON="/tmp/kolme-local-native-api-parity-live-proof-summary.json"
POLICY_OUTPUT_JSON="/tmp/kolme-local-native-api-parity-live-proof-policy.json"
MAX_SECONDS="${KAMN_KOLME_LOCAL_NATIVE_API_PARITY_MAX_SECONDS:-180}"

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
Usage: run_local_native_api_parity_live_proof_contract_lane.sh [options]

Options:
  --output-json <path>         Native parity summary output.
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
  echo "KAMN_KOLME_LOCAL_NATIVE_API_PARITY_MAX_SECONDS must be a positive integer" >&2
  exit 1
fi

if [ ! -x "$RUNNER" ]; then
  echo "expected local native API parity live proof lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local native API parity live proof policy checker to be executable" >&2
  exit 1
fi

if [ ! -f "$DOC_FILE" ]; then
  echo "expected Kolme devnet ops documentation to exist" >&2
  exit 1
fi

start_epoch="$(date +%s)"

bash "$RUNNER" \
  --mode dry-run \
  --output-json "$OUTPUT_JSON" \
  --max-seconds "$MAX_SECONDS" \
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
    --nonce-command "printf 'nonce_ok\n'" \
    --broadcast-command "printf 'broadcast_ok\n'" \
    --finality-command "printf 'finality_ok\n'" \
    --max-seconds "$MAX_SECONDS" \
    --output-json "$OUTPUT_JSON" \
    >/dev/null

python3 "$CHECKER" \
  --report-file "$OUTPUT_JSON" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code native_parity_live_proof_passed \
  --output-json "$POLICY_OUTPUT_JSON" \
  >/dev/null

if ! grep -q "run_local_native_api_parity_live_proof_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local native API parity live proof lane runner" >&2
  exit 1
fi

if ! grep -q "check_local_native_api_parity_live_proof_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local native API parity live proof policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_native_api_parity_live_proof_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local native API parity live proof contract lane" >&2
  exit 1
fi

if ! grep -q "Regression: #1465" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include native parity live proof regression marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$MAX_SECONDS" ]; then
  echo "local native API parity live proof contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "local native API parity live proof contract lane tests passed."
