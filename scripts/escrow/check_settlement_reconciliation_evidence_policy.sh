#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/escrow/check_settlement_reconciliation_evidence_policy.sh \
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
    "escrow_id",
    "settlement_outcome",
    "receipt",
    "expected_amounts",
    "observed_amounts",
    "timeout_elapsed",
    "ci_fast_gate",
    "decision_reasons",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

if payload["schema_version"] != "kamn.escrow.settlement-reconciliation.v1":
    fail("unexpected schema_version for settlement reconciliation evidence bundle")

settlement_outcome = payload["settlement_outcome"]
if settlement_outcome not in {"RELEASED", "REFUNDED", "TIMEOUT_REFUNDED", "DISPUTED_RESOLVED"}:
    fail("settlement_outcome must be RELEASED|REFUNDED|TIMEOUT_REFUNDED|DISPUTED_RESOLVED")

receipt = payload["receipt"]
if not isinstance(receipt, dict):
    fail("bundle field 'receipt' must be an object")
if "receipt_id" not in receipt or "finality" not in receipt:
    fail("receipt object must contain receipt_id and finality")
if not isinstance(receipt["receipt_id"], str):
    fail("receipt.receipt_id must be a string")
if receipt["finality"] not in {"FINAL", "PENDING", "FAILED"}:
    fail("receipt.finality must be FINAL|PENDING|FAILED")

expected_amounts = payload["expected_amounts"]
observed_amounts = payload["observed_amounts"]
if not isinstance(expected_amounts, dict) or not isinstance(observed_amounts, dict):
    fail("expected_amounts and observed_amounts must be objects")
for section_name, section in (("expected_amounts", expected_amounts), ("observed_amounts", observed_amounts)):
    for field in ("release", "refund"):
        if field not in section:
            fail(f"missing {section_name}.{field}")
        if not isinstance(section[field], int):
            fail(f"{section_name}.{field} must be an integer")
        if section[field] < 0:
            fail(f"{section_name}.{field} must be >= 0")

if not isinstance(payload["timeout_elapsed"], bool):
    fail("timeout_elapsed must be a boolean")
if payload["ci_fast_gate"] not in {"PASS", "FAIL"}:
    fail("ci_fast_gate must be PASS or FAIL")

if not isinstance(payload["decision_reasons"], list):
    fail("decision_reasons must be an array")
if not all(isinstance(item, str) for item in payload["decision_reasons"]):
    fail("decision_reasons entries must be strings")

expected_go = True
if not receipt["receipt_id"].strip() or receipt["finality"] != "FINAL":
    expected_go = False
if expected_amounts["release"] != observed_amounts["release"] or expected_amounts["refund"] != observed_amounts["refund"]:
    expected_go = False
if settlement_outcome == "TIMEOUT_REFUNDED" and not payload["timeout_elapsed"]:
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
