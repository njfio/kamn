#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_heavy_validation_matrix.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_heavy_validation_matrix_policy.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"

OUTPUT_JSON="/tmp/kolme-local-heavy-validation-summary.json"
POLICY_OUTPUT_JSON="/tmp/kolme-local-heavy-validation-policy.json"

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
Usage: run_local_heavy_validation_matrix_contract_lane.sh [options]

Options:
  --output-json <path>         Local heavy matrix summary output.
  --policy-output-json <path>  Local heavy matrix policy report output.
USAGE
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [ ! -x "$RUNNER" ]; then
  echo "expected local heavy validation matrix runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local heavy validation matrix policy checker to be executable" >&2
  exit 1
fi

if [ ! -f "$DOC_FILE" ]; then
  echo "expected Kolme devnet ops documentation to exist" >&2
  exit 1
fi

if [ ! -f "$README_FILE" ]; then
  echo "expected README to exist" >&2
  exit 1
fi

bash "$RUNNER" \
  --mode dry-run \
  --output-json "$OUTPUT_JSON" \
  >/dev/null

python3 "$CHECKER" \
  --report-file "$OUTPUT_JSON" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code dry_run_no_commands_executed \
  --output-json "$POLICY_OUTPUT_JSON" \
  >/dev/null

if ! grep -q "run_local_heavy_validation_matrix.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local heavy matrix runner" >&2
  exit 1
fi

if ! grep -q "check_local_heavy_validation_matrix_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local heavy matrix policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_heavy_validation_matrix_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local heavy matrix contract lane" >&2
  exit 1
fi

if ! grep -q "check_local_heavy_validation_matrix_policy.py" "$README_FILE"; then
  echo "expected README to reference local heavy matrix policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_heavy_validation_matrix_contract_lane.sh" "$README_FILE"; then
  echo "expected README to reference local heavy matrix contract lane" >&2
  exit 1
fi

echo "local heavy validation matrix contract lane tests passed."
