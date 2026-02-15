#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-bootstrap-summary.json"

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
Usage: run_local_bootstrap_health_checks.sh [--mode dry-run|run] [--output-json <path>]

Modes:
  dry-run  Emit deterministic bootstrap health-check plan without executing commands.
  run      Execute deterministic bootstrap health checks. Requires KAMN_KOLME_LOCAL_HEAVY=1.
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

if [ "$MODE" = "run" ]; then
  "$LOCAL_HEAVY_GUARD"
fi

VERSION_REPORT="/tmp/kolme-bootstrap-version-report.json"
FORK_REPORT="/tmp/kolme-bootstrap-fork-compatibility-report.json"
FORK_POLICY_REPORT="/tmp/kolme-bootstrap-fork-compatibility-policy-report.json"
DEVNET_MARKERS="/tmp/kolme-bootstrap-devnet-markers.txt"
DEVNET_REPORT="/tmp/kolme-bootstrap-devnet-report.json"

declare -a CHECK_IDS=(
  "version_compatibility"
  "fork_compatibility_evidence"
  "fork_compatibility_policy"
  "triadic_devnet_smoke"
  "triadic_devnet_validate"
)

declare -a CHECK_COMMANDS=(
  "python3 scripts/kolme/validate_version_compatibility.py --kamn-version 1.1.0 --kolme-release-tag v0.15.2 --ci-fast-gate PASS --output-json $VERSION_REPORT"
  "python3 scripts/kolme/generate_fork_compatibility_evidence.py --upstream-release-tag v0.15.2 --fork-release-tag v0.15.2 --fork-repo njfio/kolme_fork --fork-ref refs/heads/main --ci-fast-gate PASS --output-json $FORK_REPORT"
  "python3 scripts/kolme/check_fork_compatibility_policy.py --report-file $FORK_REPORT --expected-upstream-release-tag v0.15.2 --expected-fork-release-tag v0.15.2 --expected-fork-repo njfio/kolme_fork --expected-final-decision GO --ci-fast-gate PASS --output-json $FORK_POLICY_REPORT"
  "bash scripts/kolme/run_triadic_devnet_smoke.sh --output-file $DEVNET_MARKERS"
  "python3 scripts/kolme/validate_triadic_devnet_smoke.py --fixture fixtures/kolme_compatibility/devnet_smoke_markers.json --marker-file $DEVNET_MARKERS --output-json $DEVNET_REPORT"
)

declare -a ARTIFACTS=(
  "$VERSION_REPORT"
  "$FORK_REPORT"
  "$FORK_POLICY_REPORT"
  "$DEVNET_MARKERS"
  "$DEVNET_REPORT"
)

CHECK_FILE="$(mktemp)"
ARTIFACT_FILE="$(mktemp)"
trap 'rm -f "$CHECK_FILE" "$ARTIFACT_FILE"' EXIT

overall_status="ok"
readiness_status="planned"
reason_code="dry_run_no_commands_executed"
already_failed=0

if [ "$MODE" = "run" ]; then
  readiness_status="ready"
  reason_code="local_bootstrap_health_checks_passed"
fi

pushd "$ROOT_DIR" >/dev/null
for index in "${!CHECK_IDS[@]}"; do
  check_id="${CHECK_IDS[$index]}"
  check_command="${CHECK_COMMANDS[$index]}"

  check_status="planned"
  if [ "$MODE" = "run" ]; then
    if [ "$already_failed" -eq 1 ]; then
      check_status="skipped"
    else
      if eval "$check_command"; then
        check_status="pass"
      else
        check_status="fail"
        overall_status="fail"
        readiness_status="failed"
        reason_code="bootstrap_check_failed_${check_id}"
        already_failed=1
      fi
    fi
  fi

  printf '%s\t%s\t%s\n' "$check_id" "$check_command" "$check_status" >>"$CHECK_FILE"
done
popd >/dev/null

for artifact in "${ARTIFACTS[@]}"; do
  printf '%s\n' "$artifact" >>"$ARTIFACT_FILE"
done

python3 "$ROOT_DIR/scripts/kolme/contracts/local_bootstrap_health_checks_summary.py" "$OUTPUT_JSON" "$MODE" "$overall_status" "$readiness_status" "$reason_code" "$CHECK_FILE" "$ARTIFACT_FILE"

echo "status=$overall_status"
echo "bootstrap_mode=$MODE"
echo "readiness_status=$readiness_status"
echo "local_only_enforced=true"
if [ -n "$reason_code" ]; then
  echo "reason_code=$reason_code"
fi
echo "summary_file=$(realpath "$OUTPUT_JSON")"

if [ "$overall_status" != "ok" ]; then
  exit 1
fi
