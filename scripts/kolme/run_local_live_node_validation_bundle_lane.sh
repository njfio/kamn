#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INTEGRATION_RUNNER="$ROOT_DIR/scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh"
INTEGRATION_POLICY_CHECKER="$ROOT_DIR/scripts/kolme/check_local_kamn_live_runtime_integration_policy.py"
PROCESS_LIFECYCLE_RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh"
PROCESS_LIFECYCLE_POLICY_CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_fork_process_lifecycle_policy.py"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-live-node-validation-bundle-summary.json"
INTEGRATION_REPORT="/tmp/kolme-local-kamn-live-runtime-integration-summary.json"
INTEGRATION_POLICY_REPORT="/tmp/kolme-local-kamn-live-runtime-integration-policy.json"
INTEGRATION_RUNTIME_POLICY_REPORT="/tmp/kolme-local-runtime-commit-live-policy.json"
INTEGRATION_RUNTIME_COMMIT_LIVE_SUMMARY="/tmp/kolme-local-runtime-commit-live-summary.json"
PROCESS_LIFECYCLE_REPORT="/tmp/kolme-local-fork-process-lifecycle-summary.json"
PROCESS_LIFECYCLE_POLICY_REPORT="/tmp/kolme-local-fork-process-lifecycle-policy.json"
ROLLBACK_EVIDENCE_FILE="/tmp/kolme-local-fork-process-lifecycle-rollback-evidence.json"
RECOVERY_EVIDENCE_FILE="/tmp/kolme-local-fork-process-lifecycle-recovery-evidence.json"
CHECKOUT_PATH="/tmp/kolme_fork"
EXPECTED_REMOTE_URL="https://github.com/njfio/kolme_fork.git"
EXPECTED_REF="refs/heads/main"
BASE_URL="http://127.0.0.1:3000"
FORK_CHAIN_VERSION="v0.15.2"
INTEGRATION_COMMAND=""
INTEGRATION_POLICY_COMMAND=""
PROCESS_LIFECYCLE_COMMAND=""
PROCESS_LIFECYCLE_POLICY_COMMAND=""
MAX_SECONDS=480
INTEGRATION_MAX_SECONDS=240
PROCESS_LIFECYCLE_MAX_SECONDS=300

shell_escape() {
  printf "%q" "$1"
}

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
    --integration-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --integration-report" >&2
        exit 1
      fi
      INTEGRATION_REPORT="$2"
      shift 2
      ;;
    --integration-policy-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --integration-policy-report" >&2
        exit 1
      fi
      INTEGRATION_POLICY_REPORT="$2"
      shift 2
      ;;
    --integration-runtime-policy-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --integration-runtime-policy-report" >&2
        exit 1
      fi
      INTEGRATION_RUNTIME_POLICY_REPORT="$2"
      shift 2
      ;;
    --integration-runtime-commit-live-summary)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --integration-runtime-commit-live-summary" >&2
        exit 1
      fi
      INTEGRATION_RUNTIME_COMMIT_LIVE_SUMMARY="$2"
      shift 2
      ;;
    --process-lifecycle-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --process-lifecycle-report" >&2
        exit 1
      fi
      PROCESS_LIFECYCLE_REPORT="$2"
      shift 2
      ;;
    --process-lifecycle-policy-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --process-lifecycle-policy-report" >&2
        exit 1
      fi
      PROCESS_LIFECYCLE_POLICY_REPORT="$2"
      shift 2
      ;;
    --rollback-evidence-file)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --rollback-evidence-file" >&2
        exit 1
      fi
      ROLLBACK_EVIDENCE_FILE="$2"
      shift 2
      ;;
    --recovery-evidence-file)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --recovery-evidence-file" >&2
        exit 1
      fi
      RECOVERY_EVIDENCE_FILE="$2"
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
    --base-url)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --base-url" >&2
        exit 1
      fi
      BASE_URL="$2"
      shift 2
      ;;
    --fork-chain-version)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --fork-chain-version" >&2
        exit 1
      fi
      FORK_CHAIN_VERSION="$2"
      shift 2
      ;;
    --integration-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --integration-command" >&2
        exit 1
      fi
      INTEGRATION_COMMAND="$2"
      shift 2
      ;;
    --integration-policy-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --integration-policy-command" >&2
        exit 1
      fi
      INTEGRATION_POLICY_COMMAND="$2"
      shift 2
      ;;
    --process-lifecycle-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --process-lifecycle-command" >&2
        exit 1
      fi
      PROCESS_LIFECYCLE_COMMAND="$2"
      shift 2
      ;;
    --process-lifecycle-policy-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --process-lifecycle-policy-command" >&2
        exit 1
      fi
      PROCESS_LIFECYCLE_POLICY_COMMAND="$2"
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
    --integration-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --integration-max-seconds" >&2
        exit 1
      fi
      INTEGRATION_MAX_SECONDS="$2"
      shift 2
      ;;
    --process-lifecycle-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --process-lifecycle-max-seconds" >&2
        exit 1
      fi
      PROCESS_LIFECYCLE_MAX_SECONDS="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_live_node_validation_bundle_lane.sh [options]

