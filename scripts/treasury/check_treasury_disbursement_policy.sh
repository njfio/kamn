#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/treasury/check_treasury_disbursement_policy.sh \
    --bundle-file <path>
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

bundle_file=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bundle-file)
      bundle_file="${2:-}"
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

if [[ -z "$bundle_file" ]]; then
  usage
  fail "--bundle-file is required"
fi

if [[ ! -f "$bundle_file" ]]; then
  fail "bundle file not found: $bundle_file"
fi

output="$(
  python3 - "$bundle_file" <<'PY'
import json
import pathlib
import re
import sys


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    sys.exit(1)


bundle_path = pathlib.Path(sys.argv[1])
try:
    payload = json.loads(bundle_path.read_text())
except json.JSONDecodeError as exc:
    fail(f"bundle file is not valid JSON: {exc}")

required_fields = (
    "schema_version",
    "generated_at",
    "disbursement",
    "approvals",
    "policy_window_open",
    "ci_fast_gate",
    "decision_reasons",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

if payload["schema_version"] != "kamn.treasury.disbursement-approval.v1":
    fail("unexpected schema_version for treasury disbursement approval evidence bundle")

disbursement = payload["disbursement"]
if not isinstance(disbursement, dict):
    fail("bundle field 'disbursement' must be an object")
for field in (
    "disbursement_id",
    "treasury_account_id",
    "destination_account_id",
    "asset_symbol",
    "amount",
    "daily_limit_amount",
):
    if field not in disbursement:
        fail(f"disbursement object missing field: {field}")

for field in (
    "disbursement_id",
    "treasury_account_id",
    "destination_account_id",
    "asset_symbol",
):
    if not isinstance(disbursement[field], str):
        fail(f"disbursement.{field} must be a string")

if not re.fullmatch(r"[A-Z0-9]+", disbursement["asset_symbol"]):
    fail("disbursement.asset_symbol must be uppercase alphanumeric")

for field in ("amount", "daily_limit_amount"):
    if not isinstance(disbursement[field], int):
        fail(f"disbursement.{field} must be an integer")
    if disbursement[field] < 0:
        fail(f"disbursement.{field} must be >= 0")

approvals = payload["approvals"]
if not isinstance(approvals, dict):
    fail("bundle field 'approvals' must be an object")
for field in ("required", "received", "approval_quorum_hash"):
    if field not in approvals:
        fail(f"approvals object missing field: {field}")

for field in ("required", "received"):
    if not isinstance(approvals[field], int):
        fail(f"approvals.{field} must be an integer")
    if approvals[field] < 0:
        fail(f"approvals.{field} must be >= 0")

if not isinstance(approvals["approval_quorum_hash"], str):
    fail("approvals.approval_quorum_hash must be a string")

if not isinstance(payload["policy_window_open"], bool):
    fail("policy_window_open must be a boolean")
if payload["ci_fast_gate"] not in {"PASS", "FAIL"}:
    fail("ci_fast_gate must be PASS or FAIL")
if not isinstance(payload["decision_reasons"], list):
    fail("decision_reasons must be an array")
if not all(isinstance(item, str) for item in payload["decision_reasons"]):
    fail("decision_reasons entries must be strings")

expected_go = True
if disbursement["amount"] <= 0:
    expected_go = False
if disbursement["daily_limit_amount"] <= 0:
    expected_go = False
if disbursement["amount"] > disbursement["daily_limit_amount"]:
    expected_go = False
if approvals["required"] <= 0:
    expected_go = False
if approvals["received"] < approvals["required"]:
    expected_go = False
if not approvals["approval_quorum_hash"].startswith("sha256:") or len(approvals["approval_quorum_hash"]) <= len("sha256:"):
    expected_go = False
if not payload["policy_window_open"]:
    expected_go = False
if payload["ci_fast_gate"] != "PASS":
    expected_go = False

expected_decision = "GO" if expected_go else "NO-GO"
actual_decision = payload["final_decision"]
if actual_decision not in {"GO", "NO-GO"}:
    fail("final_decision must be GO or NO-GO")
if actual_decision != expected_decision:
    fail(
        "policy decision mismatch: "
        f"expected final_decision={expected_decision}, found {actual_decision}"
    )

print("status=ok")
print(f"bundle_file={bundle_path}")
print(f"final_decision={actual_decision}")
PY
)"

printf '%s\n' "$output"
