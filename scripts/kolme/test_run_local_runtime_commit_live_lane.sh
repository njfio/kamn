#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_runtime_commit_live_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_runtime_commit_live_evidence_policy.py"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
TMP_REPORT="$(mktemp)"
TMP_OUTPUT="$(mktemp)"
TMP_FINALITY_OUTPUT="$(mktemp)"
TMP_POLICY_REPORT="$(mktemp)"
TMP_POLICY_ERR="$(mktemp)"
TMP_ERR="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_OUTPUT" "$TMP_FINALITY_OUTPUT" "$TMP_POLICY_REPORT" "$TMP_POLICY_ERR" "$TMP_ERR"' EXIT

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

assert_eq() {
  local actual="$1"
  local expected="$2"
  local message="$3"
  if [ "$actual" != "$expected" ]; then
    echo "$message: expected '$expected', got '$actual'" >&2
    exit 1
  fi
}

if [ ! -x "$RUNNER" ]; then
  echo "expected local runtime commit live lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local runtime commit live evidence policy checker to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/assert_local_heavy_opt_in.sh" "$RUNNER"; then
  echo "expected runtime commit live runner to invoke shared local-heavy opt-in guard helper" >&2
  exit 1
fi

# Regression: #1969
if ! grep -q -- "--finality-command" "$RUNNER"; then
  echo "expected runtime commit live runner to expose optional finality command argument" >&2
  exit 1
fi

if ! grep -q "runtime_commit_live_finality_command" "$RUNNER"; then
  echo "expected runtime commit live runner to emit finality command check markers" >&2
  exit 1
fi

if ! grep -q "run_local_runtime_commit_live_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local runtime commit live lane runner" >&2
  exit 1
fi

dry_run_output="$(
  bash "$RUNNER" \
    --mode dry-run \
    --output-json "$TMP_REPORT" \
    --live-output-file "$TMP_OUTPUT"
)"

assert_eq "$(extract_value "$dry_run_output" "status")" "ok" "expected dry-run live lane to pass"
assert_eq "$(extract_value "$dry_run_output" "lane_mode")" "dry-run" "expected dry-run mode marker"
assert_eq "$(extract_value "$dry_run_output" "reason_code")" "dry_run_no_commands_executed" "expected dry-run reason code"
assert_eq "$(extract_value "$dry_run_output" "budget_status")" "not_run" "expected dry-run budget status"

python3 - "$TMP_REPORT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.kolme.local-runtime-commit-live-summary.v1":
    raise SystemExit("unexpected live lane summary schema")
if report.get("mode") != "dry-run":
    raise SystemExit("expected dry-run mode")
if report.get("local_only_enforced") is not True:
    raise SystemExit("expected local_only_enforced=true")
if report.get("provider_client_contract") != "KolmeRuntimeCommitLiveProvider":
    raise SystemExit("expected provider client contract marker")
if report.get("provider_submit_profile_contract") != "kolme_fork_broadcast_profile":
    raise SystemExit("expected provider submit profile contract marker")
if report.get("provider_command_marker") != "integration_kolme_fork_live_node_submit_reaches_endpoint":
    raise SystemExit("expected live provider command marker")
if report.get("provider_command_marker_present") is not True:
    raise SystemExit("expected default dry-run command to include live provider marker")
if report.get("provider_signing_profile_marker") != "KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1":
    raise SystemExit("expected deterministic live signing profile marker")
if report.get("provider_signing_profile_marker_present") is not True:
    raise SystemExit("expected default dry-run command to include live signing profile marker")
if report.get("submit_evidence_marker") != "status=submitted":
    raise SystemExit("expected deterministic submit evidence marker")
if report.get("submit_evidence_marker_present") is not False:
    raise SystemExit("expected submit evidence marker to be absent in dry-run default command profile")
if report.get("finality_evidence_marker") != "finality=final":
    raise SystemExit("expected deterministic finality evidence marker")
if report.get("finality_evidence_marker_present") is not False:
    raise SystemExit("expected finality evidence marker to be absent in dry-run default command profile")
if report.get("native_payload_pubkey_marker") != '"pubkey"':
    raise SystemExit("expected deterministic native payload pubkey marker")
if report.get("native_payload_nonce_marker") != '"nonce"':
    raise SystemExit("expected deterministic native payload nonce marker")
