#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_self_test_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_fork_self_test_policy.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"

OUTPUT_JSON="/tmp/kolme-local-fork-self-test-summary.json"
POLICY_OUTPUT_JSON="/tmp/kolme-local-fork-self-test-policy.json"
CHECKOUT_PATH="/tmp/kolme_fork"
EXPECTED_REMOTE_URL="https://github.com/njfio/kolme_fork.git"
EXPECTED_REF="refs/heads/main"
MAX_SECONDS="${KAMN_KOLME_LOCAL_FORK_SELF_TEST_CONTRACT_MAX_SECONDS:-120}"
MATRIX_MAX_SECONDS="${KAMN_KOLME_LOCAL_FORK_SELF_TEST_CONTRACT_MATRIX_MAX_SECONDS:-60}"
MATRIX_CARGO_PROFILE="${KAMN_KOLME_LOCAL_FORK_SELF_TEST_CONTRACT_CARGO_PROFILE:-portable}"

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
    --expected-remote-url)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --expected-remote-url" >&2
        exit 1
      fi
      EXPECTED_REMOTE_URL="$2"
      shift 2
      ;;
    --expected-ref)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --expected-ref" >&2
        exit 1
      fi
      EXPECTED_REF="$2"
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
    --matrix-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --matrix-max-seconds" >&2
        exit 1
      fi
      MATRIX_MAX_SECONDS="$2"
      shift 2
      ;;
    --matrix-cargo-profile)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --matrix-cargo-profile" >&2
        exit 1
      fi
      MATRIX_CARGO_PROFILE="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_kolme_fork_self_test_contract_lane.sh [options]

Options:
  --output-json <path>            Self-test summary output.
  --policy-output-json <path>     Self-test policy report output.
  --checkout-path <path>          Local fork checkout path passed to dry-run runner.
  --expected-remote-url <url>     Expected checkout remote URL passed to dry-run runner.
  --expected-ref <ref>            Expected checkout ref passed to dry-run runner.
  --max-seconds <n>               Runtime budget value passed through summary metadata.
  --matrix-max-seconds <n>        Matrix runtime budget passed through summary metadata.
  --matrix-cargo-profile <value>  Matrix cargo profile passed through summary metadata.
USAGE
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

for numeric_value in "$MAX_SECONDS" "$MATRIX_MAX_SECONDS"; do
  if ! [[ "$numeric_value" =~ ^[0-9]+$ ]] || [ "$numeric_value" -le 0 ]; then
    echo "max-second arguments must be positive integers" >&2
    exit 1
  fi
done

if [ "$MATRIX_CARGO_PROFILE" != "strict" ] && [ "$MATRIX_CARGO_PROFILE" != "portable" ]; then
  echo "matrix-cargo-profile must be one of: strict, portable" >&2
  exit 1
fi

if [ ! -x "$RUNNER" ]; then
  echo "expected local fork self-test runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local fork self-test policy checker to be executable" >&2
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
  --expected-remote-url "$EXPECTED_REMOTE_URL" \
  --expected-ref "$EXPECTED_REF" \
  --max-seconds "$MAX_SECONDS" \
  --matrix-max-seconds "$MATRIX_MAX_SECONDS" \
  --matrix-cargo-profile "$MATRIX_CARGO_PROFILE" \
  --output-json "$OUTPUT_JSON" \
  >/dev/null

python3 "$CHECKER" \
  --report-file "$OUTPUT_JSON" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code dry_run_no_commands_executed \
  --output-json "$POLICY_OUTPUT_JSON" \
  >/dev/null

if ! grep -q "run_local_kolme_fork_self_test_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork self-test runner" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_fork_self_test_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork self-test policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_self_test_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork self-test contract lane" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_fork_self_test_policy.py" "$README_FILE"; then
  echo "expected README to reference local fork self-test policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_self_test_contract_lane.sh" "$README_FILE"; then
  echo "expected README to reference local fork self-test contract lane" >&2
  exit 1
fi

echo "local fork self-test contract lane tests passed."
