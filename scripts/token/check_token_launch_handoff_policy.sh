#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/token/check_token_launch_handoff_policy.sh \
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
    "token_symbol",
    "supply",
    "allocations",
    "genesis",
    "approvals",
    "ci_fast_gate",
    "decision_reasons",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

if payload["schema_version"] != "kamn.token.launch-handoff.v1":
    fail("unexpected schema_version for token launch handoff evidence bundle")

token_symbol = payload["token_symbol"]
if not isinstance(token_symbol, str) or not re.fullmatch(r"[A-Z0-9]+", token_symbol):
    fail("token_symbol must be uppercase alphanumeric")

supply = payload["supply"]
if not isinstance(supply, dict):
    fail("bundle field 'supply' must be an object")
if "configured_total_supply" not in supply or "expected_total_supply" not in supply:
    fail("supply object must contain configured_total_supply and expected_total_supply")

allocations = payload["allocations"]
if not isinstance(allocations, dict):
    fail("bundle field 'allocations' must be an object")
for field in ("configured_sum", "expected_sum", "bucket_count", "expected_bucket_count"):
    if field not in allocations:
        fail(f"allocations object missing field: {field}")

for section_name, section in (
    ("supply", supply),
    ("allocations", allocations),
):
    for key, value in section.items():
        if not isinstance(value, int):
            fail(f"{section_name}.{key} must be an integer")
        if value < 0:
            fail(f"{section_name}.{key} must be >= 0")

genesis = payload["genesis"]
if not isinstance(genesis, dict):
    fail("bundle field 'genesis' must be an object")
genesis_hash = genesis.get("genesis_hash")
if not isinstance(genesis_hash, str):
    fail("genesis.genesis_hash must be a string")

approvals = payload["approvals"]
if not isinstance(approvals, dict):
    fail("bundle field 'approvals' must be an object")
for field in ("required", "received"):
    if field not in approvals:
        fail(f"approvals object missing field: {field}")
    if not isinstance(approvals[field], int):
        fail(f"approvals.{field} must be an integer")
    if approvals[field] < 0:
        fail(f"approvals.{field} must be >= 0")

if payload["ci_fast_gate"] not in {"PASS", "FAIL"}:
    fail("ci_fast_gate must be PASS or FAIL")

if not isinstance(payload["decision_reasons"], list):
    fail("decision_reasons must be an array")
if not all(isinstance(item, str) for item in payload["decision_reasons"]):
    fail("decision_reasons entries must be strings")

expected_go = True
if supply["configured_total_supply"] != supply["expected_total_supply"]:
    expected_go = False
if allocations["configured_sum"] != allocations["expected_sum"]:
    expected_go = False
if allocations["configured_sum"] != supply["configured_total_supply"]:
    expected_go = False
if allocations["expected_sum"] != supply["expected_total_supply"]:
    expected_go = False
if allocations["bucket_count"] != allocations["expected_bucket_count"] or allocations["bucket_count"] <= 0:
    expected_go = False
if approvals["required"] <= 0:
    expected_go = False
if approvals["received"] < approvals["required"]:
    expected_go = False
if not genesis_hash.startswith("sha256:") or len(genesis_hash) <= len("sha256:"):
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
