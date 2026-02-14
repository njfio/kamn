#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/deploy/generate_staging_rehearsal_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/deploy/check_staging_rehearsal_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

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

if [ ! -x "$GENERATOR" ]; then
  echo "expected staging rehearsal bundle generator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected staging rehearsal policy checker to be executable" >&2
  exit 1
fi

go_bundle="$TMP_DIR/rehearsal-go.json"
go_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --release-candidate "v1.1.0-rc.1" \
    --deploy-status PASS \
    --rollback-status PASS \
    --rollback-target-hash "state-hash-abc" \
    --post-rollback-hash "state-hash-abc" \
    --recovery-time-seconds 420 \
    --max-allowed-recovery-time-seconds 900 \
    --evidence-complete true \
    --ci-fast-gate PASS \
    --runtime-submit-success-rate-bps 10000 \
    --min-runtime-submit-success-rate-bps 9900 \
    --runtime-finality-timeout-count 0 \
    --max-runtime-finality-timeout-count 1 \
    --signer-profile-drift-events 0 \
    --max-signer-profile-drift-events 0
)"

assert_eq "$(extract_value "$go_generate_output" "status")" "generated" "expected GO rehearsal bundle generation to succeed"
assert_eq "$(extract_value "$go_generate_output" "final_decision")" "GO" "expected generator to derive GO rehearsal decision"

python3 - "$go_bundle" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
signoff = payload.get("staged_rehearsal_signoff")
if not isinstance(signoff, dict):
    raise SystemExit("expected staged_rehearsal_signoff object")
if signoff.get("schema_version") != "kamn.release.staged-rehearsal-signoff.v1":
    raise SystemExit("expected staged_rehearsal_signoff schema marker")
if signoff.get("lineage_status") != "verified":
    raise SystemExit("expected staged_rehearsal_signoff lineage_status=verified for GO rehearsal")
if signoff.get("final_decision") != "GO":
    raise SystemExit("expected staged_rehearsal_signoff final_decision=GO for GO rehearsal")
required_artifacts = signoff.get("required_artifacts")
if not isinstance(required_artifacts, list) or "rollback_hash_match" not in required_artifacts:
    raise SystemExit("expected staged_rehearsal_signoff required_artifacts to include rollback_hash_match")
if "contracts" not in signoff:
    raise SystemExit("expected staged_rehearsal_signoff contracts object")
PY

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO rehearsal policy check to pass"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected GO rehearsal policy check decision"
assert_eq "$(extract_value "$go_policy_output" "mttr_within_bound")" "true" "expected GO rehearsal policy check to report bounded MTTR"
assert_eq "$(extract_value "$go_policy_output" "staged_rehearsal_signoff_status")" "verified" "expected policy checker to report verified staged rehearsal signoff status for GO rehearsal"

no_go_bundle="$TMP_DIR/rehearsal-no-go.json"
no_go_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --release-candidate "v1.1.0-rc.2" \
    --deploy-status PASS \
    --rollback-status PASS \
    --rollback-target-hash "state-hash-expected" \
    --post-rollback-hash "state-hash-observed" \
    --recovery-time-seconds 420 \
    --max-allowed-recovery-time-seconds 900 \
    --evidence-complete true \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$no_go_generate_output" "final_decision")" "NO-GO" "expected rollback hash mismatch to force NO-GO"

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO rehearsal policy check to pass"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected NO-GO rehearsal policy check decision"

mttr_no_go_bundle="$TMP_DIR/rehearsal-no-go-mttr.json"
mttr_no_go_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$mttr_no_go_bundle" \
    --release-candidate "v1.1.0-rc.3" \
    --deploy-status PASS \
    --rollback-status PASS \
    --rollback-target-hash "state-hash-stable" \
    --post-rollback-hash "state-hash-stable" \
    --recovery-time-seconds 1200 \
    --max-allowed-recovery-time-seconds 900 \
    --evidence-complete true \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$mttr_no_go_generate_output" "final_decision")" "NO-GO" "expected MTTR bound breach to force NO-GO"

mttr_no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$mttr_no_go_bundle")"
assert_eq "$(extract_value "$mttr_no_go_policy_output" "status")" "ok" "expected MTTR NO-GO rehearsal policy check to pass"
assert_eq "$(extract_value "$mttr_no_go_policy_output" "final_decision")" "NO-GO" "expected MTTR NO-GO rehearsal policy check decision"
assert_eq "$(extract_value "$mttr_no_go_policy_output" "mttr_within_bound")" "false" "expected MTTR NO-GO rehearsal policy check to report out-of-bound recovery time"