Options:
  --mode dry-run|run                              Emit planned checks or execute local validation bundle.
  --output-json <path>                            Deterministic bundle summary output path.
  --integration-report <path>                     Nested local integration summary path.
  --integration-policy-report <path>              Nested local integration policy report path.
  --integration-runtime-policy-report <path>      Nested runtime policy report path produced by integration lane.
  --integration-runtime-commit-live-summary <path>
                                                  Nested runtime-commit live summary path.
  --process-lifecycle-report <path>               Nested process lifecycle summary path.
  --process-lifecycle-policy-report <path>        Nested process lifecycle policy report path.
  --rollback-evidence-file <path>                 Rollback evidence artifact path.
  --recovery-evidence-file <path>                 Recovery evidence artifact path.
  --checkout-path <path>                          Local kolme_fork checkout path.
  --expected-remote-url <url>                     Expected origin URL for checkout validation.
  --expected-ref <ref>                            Expected symbolic HEAD ref for checkout.
  --base-url <url>                                Base URL for local Kolme API server.
  --fork-chain-version <value>                    Required fork-info chain_version query value.
  --integration-command <command>                 Override nested integration command.
  --integration-policy-command <command>          Override nested integration policy command.
  --process-lifecycle-command <command>           Override nested process lifecycle command.
  --process-lifecycle-policy-command <command>    Override nested process lifecycle policy command.
  --max-seconds <n>                               Max total runtime budget.
  --integration-max-seconds <n>                   Max runtime budget for nested integration command.
  --process-lifecycle-max-seconds <n>             Max runtime budget for nested process lifecycle command.
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

for numeric_value in "$MAX_SECONDS" "$INTEGRATION_MAX_SECONDS" "$PROCESS_LIFECYCLE_MAX_SECONDS"; do
  if ! [[ "$numeric_value" =~ ^[0-9]+$ ]] || [ "$numeric_value" -le 0 ]; then
    echo "all max-second arguments must be positive integers" >&2
    exit 1
  fi
done

for required_value in "$CHECKOUT_PATH" "$EXPECTED_REMOTE_URL" "$EXPECTED_REF" "$BASE_URL" "$FORK_CHAIN_VERSION"; do
  if [ -z "$required_value" ]; then
    echo "checkout and endpoint contract inputs must not be empty" >&2
    exit 1
  fi
done

if [ ! -x "$INTEGRATION_RUNNER" ]; then
  echo "expected local KAMN live runtime integration lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$INTEGRATION_POLICY_CHECKER" ]; then
  echo "expected local KAMN live runtime integration policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$PROCESS_LIFECYCLE_RUNNER" ]; then
  echo "expected local fork process lifecycle lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$PROCESS_LIFECYCLE_POLICY_CHECKER" ]; then
  echo "expected local fork process lifecycle policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