if report.get("native_payload_messages_marker") != '"messages"':
    raise SystemExit("expected deterministic native payload messages marker")
if report.get("native_payload_marker_contract_version") != "v1":
    raise SystemExit("expected native payload marker contract version")
if report.get("native_payload_pubkey_marker_present") is not False:
    raise SystemExit("expected native payload pubkey marker to be absent in dry-run default command profile")
if report.get("native_payload_nonce_marker_present") is not False:
    raise SystemExit("expected native payload nonce marker to be absent in dry-run default command profile")
if report.get("native_payload_messages_marker_present") is not False:
    raise SystemExit("expected native payload messages marker to be absent in dry-run default command profile")
if report.get("live_command_synthetic") is not False:
    raise SystemExit("expected default dry-run live command to be classified non-synthetic")
if report.get("finality_command_synthetic") is not False:
    raise SystemExit("expected default dry-run finality command classification false")
if report.get("synthetic_evidence_classification_version") != "v1":
    raise SystemExit("expected synthetic evidence classification version marker")
checks = report.get("checks")
if not isinstance(checks, list) or not checks:
    raise SystemExit("expected deterministic checks in summary")
if not any(
    check.get("id") == "runtime_commit_live_preflight" and check.get("status") == "planned"
    for check in checks
):
    raise SystemExit("expected planned runtime commit live preflight check")
if not any(
    check.get("id") == "runtime_commit_live_finality_command" and check.get("status") == "planned"
    for check in checks
):
    raise SystemExit("expected planned runtime commit live finality check")
PY

checker_output="$(
  python3 "$CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY_REPORT"
)"
assert_eq "$(extract_value "$checker_output" "status")" "ok" "expected live evidence policy checker to pass dry-run report"

set +e
KAMN_KOLME_LOCAL_HEAVY=1 \
  bash "$RUNNER" \
    --mode run \
    --live-command "printf 'status=submitted\n'" \
    --max-seconds 5 \
    --base-url "http://127.0.0.1:1" \
    --output-json "$TMP_REPORT" \
    --live-output-file "$TMP_OUTPUT" >"$TMP_ERR" 2>&1
preflight_failure_code=$?
set -e

if [ "$preflight_failure_code" -eq 0 ]; then
  echo "expected run mode preflight failure to fail closed" >&2
  exit 1
fi

if ! grep -q "reason_code=live_preflight_failed" "$TMP_ERR"; then
  echo "expected preflight failure reason marker" >&2
  exit 1
fi

set +e
bash "$RUNNER" \
  --mode run \
  --output-json "$TMP_REPORT" \
  --live-output-file "$TMP_OUTPUT" >"$TMP_ERR" 2>&1
run_without_opt_in_code=$?
set -e

if [ "$run_without_opt_in_code" -eq 0 ]; then
  echo "expected run mode without opt-in to fail closed" >&2
  exit 1
fi

if ! grep -q "requires explicit local-only opt-in" "$TMP_ERR"; then
  echo "expected explicit local opt-in failure message" >&2
  exit 1
fi

run_output="$(
  KAMN_KOLME_LOCAL_HEAVY=1 \
    bash "$RUNNER" \
      --mode run \
      --skip-preflight \
    --live-command "printf 'status=submitted\nprovider=kolme-local\ncommit_id=kolme-commit:1\nfinality=final\n'" \
    --max-seconds 5 \
    --output-json "$TMP_REPORT" \
    --live-output-file "$TMP_OUTPUT"
)"

assert_eq "$(extract_value "$run_output" "status")" "ok" "expected run mode to pass"
assert_eq "$(extract_value "$run_output" "lane_mode")" "run" "expected run mode marker"
assert_eq "$(extract_value "$run_output" "reason_code")" "live_runtime_commit_command_passed" "expected pass reason code"
assert_eq "$(extract_value "$run_output" "budget_status")" "within_budget" "expected within_budget status"

python3 - "$TMP_REPORT" "$TMP_OUTPUT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
live_output = pathlib.Path(sys.argv[2]).read_text(encoding="utf-8")
if report.get("mode") != "run":
    raise SystemExit("expected run mode in summary")
if report.get("status") != "ok":
    raise SystemExit("expected ok status in summary")
if report.get("reason_code") != "live_runtime_commit_command_passed":
    raise SystemExit("expected pass reason code in summary")
if report.get("max_seconds") != 5:
    raise SystemExit("expected max_seconds=5")