telemetry_no_go_bundle="$TMP_DIR/rehearsal-no-go-telemetry.json"
telemetry_no_go_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$telemetry_no_go_bundle" \
    --release-candidate "v1.1.0-rc.4" \
    --deploy-status PASS \
    --rollback-status PASS \
    --rollback-target-hash "state-hash-stable" \
    --post-rollback-hash "state-hash-stable" \
    --recovery-time-seconds 420 \
    --max-allowed-recovery-time-seconds 900 \
    --evidence-complete true \
    --ci-fast-gate PASS \
    --runtime-submit-success-rate-bps 9200 \
    --min-runtime-submit-success-rate-bps 9500 \
    --runtime-finality-timeout-count 3 \
    --max-runtime-finality-timeout-count 1 \
    --signer-profile-drift-events 2 \
    --max-signer-profile-drift-events 0
)"

assert_eq "$(extract_value "$telemetry_no_go_generate_output" "final_decision")" "NO-GO" "expected runtime telemetry threshold breach to force NO-GO"

telemetry_no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$telemetry_no_go_bundle")"
assert_eq "$(extract_value "$telemetry_no_go_policy_output" "status")" "ok" "expected telemetry NO-GO rehearsal policy check to pass"
assert_eq "$(extract_value "$telemetry_no_go_policy_output" "final_decision")" "NO-GO" "expected telemetry NO-GO rehearsal policy check decision"
assert_eq "$(extract_value "$telemetry_no_go_policy_output" "staged_rehearsal_signoff_status")" "fail-closed" "expected policy checker to report fail-closed staged rehearsal signoff status for NO-GO rehearsal"

telemetry_reason_codes="$(extract_value "$telemetry_no_go_policy_output" "reason_codes")"
if ! printf '%s\n' "$telemetry_reason_codes" | grep -q "runtime_submit_success_rate_below_threshold"; then
  echo "expected runtime submit success-rate threshold reason for telemetry NO-GO policy output" >&2
  exit 1
fi
if ! printf '%s\n' "$telemetry_reason_codes" | grep -q "runtime_finality_timeout_threshold_exceeded"; then
  echo "expected runtime finality-timeout threshold reason for telemetry NO-GO policy output" >&2
  exit 1
fi
if ! printf '%s\n' "$telemetry_reason_codes" | grep -q "signer_profile_drift_threshold_exceeded"; then
  echo "expected signer-profile drift threshold reason for telemetry NO-GO policy output" >&2
  exit 1
fi

tampered_bundle="$TMP_DIR/rehearsal-tampered.json"
cp "$no_go_bundle" "$tampered_bundle"
python3 - "$tampered_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text())
payload["final_decision"] = "GO"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
PY

set +e
tampered_output="$(bash "$POLICY_CHECKER" --bundle-file "$tampered_bundle" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered rehearsal decision bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit final-decision mismatch error from rehearsal policy checker" >&2
  exit 1
fi

# Regression: #623
if ! printf '%s\n' "$tampered_output" | grep -q "rollback target hash mismatch"; then
  echo "expected rollback mismatch regression guard to be enforced" >&2
  exit 1
fi

tampered_mttr_bundle="$TMP_DIR/rehearsal-tampered-mttr.json"
cp "$go_bundle" "$tampered_mttr_bundle"
python3 - "$tampered_mttr_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text())
payload["rehearsal"]["mttr_within_bound"] = False
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
PY

set +e
tampered_mttr_output="$(bash "$POLICY_CHECKER" --bundle-file "$tampered_mttr_bundle" 2>&1)"
tampered_mttr_code=$?
set -e

if [ "$tampered_mttr_code" -eq 0 ]; then
  echo "expected tampered MTTR rehearsal bundle to fail policy validation" >&2
  exit 1
fi

# Regression: #2337
if ! printf '%s\n' "$tampered_mttr_output" | grep -q "mttr bound mismatch"; then
  echo "expected explicit MTTR bound mismatch regression guard to be enforced" >&2
  exit 1
fi

echo "staging rehearsal bundle tests passed."
