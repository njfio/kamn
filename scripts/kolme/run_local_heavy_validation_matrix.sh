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
RUNTIME_COMMIT_LIVE_SUMMARY_REPORT="/tmp/kolme-local-runtime-commit-live-summary.json"
RUNTIME_COMMIT_LIVE_POLICY_REPORT="/tmp/kolme-local-runtime-commit-live-policy.json"
NATIVE_API_PARITY_SUMMARY_REPORT="/tmp/kolme-local-native-api-parity-live-proof-summary.json"
NATIVE_API_PARITY_POLICY_REPORT="/tmp/kolme-local-native-api-parity-live-proof-policy.json"
REAL_NODE_RUNTIME_INTEGRATION_REPORT="/tmp/kolme-local-kamn-live-runtime-integration-summary.json"
REAL_NODE_RUNTIME_INTEGRATION_POLICY_REPORT="/tmp/kolme-local-kamn-live-runtime-real-node-policy.json"

RUNTIME_COMMIT_FINALITY_LANE_MAX_SECONDS=120
RUNTIME_COMMIT_FINALITY_CHECK_MAX_SECONDS=15
NATIVE_API_PARITY_MAX_SECONDS=180
REAL_NODE_INTEGRATION_MAX_SECONDS=210
REAL_NODE_RUNTIME_COMMIT_MAX_SECONDS=30
REAL_NODE_RUNTIME_COMMIT_FINALITY_MAX_SECONDS=15

declare -a COMMANDS=(
  "bash scripts/kolme/run_local_bootstrap_health_checks.sh --mode run --output-json $BOOTSTRAP_REPORT"
  "bash scripts/kolme/run_version_compatibility_replay_deep_lane.sh --output-json $DEEP_REPORT"
  "bash scripts/kolme/run_local_kolme_fork_rust_test_matrix_contract_lane.sh --output-json $FORK_RUST_MATRIX_REPORT --policy-output-json $FORK_RUST_MATRIX_POLICY_REPORT"
  "bash scripts/kolme/run_local_kolme_live_api_conformance_contract_lane.sh --output-json $LIVE_API_CONFORMANCE_REPORT --policy-output-json $LIVE_API_CONFORMANCE_POLICY_REPORT"
  "bash scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh --output-json $RUNTIME_COMMIT_LIVE_SUMMARY_REPORT --policy-output-json $RUNTIME_COMMIT_LIVE_POLICY_REPORT --max-seconds $RUNTIME_COMMIT_FINALITY_LANE_MAX_SECONDS --finality-max-seconds $RUNTIME_COMMIT_FINALITY_CHECK_MAX_SECONDS --require-non-synthetic-run-evidence --require-native-payload-evidence"
  "bash scripts/kolme/run_local_native_api_parity_live_proof_contract_lane.sh --output-json $NATIVE_API_PARITY_SUMMARY_REPORT --policy-output-json $NATIVE_API_PARITY_POLICY_REPORT --max-seconds $NATIVE_API_PARITY_MAX_SECONDS"
  "bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode dry-run --runtime-profile real-node --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider --max-seconds $REAL_NODE_INTEGRATION_MAX_SECONDS --runtime-commit-max-seconds $REAL_NODE_RUNTIME_COMMIT_MAX_SECONDS --runtime-commit-finality-max-seconds $REAL_NODE_RUNTIME_COMMIT_FINALITY_MAX_SECONDS --runtime-commit-live-summary $RUNTIME_COMMIT_LIVE_SUMMARY_REPORT --runtime-commit-live-policy-report $RUNTIME_COMMIT_LIVE_POLICY_REPORT --output-json $REAL_NODE_RUNTIME_INTEGRATION_REPORT"
  "python3 scripts/kolme/check_local_kamn_live_runtime_real_node_profile_policy.py --report-file $REAL_NODE_RUNTIME_INTEGRATION_REPORT --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --require-non-synthetic-run-evidence --output-json $REAL_NODE_RUNTIME_INTEGRATION_POLICY_REPORT"
)

declare -a ARTIFACTS=(
  "$BOOTSTRAP_REPORT"
  "$DEEP_REPORT"
  "$FORK_RUST_MATRIX_REPORT"
  "$FORK_RUST_MATRIX_POLICY_REPORT"
  "$LIVE_API_CONFORMANCE_REPORT"
  "$LIVE_API_CONFORMANCE_POLICY_REPORT"
  "$RUNTIME_COMMIT_LIVE_SUMMARY_REPORT"
  "$RUNTIME_COMMIT_LIVE_POLICY_REPORT"
  "$NATIVE_API_PARITY_SUMMARY_REPORT"
  "$NATIVE_API_PARITY_POLICY_REPORT"
  "$REAL_NODE_RUNTIME_INTEGRATION_REPORT"
  "$REAL_NODE_RUNTIME_INTEGRATION_POLICY_REPORT"
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