if "status=submitted" not in live_output:
    raise SystemExit("expected live command output marker")
if report.get("finality_enabled") is not False:
    raise SystemExit("expected finality_enabled=false when no finality command is configured")
if report.get("submit_evidence_marker_present") is not True:
    raise SystemExit("expected submit evidence marker to be present for run command output")
if report.get("finality_evidence_marker_present") is not False:
    raise SystemExit("expected finality evidence marker to remain false when no finality command is configured")
if report.get("live_command_synthetic") is not True:
    raise SystemExit("expected printf-only live command to be classified synthetic")
if report.get("finality_command_synthetic") is not False:
    raise SystemExit("expected finality command classification false when finality command is disabled")
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_REPORT" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-native-payload-evidence >"$TMP_POLICY_ERR" 2>&1
native_payload_policy_code=$?
set -e

if [ "$native_payload_policy_code" -eq 0 ]; then
  echo "expected evidence policy checker to fail when native payload evidence markers are absent" >&2
  exit 1
fi
if ! grep -q "native_payload_pubkey_marker_missing" "$TMP_POLICY_ERR"; then
  echo "expected native payload pubkey marker failure reason from evidence policy checker" >&2
  exit 1
fi
if ! grep -q "native_payload_nonce_marker_missing" "$TMP_POLICY_ERR"; then
  echo "expected native payload nonce marker failure reason from evidence policy checker" >&2
  exit 1
fi
if ! grep -q "native_payload_messages_marker_missing" "$TMP_POLICY_ERR"; then
  echo "expected native payload messages marker failure reason from evidence policy checker" >&2
  exit 1
fi

run_with_finality_output="$(
  KAMN_KOLME_LOCAL_HEAVY=1 \
    bash "$RUNNER" \
      --mode run \
      --skip-preflight \
      --live-command "KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1 printf 'status=submitted\nintegration_kolme_fork_live_node_submit_reaches_endpoint\n{\"pubkey\":\"proof\",\"nonce\":1,\"messages\":[]}\n'" \
      --finality-command "printf 'finality=final\n'" \
      --finality-max-seconds 3 \
      --max-seconds 5 \
      --output-json "$TMP_REPORT" \
      --live-output-file "$TMP_OUTPUT" \
      --finality-output-file "$TMP_FINALITY_OUTPUT"
)"

assert_eq "$(extract_value "$run_with_finality_output" "status")" "ok" "expected run mode with finality command to pass"
assert_eq "$(extract_value "$run_with_finality_output" "reason_code")" "live_runtime_commit_and_finality_commands_passed" "expected combined pass reason code"

python3 - "$TMP_REPORT" "$TMP_FINALITY_OUTPUT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
finality_output = pathlib.Path(sys.argv[2]).read_text(encoding="utf-8")
if report.get("finality_enabled") is not True:
    raise SystemExit("expected finality_enabled=true when finality command is configured")
if report.get("reason_code") != "live_runtime_commit_and_finality_commands_passed":
    raise SystemExit("expected combined pass reason code in summary")
if "finality=final" not in finality_output:
    raise SystemExit("expected finality command output marker")
if report.get("submit_evidence_marker_present") is not True:
    raise SystemExit("expected submit evidence marker to remain true in finality-enabled run summary")
if report.get("finality_evidence_marker_present") is not True:
    raise SystemExit("expected finality evidence marker to be true in finality-enabled run summary")
if report.get("live_command_synthetic") is not True:
    raise SystemExit("expected synthetic live command classification for printf fixture")
if report.get("finality_command_synthetic") is not True:
    raise SystemExit("expected synthetic finality command classification for printf fixture")
if report.get("native_payload_pubkey_marker_present") is not True:
    raise SystemExit("expected native payload pubkey marker evidence for finality-enabled run summary")
if report.get("native_payload_nonce_marker_present") is not True:
    raise SystemExit("expected native payload nonce marker evidence for finality-enabled run summary")
if report.get("native_payload_messages_marker_present") is not True:
    raise SystemExit("expected native payload messages marker evidence for finality-enabled run summary")
PY

checker_native_payload_output="$(
  python3 "$CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --require-native-payload-evidence \
    --output-json "$TMP_POLICY_REPORT"
)"
assert_eq "$(extract_value "$checker_native_payload_output" "status")" "ok" "expected native payload evidence policy checker to pass marker-complete run report"

