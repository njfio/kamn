#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/sdk/check_example_fixture_drift.py"
POLICY_CHECKER="$ROOT_DIR/scripts/sdk/check_example_fixture_drift_policy.sh"
FIXTURE="$ROOT_DIR/fixtures/sdk_parity/register_validation_cases.json"
SNAPSHOT="$ROOT_DIR/fixtures/sdk_parity/register_validation_snapshot.json"
PLANNING_DOC="$ROOT_DIR/docs/planning/sdk-parity-wave.md"
RUST_DOC="$ROOT_DIR/docs/foundation/rust-sdk-alpha.md"
PYTHON_DOC="$ROOT_DIR/docs/foundation/python-sdk-beta.md"
TYPESCRIPT_DOC="$ROOT_DIR/docs/foundation/typescript-sdk-beta.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

usage() {
  cat <<'EOF'
Usage:
  bash scripts/sdk/run_example_fixture_drift_contract_lane.sh \
    [--output-report <path>]
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

output_report=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-report)
      output_report="${2:-}"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument: $1"
      ;;
  esac
done

for required_exec in "$CHECKER" "$POLICY_CHECKER"; do
  if [ ! -x "$required_exec" ]; then
    fail "expected executable script '$required_exec'"
  fi
done

for required_file in "$FIXTURE" "$SNAPSHOT" "$PLANNING_DOC" "$RUST_DOC" "$PYTHON_DOC" "$TYPESCRIPT_DOC"; do
  if [ ! -f "$required_file" ]; then
    fail "expected required file '$required_file'"
  fi
done

if [[ -z "$output_report" ]]; then
  output_report="$TMP_DIR/sdk-example-fixture-drift-report.json"
fi

max_seconds="${KAMN_SDK_EXAMPLE_FIXTURE_DRIFT_MAX_SECONDS:-45}"
if [[ ! "$max_seconds" =~ ^[1-9][0-9]*$ ]]; then
  fail "KAMN_SDK_EXAMPLE_FIXTURE_DRIFT_MAX_SECONDS must be a positive integer"
fi

start_epoch="$(date +%s)"

checker_output="$(
  python3 "$CHECKER" \
    --fixture "$FIXTURE" \
    --snapshot "$SNAPSHOT" \
    --output-json "$output_report"
)"

if ! printf '%s\n' "$checker_output" | grep -q "^status=pass$"; then
  fail "expected sdk example fixture drift checker to pass in contract lane"
fi

if ! printf '%s\n' "$checker_output" | grep -q "^reason_codes=none$"; then
  fail "expected sdk example fixture drift checker reason codes to be none in contract lane"
fi

policy_output="$(bash "$POLICY_CHECKER" --report-file "$output_report")"
if ! printf '%s\n' "$policy_output" | grep -q "^status=ok$"; then
  fail "expected sdk example fixture drift policy checker status marker"
fi

if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  fail "expected sdk example fixture drift policy checker final decision to be GO"
fi

for doc in "$PLANNING_DOC" "$RUST_DOC" "$PYTHON_DOC" "$TYPESCRIPT_DOC"; do
  if ! grep -q "run_example_fixture_drift_contract_lane.sh" "$doc"; then
    fail "expected documentation '$doc' to reference sdk example fixture drift contract lane command"
  fi
  if ! grep -q "register_validation_snapshot.json" "$doc"; then
    fail "expected documentation '$doc' to reference sdk fixture snapshot path"
  fi
done

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  fail "sdk example fixture drift contract lane exceeded runtime budget: ${elapsed_seconds}s"
fi

echo "status=ok"
echo "report_file=$output_report"
echo "final_decision=GO"
echo "reason_key=sdk_example_fixture_drift_reason_codes:GO:v1"
echo "sdk example fixture drift contract lane tests passed."
