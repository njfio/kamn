#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_bootstrap_health_checks.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_bootstrap_health_policy.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"

OUTPUT_JSON="/tmp/kolme-local-bootstrap-summary.json"
POLICY_OUTPUT_JSON="/tmp/kolme-local-bootstrap-policy.json"

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
Usage: run_local_bootstrap_health_checks_contract_lane.sh [options]

Options:
  --output-json <path>         Local bootstrap summary output.
  --policy-output-json <path>  Local bootstrap policy report output.
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
  echo "expected local bootstrap health-check runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local bootstrap health policy checker to be executable" >&2
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

if ! grep -q "run_local_bootstrap_health_checks.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local bootstrap runner" >&2
  exit 1
fi

if ! grep -q "check_local_bootstrap_health_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local bootstrap policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_bootstrap_health_checks_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local bootstrap contract lane" >&2
  exit 1
fi

if ! grep -q "check_local_bootstrap_health_policy.py" "$README_FILE"; then
  echo "expected README to reference local bootstrap policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_bootstrap_health_checks_contract_lane.sh" "$README_FILE"; then
  echo "expected README to reference local bootstrap contract lane" >&2
  exit 1
fi

echo "local bootstrap health-check contract lane tests passed."