set +e
python3 "$CHECKER" \
  --report-file "$TMP_REPORT" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-non-synthetic-run-evidence >"$TMP_POLICY_ERR" 2>&1
non_synthetic_policy_code=$?
set -e

if [ "$non_synthetic_policy_code" -eq 0 ]; then
  echo "expected evidence policy checker to fail when synthetic run commands are disallowed" >&2
  exit 1
fi
if ! grep -q "synthetic_live_command_detected" "$TMP_POLICY_ERR"; then
  echo "expected synthetic live command failure reason from evidence policy checker" >&2
  exit 1
fi
if ! grep -q "synthetic_finality_command_detected" "$TMP_POLICY_ERR"; then
  echo "expected synthetic finality command failure reason from evidence policy checker" >&2
  exit 1
fi

set +e
KAMN_KOLME_LOCAL_HEAVY=1 \
  bash "$RUNNER" \
    --mode run \
    --skip-preflight \
    --live-command "sleep 2" \
    --max-seconds 1 \
    --output-json "$TMP_REPORT" \
    --live-output-file "$TMP_OUTPUT" >"$TMP_ERR" 2>&1
timeout_code=$?
set -e

if [ "$timeout_code" -eq 0 ]; then
  echo "expected run mode timeout to fail closed" >&2
  exit 1
fi

if ! grep -q "reason_code=live_runtime_commit_command_timeout" "$TMP_ERR"; then
  echo "expected timeout reason marker" >&2
  exit 1
fi

set +e
python3 "$CHECKER" \
  --report-file "$TMP_REPORT" \
  --expected-final-decision GO \
  --ci-fast-gate PASS >"$TMP_POLICY_ERR" 2>&1
policy_failure_code=$?
set -e

if [ "$policy_failure_code" -eq 0 ]; then
  echo "expected evidence policy checker to fail when live provider command marker is absent" >&2
  exit 1
fi
if ! grep -q "provider_command_marker_missing" "$TMP_POLICY_ERR"; then
  echo "expected provider marker failure reason from evidence policy checker" >&2
  exit 1
fi

run_missing_submit_evidence_output="$(
  KAMN_KOLME_LOCAL_HEAVY=1 \
    bash "$RUNNER" \
      --mode run \
      --skip-preflight \
      --live-command "printf 'integration_kolme_fork_live_node_submit_reaches_endpoint\n'" \
      --finality-command "printf 'finality=final\n'" \
      --max-seconds 5 \
      --finality-max-seconds 3 \
      --output-json "$TMP_REPORT" \
      --live-output-file "$TMP_OUTPUT" \
      --finality-output-file "$TMP_FINALITY_OUTPUT"
)"
assert_eq "$(extract_value "$run_missing_submit_evidence_output" "status")" "ok" "expected run mode to pass even when submit evidence marker is absent from output"

set +e
python3 "$CHECKER" \
  --report-file "$TMP_REPORT" \
  --expected-final-decision GO \
  --ci-fast-gate PASS >"$TMP_POLICY_ERR" 2>&1
missing_submit_policy_code=$?
set -e

if [ "$missing_submit_policy_code" -eq 0 ]; then
  echo "expected evidence policy checker to fail when submit evidence marker is absent in run output" >&2
  exit 1
fi
if ! grep -q "submit_evidence_marker_missing" "$TMP_POLICY_ERR"; then
  echo "expected submit evidence marker failure reason from evidence policy checker" >&2
  exit 1
fi

set +e
KAMN_KOLME_LOCAL_HEAVY=1 \
  bash "$RUNNER" \
    --mode run \
    --skip-preflight \
    --live-command "printf 'status=submitted\n'" \
    --finality-command "sleep 2" \
    --finality-max-seconds 1 \
    --max-seconds 5 \
    --output-json "$TMP_REPORT" \
    --live-output-file "$TMP_OUTPUT" \
    --finality-output-file "$TMP_FINALITY_OUTPUT" >"$TMP_ERR" 2>&1
finality_timeout_code=$?
set -e

if [ "$finality_timeout_code" -eq 0 ]; then
  echo "expected finality command timeout to fail closed" >&2
  exit 1
fi

if ! grep -q "reason_code=live_finality_command_timeout" "$TMP_ERR"; then
  echo "expected finality timeout reason marker" >&2
  exit 1
fi

echo "local runtime commit live lane tests passed."