default_integration_command="KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode run --runtime-profile real-node --checkout-path $(shell_escape "$CHECKOUT_PATH") --expected-remote-url $(shell_escape "$EXPECTED_REMOTE_URL") --expected-ref $(shell_escape "$EXPECTED_REF") --base-url $(shell_escape "$BASE_URL") --fork-chain-version $(shell_escape "$FORK_CHAIN_VERSION") --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider --max-seconds ${INTEGRATION_MAX_SECONDS} --bootstrap-max-seconds 90 --localhost-signed-max-seconds 45 --conformance-max-seconds 180 --runtime-commit-max-seconds 30 --runtime-commit-live-summary $(shell_escape "$INTEGRATION_RUNTIME_COMMIT_LIVE_SUMMARY") --runtime-commit-live-policy-report $(shell_escape "$INTEGRATION_RUNTIME_POLICY_REPORT") --output-json $(shell_escape "$INTEGRATION_REPORT")"
default_integration_policy_command="python3 scripts/kolme/check_local_kamn_live_runtime_integration_policy.py --report-file $(shell_escape "$INTEGRATION_REPORT") --expected-final-decision GO --ci-fast-gate PASS --output-json $(shell_escape "$INTEGRATION_POLICY_REPORT")"
default_process_lifecycle_command="KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh --mode run --checkout-path $(shell_escape "$CHECKOUT_PATH") --expected-remote-url $(shell_escape "$EXPECTED_REMOTE_URL") --expected-ref $(shell_escape "$EXPECTED_REF") --base-url $(shell_escape "$BASE_URL") --fork-chain-version $(shell_escape "$FORK_CHAIN_VERSION") --max-seconds ${PROCESS_LIFECYCLE_MAX_SECONDS} --startup-max-seconds 45 --integration-max-seconds ${INTEGRATION_MAX_SECONDS} --integration-bootstrap-max-seconds 90 --integration-conformance-max-seconds 180 --integration-runtime-commit-max-seconds 30 --integration-runtime-commit-live-policy-report $(shell_escape "$INTEGRATION_RUNTIME_POLICY_REPORT") --rollback-evidence-file $(shell_escape "$ROLLBACK_EVIDENCE_FILE") --recovery-evidence-file $(shell_escape "$RECOVERY_EVIDENCE_FILE") --output-json $(shell_escape "$PROCESS_LIFECYCLE_REPORT")"
default_process_lifecycle_policy_command="python3 scripts/kolme/check_local_kolme_fork_process_lifecycle_policy.py --report-file $(shell_escape "$PROCESS_LIFECYCLE_REPORT") --expected-final-decision GO --ci-fast-gate PASS --output-json $(shell_escape "$PROCESS_LIFECYCLE_POLICY_REPORT")"

if [ -z "$INTEGRATION_COMMAND" ]; then
  INTEGRATION_COMMAND="$default_integration_command"
fi
if [ -z "$INTEGRATION_POLICY_COMMAND" ]; then
  INTEGRATION_POLICY_COMMAND="$default_integration_policy_command"
fi
if [ -z "$PROCESS_LIFECYCLE_COMMAND" ]; then
  PROCESS_LIFECYCLE_COMMAND="$default_process_lifecycle_command"
fi
if [ -z "$PROCESS_LIFECYCLE_POLICY_COMMAND" ]; then
  PROCESS_LIFECYCLE_POLICY_COMMAND="$default_process_lifecycle_policy_command"
fi

CHECK_FILE="$(mktemp)"
trap 'rm -f "$CHECK_FILE"' EXIT

record_check() {
  local check_id="$1"
  local command="$2"
  local status="$3"
  local reason_code="$4"
  printf '%s\t%s\t%s\t%s\n' "$check_id" "$command" "$status" "$reason_code" >>"$CHECK_FILE"
}

read_policy_final_decision() {
  local report_file="$1"
  python3 - "$report_file" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
if not path.exists():
    print("report_missing")
    raise SystemExit(0)

try:
    payload = json.loads(path.read_text(encoding="utf-8"))
except json.JSONDecodeError:
    print("report_invalid_json")
    raise SystemExit(0)

value = payload.get("final_decision")
if isinstance(value, str) and value.strip():
    print(value)
else:
    print("final_decision_missing")
PY
}

overall_status="ok"
reason_code="dry_run_no_commands_executed"
budget_status="not_run"
elapsed_seconds=0

