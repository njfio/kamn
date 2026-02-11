#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_portability_preflight_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_fork_portability_preflight_policy.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"

OUTPUT_JSON="/tmp/kolme-local-fork-portability-preflight-summary.json"
POLICY_OUTPUT_JSON="/tmp/kolme-local-fork-portability-preflight-policy.json"
CHECKOUT_PATH="/tmp/kolme_fork"
MAX_SECONDS="${KAMN_KOLME_LOCAL_FORK_PORTABILITY_PREFLIGHT_CONTRACT_MAX_SECONDS:-300}"

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
    --checkout-path)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --checkout-path" >&2
        exit 1
      fi
      CHECKOUT_PATH="$2"
      shift 2
      ;;
    --max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --max-seconds" >&2
        exit 1
      fi
      MAX_SECONDS="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_kolme_fork_portability_preflight_contract_lane.sh [options]

Options:
  --output-json <path>         Portability preflight summary output.
  --policy-output-json <path>  Portability preflight policy report output.
  --checkout-path <path>       Local fork checkout path passed to dry-run runner.
  --max-seconds <n>            Runtime budget value passed through summary metadata.
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
  echo "max-seconds must be a positive integer" >&2
  exit 1
fi

if [ ! -x "$RUNNER" ]; then
  echo "expected local fork portability preflight runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local fork portability preflight policy checker to be executable" >&2
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
  --checkout-path "$CHECKOUT_PATH" \
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

if ! grep -q "run_local_kolme_fork_portability_preflight_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork portability preflight runner" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_fork_portability_preflight_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork portability preflight policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_portability_preflight_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork portability preflight contract lane" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_fork_portability_preflight_policy.py" "$README_FILE"; then
  echo "expected README to reference local fork portability preflight policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_portability_preflight_contract_lane.sh" "$README_FILE"; then
  echo "expected README to reference local fork portability preflight contract lane" >&2
  exit 1
fi

echo "local fork portability preflight contract lane tests passed."
