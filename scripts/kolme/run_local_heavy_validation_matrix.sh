#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SUMMARY_HELPER="$ROOT_DIR/scripts/framework/generate_local_lane_summary.py"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-heavy-validation-summary.json"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --mode)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --mode" >&2
        exit 1
      fi
      MODE="$2"
      shift 2
      ;;
    --output-json)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --output-json" >&2
        exit 1
      fi
      OUTPUT_JSON="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_heavy_validation_matrix.sh [--mode dry-run|run] [--output-json <path>]

Modes:
  dry-run  Print and record the heavy validation command matrix without executing commands.
  run      Execute heavy local validation commands. Requires KAMN_KOLME_LOCAL_HEAVY=1.
USAGE
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [ "$MODE" != "dry-run" ] && [ "$MODE" != "run" ]; then
  echo "mode must be one of: dry-run, run" >&2
  exit 1
fi

if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

if [ ! -x "$SUMMARY_HELPER" ]; then
  echo "expected shared local-lane summary helper to be executable" >&2
  exit 1
fi

if [ "$MODE" = "run" ]; then
  "$LOCAL_HEAVY_GUARD"
fi

reason_code="dry_run_no_commands_executed"
if [ "$MODE" = "run" ]; then
  reason_code="local_heavy_validation_passed"
fi

BOOTSTRAP_REPORT="/tmp/kolme-local-bootstrap-summary.json"
DEEP_REPORT="/tmp/kolme-version-compatibility-report.json"
FORK_RUST_MATRIX_REPORT="/tmp/kolme-local-fork-rust-test-matrix-summary.json"
FORK_RUST_MATRIX_POLICY_REPORT="/tmp/kolme-local-fork-rust-test-matrix-policy.json"
LIVE_API_CONFORMANCE_REPORT="/tmp/kolme-local-live-api-conformance-summary.json"
LIVE_API_CONFORMANCE_POLICY_REPORT="/tmp/kolme-local-live-api-conformance-policy.json"

declare -a COMMANDS=(
  "bash scripts/kolme/run_local_bootstrap_health_checks.sh --mode run --output-json $BOOTSTRAP_REPORT"
  "bash scripts/kolme/run_version_compatibility_replay_deep_lane.sh --output-json $DEEP_REPORT"
  "bash scripts/kolme/run_local_kolme_fork_rust_test_matrix_contract_lane.sh --output-json $FORK_RUST_MATRIX_REPORT --policy-output-json $FORK_RUST_MATRIX_POLICY_REPORT"
  "bash scripts/kolme/run_local_kolme_live_api_conformance_contract_lane.sh --output-json $LIVE_API_CONFORMANCE_REPORT --policy-output-json $LIVE_API_CONFORMANCE_POLICY_REPORT"
)

declare -a ARTIFACTS=(
  "$BOOTSTRAP_REPORT"
  "$DEEP_REPORT"
  "$FORK_RUST_MATRIX_REPORT"
  "$FORK_RUST_MATRIX_POLICY_REPORT"
  "$LIVE_API_CONFORMANCE_REPORT"
  "$LIVE_API_CONFORMANCE_POLICY_REPORT"
)

if [ "$MODE" = "run" ]; then
  pushd "$ROOT_DIR" >/dev/null
  for command in "${COMMANDS[@]}"; do
    eval "$command"
  done
  popd >/dev/null
fi

COMMAND_FILE="$(mktemp)"
ARTIFACT_FILE="$(mktemp)"
trap 'rm -f "$COMMAND_FILE" "$ARTIFACT_FILE"' EXIT

for command in "${COMMANDS[@]}"; do
  printf '%s\n' "$command" >>"$COMMAND_FILE"
done

for artifact in "${ARTIFACTS[@]}"; do
  printf '%s\n' "$artifact" >>"$ARTIFACT_FILE"
done

python3 "$SUMMARY_HELPER" \
  --schema-version "kamn.kolme.local-heavy-validation-summary.v1" \
  --summary-type commands \
  --mode "$MODE" \
  --status ok \
  --reason-code "$reason_code" \
  --local-only-enforced true \
  --commands-file "$COMMAND_FILE" \
  --artifacts-file "$ARTIFACT_FILE" \
  --output-json "$OUTPUT_JSON"

echo "status=ok"
echo "matrix_mode=$MODE"
echo "reason_code=$reason_code"
echo "local_only_enforced=true"
echo "summary_file=$(realpath "$OUTPUT_JSON")"