record_check "integration_bundle" "$INTEGRATION_COMMAND" "planned" "not_run"
record_check "integration_policy" "$INTEGRATION_POLICY_COMMAND" "planned" "not_run"
record_check "process_lifecycle_bundle" "$PROCESS_LIFECYCLE_COMMAND" "planned" "not_run"
record_check "process_lifecycle_policy" "$PROCESS_LIFECYCLE_POLICY_COMMAND" "planned" "not_run"

if [ "$MODE" = "run" ]; then
  : >"$CHECK_FILE"
  start_epoch="$(date +%s)"

  if ! "$LOCAL_HEAVY_GUARD"; then
    record_check "integration_bundle" "$INTEGRATION_COMMAND" "fail" "local_opt_in_missing"
    record_check "integration_policy" "$INTEGRATION_POLICY_COMMAND" "skipped" "integration_bundle_failed"
    record_check "process_lifecycle_bundle" "$PROCESS_LIFECYCLE_COMMAND" "skipped" "integration_bundle_failed"
    record_check "process_lifecycle_policy" "$PROCESS_LIFECYCLE_POLICY_COMMAND" "skipped" "integration_bundle_failed"
    overall_status="fail"
    reason_code="local_opt_in_missing"
  else
    set +e
    timeout "$INTEGRATION_MAX_SECONDS" bash -lc "$INTEGRATION_COMMAND" >/dev/null 2>&1
    integration_exit_code=$?
    set -e

    if [ "$integration_exit_code" -eq 0 ]; then
      record_check "integration_bundle" "$INTEGRATION_COMMAND" "pass" "integration_bundle_passed"
    elif [ "$integration_exit_code" -eq 124 ]; then
      record_check "integration_bundle" "$INTEGRATION_COMMAND" "fail" "integration_bundle_timeout"
      record_check "integration_policy" "$INTEGRATION_POLICY_COMMAND" "skipped" "integration_bundle_failed"
      record_check "process_lifecycle_bundle" "$PROCESS_LIFECYCLE_COMMAND" "skipped" "integration_bundle_failed"
      record_check "process_lifecycle_policy" "$PROCESS_LIFECYCLE_POLICY_COMMAND" "skipped" "integration_bundle_failed"
      overall_status="fail"
      reason_code="integration_bundle_timeout"
    else
      record_check "integration_bundle" "$INTEGRATION_COMMAND" "fail" "integration_bundle_failed"
      record_check "integration_policy" "$INTEGRATION_POLICY_COMMAND" "skipped" "integration_bundle_failed"
      record_check "process_lifecycle_bundle" "$PROCESS_LIFECYCLE_COMMAND" "skipped" "integration_bundle_failed"
      record_check "process_lifecycle_policy" "$PROCESS_LIFECYCLE_POLICY_COMMAND" "skipped" "integration_bundle_failed"
      overall_status="fail"
      reason_code="integration_bundle_failed"
    fi

    if [ "$overall_status" = "ok" ]; then
      set +e
      timeout 30 bash -lc "$INTEGRATION_POLICY_COMMAND" >/dev/null 2>&1
      integration_policy_exit_code=$?
      set -e

      if [ "$integration_policy_exit_code" -eq 0 ]; then
        integration_policy_final_decision="$(read_policy_final_decision "$INTEGRATION_POLICY_REPORT")"
        if [ "$integration_policy_final_decision" = "GO" ]; then
          record_check "integration_policy" "$INTEGRATION_POLICY_COMMAND" "pass" "integration_policy_passed"
        else
          record_check "integration_policy" "$INTEGRATION_POLICY_COMMAND" "fail" "integration_policy_failed:${integration_policy_final_decision}"
          record_check "process_lifecycle_bundle" "$PROCESS_LIFECYCLE_COMMAND" "skipped" "integration_policy_failed"
          record_check "process_lifecycle_policy" "$PROCESS_LIFECYCLE_POLICY_COMMAND" "skipped" "integration_policy_failed"
          overall_status="fail"
          reason_code="integration_policy_failed"
        fi
      elif [ "$integration_policy_exit_code" -eq 124 ]; then
        record_check "integration_policy" "$INTEGRATION_POLICY_COMMAND" "fail" "integration_policy_timeout"
        record_check "process_lifecycle_bundle" "$PROCESS_LIFECYCLE_COMMAND" "skipped" "integration_policy_failed"
        record_check "process_lifecycle_policy" "$PROCESS_LIFECYCLE_POLICY_COMMAND" "skipped" "integration_policy_failed"
        overall_status="fail"
        reason_code="integration_policy_failed"
      else
        record_check "integration_policy" "$INTEGRATION_POLICY_COMMAND" "fail" "integration_policy_failed"
        record_check "process_lifecycle_bundle" "$PROCESS_LIFECYCLE_COMMAND" "skipped" "integration_policy_failed"
        record_check "process_lifecycle_policy" "$PROCESS_LIFECYCLE_POLICY_COMMAND" "skipped" "integration_policy_failed"
        overall_status="fail"
        reason_code="integration_policy_failed"
      fi
    fi

    if [ "$overall_status" = "ok" ]; then
      set +e
      timeout "$PROCESS_LIFECYCLE_MAX_SECONDS" bash -lc "$PROCESS_LIFECYCLE_COMMAND" >/dev/null 2>&1
      process_exit_code=$?
      set -e

      if [ "$process_exit_code" -eq 0 ]; then
        record_check "process_lifecycle_bundle" "$PROCESS_LIFECYCLE_COMMAND" "pass" "process_lifecycle_bundle_passed"
      elif [ "$process_exit_code" -eq 124 ]; then
        record_check "process_lifecycle_bundle" "$PROCESS_LIFECYCLE_COMMAND" "fail" "process_lifecycle_bundle_timeout"
        record_check "process_lifecycle_policy" "$PROCESS_LIFECYCLE_POLICY_COMMAND" "skipped" "process_lifecycle_bundle_failed"
        overall_status="fail"
        reason_code="process_lifecycle_bundle_timeout"
      else
        record_check "process_lifecycle_bundle" "$PROCESS_LIFECYCLE_COMMAND" "fail" "process_lifecycle_bundle_failed"
        record_check "process_lifecycle_policy" "$PROCESS_LIFECYCLE_POLICY_COMMAND" "skipped" "process_lifecycle_bundle_failed"
        overall_status="fail"
        reason_code="process_lifecycle_bundle_failed"
      fi
    fi

    if [ "$overall_status" = "ok" ]; then
      set +e
      timeout 30 bash -lc "$PROCESS_LIFECYCLE_POLICY_COMMAND" >/dev/null 2>&1
      process_policy_exit_code=$?
      set -e

      if [ "$process_policy_exit_code" -eq 0 ]; then
        process_policy_final_decision="$(read_policy_final_decision "$PROCESS_LIFECYCLE_POLICY_REPORT")"
        if [ "$process_policy_final_decision" = "GO" ]; then
          record_check "process_lifecycle_policy" "$PROCESS_LIFECYCLE_POLICY_COMMAND" "pass" "process_lifecycle_policy_passed"
          reason_code="live_node_validation_bundle_passed"
        else
          record_check "process_lifecycle_policy" "$PROCESS_LIFECYCLE_POLICY_COMMAND" "fail" "process_lifecycle_policy_failed:${process_policy_final_decision}"
          overall_status="fail"
          reason_code="process_lifecycle_policy_failed"
        fi
      elif [ "$process_policy_exit_code" -eq 124 ]; then
        record_check "process_lifecycle_policy" "$PROCESS_LIFECYCLE_POLICY_COMMAND" "fail" "process_lifecycle_policy_timeout"
        overall_status="fail"
        reason_code="process_lifecycle_policy_failed"
      else
        record_check "process_lifecycle_policy" "$PROCESS_LIFECYCLE_POLICY_COMMAND" "fail" "process_lifecycle_policy_failed"
        overall_status="fail"
        reason_code="process_lifecycle_policy_failed"
      fi
    fi
  fi

  elapsed_seconds="$(( $(date +%s) - start_epoch ))"
  if [ "$elapsed_seconds" -le "$MAX_SECONDS" ]; then
    budget_status="within_budget"
  else
    budget_status="exceeded_budget"
    if [ "$overall_status" = "ok" ]; then
      overall_status="fail"
      reason_code="bundle_budget_exceeded"
    fi
  fi
