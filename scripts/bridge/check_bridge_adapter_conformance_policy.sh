#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/bridge/check_bridge_adapter_conformance_policy.sh \
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


def ensure_string_list(value: object, field_name: str) -> list[str]:
    if not isinstance(value, list):
        fail(f"{field_name} must be an array")
    if not all(isinstance(item, str) for item in value):
        fail(f"{field_name} entries must be strings")
    normalized = [item.strip() for item in value if item.strip()]
    return normalized


def normalize_set(values: list[str]) -> list[str]:
    return sorted(set(values))


bundle_path = pathlib.Path(sys.argv[1])
try:
    payload = json.loads(bundle_path.read_text())
except json.JSONDecodeError as exc:
    fail(f"bundle file is not valid JSON: {exc}")

required_fields = (
    "schema_version",
    "generated_at",
    "adapter_id",
    "bridge_network",
    "dry_run",
    "request_contract",
    "receipt_contract",
    "ci_fast_gate",
    "reason_key",
    "reason_codes",
    "decision_reasons",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

if payload["schema_version"] != "kamn.bridge.adapter-conformance.v1":
    fail("unexpected schema_version for bridge adapter conformance evidence bundle")

if not isinstance(payload["adapter_id"], str) or not payload["adapter_id"].strip():
    fail("adapter_id must be a non-empty string")

if payload["bridge_network"] not in {"ethereum", "solana", "near", "custom"}:
    fail("bridge_network must be ethereum|solana|near|custom")

if not isinstance(payload["dry_run"], bool):
    fail("dry_run must be a boolean")

if payload["ci_fast_gate"] not in {"PASS", "FAIL"}:
    fail("ci_fast_gate must be PASS or FAIL")

request_contract = payload["request_contract"]
if not isinstance(request_contract, dict):
    fail("request_contract must be an object")
receipt_contract = payload["receipt_contract"]
if not isinstance(receipt_contract, dict):
    fail("receipt_contract must be an object")

for section_name, section in (("request_contract", request_contract), ("receipt_contract", receipt_contract)):
    for key in (
        "expected_schema_version",
        "observed_schema_version",
        "required_fields",
        "observed_fields",
        "missing_required_fields",
    ):
        if key not in section:
            fail(f"{section_name} missing field: {key}")
    if not isinstance(section["expected_schema_version"], str):
        fail(f"{section_name}.expected_schema_version must be a string")
    if not isinstance(section["observed_schema_version"], str):
        fail(f"{section_name}.observed_schema_version must be a string")

request_required_fields = normalize_set(
    ensure_string_list(request_contract["required_fields"], "request_contract.required_fields")
)
request_observed_fields = normalize_set(
    ensure_string_list(request_contract["observed_fields"], "request_contract.observed_fields")
)
request_missing_required_fields = normalize_set(
    ensure_string_list(
        request_contract["missing_required_fields"], "request_contract.missing_required_fields"
    )
)

receipt_required_fields = normalize_set(
    ensure_string_list(receipt_contract["required_fields"], "receipt_contract.required_fields")
)
receipt_observed_fields = normalize_set(
    ensure_string_list(receipt_contract["observed_fields"], "receipt_contract.observed_fields")
)
receipt_missing_required_fields = normalize_set(
    ensure_string_list(
        receipt_contract["missing_required_fields"], "receipt_contract.missing_required_fields"
    )
)

computed_request_missing_required_fields = sorted(
    [field for field in request_required_fields if field not in set(request_observed_fields)]
)
computed_receipt_missing_required_fields = sorted(
    [field for field in receipt_required_fields if field not in set(receipt_observed_fields)]
)

if request_missing_required_fields != computed_request_missing_required_fields:
    fail(
        "request missing_required_fields mismatch: "
        f"expected {computed_request_missing_required_fields}, found {request_missing_required_fields}"
    )
if receipt_missing_required_fields != computed_receipt_missing_required_fields:
    fail(
        "receipt missing_required_fields mismatch: "
        f"expected {computed_receipt_missing_required_fields}, found {receipt_missing_required_fields}"
    )

reason_key = payload["reason_key"]
if not isinstance(reason_key, str):
    fail("reason_key must be a string")
reason_codes = ensure_string_list(payload["reason_codes"], "reason_codes")
decision_reasons = ensure_string_list(payload["decision_reasons"], "decision_reasons")

expected_reason_codes: list[str] = []
if not payload["dry_run"]:
    expected_reason_codes.append("dry_run_disabled")
if request_contract["expected_schema_version"] != request_contract["observed_schema_version"]:
    expected_reason_codes.append("request_schema_version_mismatch")
if receipt_contract["expected_schema_version"] != receipt_contract["observed_schema_version"]:
    expected_reason_codes.append("receipt_schema_version_mismatch")
if not request_required_fields:
    expected_reason_codes.append("request_required_fields_contract_missing")
if computed_request_missing_required_fields:
    expected_reason_codes.append("request_required_fields_missing")
if not receipt_required_fields:
    expected_reason_codes.append("receipt_required_fields_contract_missing")
if computed_receipt_missing_required_fields:
    expected_reason_codes.append("receipt_required_fields_missing")
if payload["ci_fast_gate"] != "PASS":
    expected_reason_codes.append("ci_fast_gate_failed")

expected_final_decision = "GO"
if expected_reason_codes:
    expected_final_decision = "NO-GO"
else:
    expected_reason_codes = [
        "adapter_conformance_dry_run_mode",
        "adapter_request_receipt_contracts_compatible",
    ]

actual_final_decision = payload["final_decision"]
if actual_final_decision not in {"GO", "NO-GO"}:
    fail("final_decision must be GO or NO-GO")
if actual_final_decision != expected_final_decision:
    fail(
        "policy decision mismatch: "
        f"expected final_decision={expected_final_decision}, found {actual_final_decision}"
    )

if reason_codes != expected_reason_codes:
    fail(
        "reason_codes mismatch: "
        f"expected {expected_reason_codes}, found {reason_codes}"
    )

if decision_reasons != expected_reason_codes:
    fail(
        "decision_reasons mismatch: "
        f"expected {expected_reason_codes}, found {decision_reasons}"
    )

expected_reason_key = f"bridge_adapter_conformance_reason_codes:{expected_final_decision}:v1"
if reason_key != expected_reason_key:
    fail(
        "reason_key mismatch: "
        f"expected {expected_reason_key}, found {reason_key}"
    )

print("status=ok")
print(f"bundle_file={bundle_path}")
print(f"final_decision={actual_final_decision}")
print(f"reason_key={reason_key}")
print(f"decision_reasons={'; '.join(expected_reason_codes)}")
PY
)"

printf '%s\n' "$output"
