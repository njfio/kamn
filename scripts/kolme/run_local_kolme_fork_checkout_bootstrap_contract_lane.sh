#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_checkout_bootstrap_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_fork_checkout_bootstrap_policy.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"

OUTPUT_JSON="/tmp/kolme-local-fork-checkout-bootstrap-summary.json"
POLICY_OUTPUT_JSON="/tmp/kolme-local-fork-checkout-bootstrap-policy.json"
MAX_SECONDS="${KAMN_KOLME_LOCAL_FORK_CHECKOUT_BOOTSTRAP_MAX_SECONDS:-120}"

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
Usage: run_local_kolme_fork_checkout_bootstrap_contract_lane.sh [options]

Options:
  --output-json <path>         Checkout bootstrap summary output.
  --policy-output-json <path>  Checkout bootstrap policy report output.
  --max-seconds <n>            Total runtime budget in seconds.
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
  echo "expected local fork checkout bootstrap lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local fork checkout bootstrap policy checker to be executable" >&2
  exit 1
fi

if [ ! -f "$DOC_FILE" ]; then
  echo "expected Kolme devnet ops documentation to exist" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
SOURCE_REPO="$TMP_DIR/source_fork"
CHECKOUT_PATH="$TMP_DIR/checkout_fork"
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p "$SOURCE_REPO"
git -C "$SOURCE_REPO" init -q
git -C "$SOURCE_REPO" checkout -q -b main
git -C "$SOURCE_REPO" config user.email "ci@example.com"
git -C "$SOURCE_REPO" config user.name "CI Runner"
cat >"$SOURCE_REPO/README.md" <<'EOF'
checkout bootstrap contract lane source fixture
EOF
git -C "$SOURCE_REPO" add README.md
git -C "$SOURCE_REPO" commit -q -m "init checkout bootstrap fixture"

start_epoch="$(date +%s)"

bash "$RUNNER" \
  --mode dry-run \
  --checkout-path "$CHECKOUT_PATH" \
  --fork-remote-url "$SOURCE_REPO" \
  --expected-remote-url "$SOURCE_REPO" \
  --expected-ref "refs/heads/main" \
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
    --checkout-path "$CHECKOUT_PATH" \
    --fork-remote-url "$SOURCE_REPO" \
    --expected-remote-url "$SOURCE_REPO" \
    --expected-ref "refs/heads/main" \
    --max-seconds "$MAX_SECONDS" \
    --output-json "$OUTPUT_JSON" \
    >/dev/null

python3 "$CHECKER" \
  --report-file "$OUTPUT_JSON" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code fork_checkout_bootstrap_passed \
  --output-json "$POLICY_OUTPUT_JSON" \
  >/dev/null

if ! grep -q "run_local_kolme_fork_checkout_bootstrap_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork checkout bootstrap runner" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_fork_checkout_bootstrap_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork checkout bootstrap policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_checkout_bootstrap_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork checkout bootstrap contract lane" >&2
  exit 1
fi

if ! grep -q "Regression: #1663" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include local fork checkout bootstrap regression marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$MAX_SECONDS" ]; then
  echo "local fork checkout bootstrap contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "local fork checkout bootstrap contract lane tests passed."