fi

python3 - "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$elapsed_seconds" "$MAX_SECONDS" "$budget_status" "$CHECKOUT_PATH" "$EXPECTED_REMOTE_URL" "$EXPECTED_REF" "$BASE_URL" "$FORK_CHAIN_VERSION" "$INTEGRATION_COMMAND" "$INTEGRATION_POLICY_COMMAND" "$PROCESS_LIFECYCLE_COMMAND" "$PROCESS_LIFECYCLE_POLICY_COMMAND" "$INTEGRATION_REPORT" "$INTEGRATION_POLICY_REPORT" "$INTEGRATION_RUNTIME_POLICY_REPORT" "$INTEGRATION_RUNTIME_COMMIT_LIVE_SUMMARY" "$PROCESS_LIFECYCLE_REPORT" "$PROCESS_LIFECYCLE_POLICY_REPORT" "$ROLLBACK_EVIDENCE_FILE" "$RECOVERY_EVIDENCE_FILE" "$CHECK_FILE" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

output_path = pathlib.Path(sys.argv[1]).resolve()
mode = sys.argv[2]
status = sys.argv[3]
reason_code = sys.argv[4]
elapsed_seconds = int(sys.argv[5])
max_seconds = int(sys.argv[6])
budget_status = sys.argv[7]
checkout_path = sys.argv[8]
expected_remote_url = sys.argv[9]
expected_ref = sys.argv[10]
base_url = sys.argv[11]
fork_chain_version = sys.argv[12]
integration_command = sys.argv[13]
integration_policy_command = sys.argv[14]
process_command = sys.argv[15]
process_policy_command = sys.argv[16]
integration_report = sys.argv[17]
integration_policy_report = sys.argv[18]
integration_runtime_policy_report = sys.argv[19]
integration_runtime_live_summary = sys.argv[20]
process_report = sys.argv[21]
process_policy_report = sys.argv[22]
rollback_evidence_file = sys.argv[23]
recovery_evidence_file = sys.argv[24]
checks_path = pathlib.Path(sys.argv[25])

checks = []
for raw_line in checks_path.read_text(encoding="utf-8").splitlines():
    if not raw_line.strip():
        continue
    parts = raw_line.split("\t")
    if len(parts) != 4:
        continue
    check_id, command, check_status, check_reason_code = parts
    checks.append(
        {
            "id": check_id,
            "command": command,
            "status": check_status,
            "reason_code": check_reason_code,
        }
    )

summary = {
    "schema_version": "kamn.kolme.local-live-node-validation-bundle-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": True,
    "ci_fast_gate_eligible": False,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": budget_status,
    "checkout_path": checkout_path,
    "expected_remote_url": expected_remote_url,
    "expected_ref": expected_ref,
    "base_url": base_url,
    "fork_chain_version": fork_chain_version,
    "integration_command": integration_command,
    "integration_policy_command": integration_policy_command,
    "process_lifecycle_command": process_command,
    "process_lifecycle_policy_command": process_policy_command,
    "integration_report": integration_report,
    "integration_policy_report": integration_policy_report,
    "integration_runtime_policy_report": integration_runtime_policy_report,
    "integration_runtime_commit_live_summary": integration_runtime_live_summary,
    "process_lifecycle_report": process_report,
    "process_lifecycle_policy_report": process_policy_report,
    "checks": checks,
    "artifact_paths": [
        integration_report,
        integration_policy_report,
        integration_runtime_policy_report,
        integration_runtime_live_summary,
        process_report,
        process_policy_report,
        rollback_evidence_file,
        recovery_evidence_file,
    ],
    "contracts": {
        "ci_fast_gate_scope": "local-only",
        "runtime_provider_client_contract": "KolmeRuntimeCommitLiveProvider",
        "bundle_contract": "live_node_release_bundle_v1",
    },
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

echo "status=$overall_status"
echo "lane_mode=$MODE"
echo "reason_code=$reason_code"
echo "budget_status=$budget_status"
echo "local_only_enforced=true"
echo "ci_fast_gate_eligible=false"
echo "summary_file=$(realpath "$OUTPUT_JSON")"

if [ "$overall_status" != "ok" ]; then
  exit 1
fi
