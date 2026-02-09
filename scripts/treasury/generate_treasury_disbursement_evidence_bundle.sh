#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/treasury/generate_treasury_disbursement_evidence_bundle.sh \
    --output-file <path> \
    --disbursement-id <value> \
    --treasury-account-id <value> \
    --destination-account-id <value> \
    --asset-symbol <value> \
    --disbursement-amount <n> \
    --daily-limit-amount <n> \
    --required-approvals <n> \
    --received-approvals <n> \
    --approval-quorum-hash <sha256:...> \
    --policy-window-open true|false \
    --ci-fast-gate PASS|FAIL
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

require_int() {
  local name="$1"
  local value="$2"
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    fail "${name} must be an integer"
  fi
}

output_file=""
disbursement_id=""
treasury_account_id=""
destination_account_id=""
asset_symbol=""
disbursement_amount=""
daily_limit_amount=""
required_approvals=""
received_approvals=""
approval_quorum_hash=""
policy_window_open=""
ci_fast_gate=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --disbursement-id)
      disbursement_id="${2:-}"
      shift 2
      ;;
    --treasury-account-id)
      treasury_account_id="${2:-}"
      shift 2
      ;;
    --destination-account-id)
      destination_account_id="${2:-}"
      shift 2
      ;;
    --asset-symbol)
      asset_symbol="${2:-}"
      shift 2
      ;;
    --disbursement-amount)
      disbursement_amount="${2:-}"
      shift 2
      ;;
    --daily-limit-amount)
      daily_limit_amount="${2:-}"
      shift 2
      ;;
    --required-approvals)
      required_approvals="${2:-}"
      shift 2
      ;;
    --received-approvals)
      received_approvals="${2:-}"
      shift 2
      ;;
    --approval-quorum-hash)
      approval_quorum_hash="${2:-}"
      shift 2
      ;;
    --policy-window-open)
      policy_window_open="${2:-}"
      shift 2
      ;;
    --ci-fast-gate)
      ci_fast_gate="${2:-}"
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

if [[ -z "$output_file" || -z "$disbursement_id" || -z "$treasury_account_id" || -z "$destination_account_id" || -z "$asset_symbol" || -z "$disbursement_amount" || -z "$daily_limit_amount" || -z "$required_approvals" || -z "$received_approvals" || -z "$approval_quorum_hash" || -z "$policy_window_open" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all treasury disbursement evidence bundle arguments are required"
fi

require_int "disbursement-amount" "$disbursement_amount"
require_int "daily-limit-amount" "$daily_limit_amount"
require_int "required-approvals" "$required_approvals"
require_int "received-approvals" "$received_approvals"

if [[ "$policy_window_open" != "true" && "$policy_window_open" != "false" ]]; then
  fail "policy-window-open must be true or false"
fi

mkdir -p "$(dirname "$output_file")"

generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$disbursement_id" "$treasury_account_id" "$destination_account_id" "$asset_symbol" "$disbursement_amount" "$daily_limit_amount" "$required_approvals" "$received_approvals" "$approval_quorum_hash" "$policy_window_open" "$ci_fast_gate" <<'PY'
import json
import pathlib
import re
import sys


def fail(message: str) -> None:
    raise ValueError(message)


(
    output_file,
    generated_at,
    disbursement_id,
    treasury_account_id,
    destination_account_id,
    asset_symbol,
    disbursement_amount_raw,
    daily_limit_amount_raw,
    required_approvals_raw,
    received_approvals_raw,
    approval_quorum_hash,
    policy_window_open_raw,
    ci_fast_gate,
) = sys.argv[1:]

if ci_fast_gate not in {"PASS", "FAIL"}:
    fail("ci-fast-gate must be PASS or FAIL")

if policy_window_open_raw not in {"true", "false"}:
    fail("policy-window-open must be true or false")
policy_window_open = policy_window_open_raw == "true"

if not re.fullmatch(r"[A-Z0-9]+", asset_symbol or ""):
    fail("asset-symbol must be uppercase alphanumeric")

disbursement_amount = int(disbursement_amount_raw)
daily_limit_amount = int(daily_limit_amount_raw)
required_approvals = int(required_approvals_raw)
received_approvals = int(received_approvals_raw)

decision_reasons: list[str] = []

if disbursement_amount <= 0:
    decision_reasons.append("disbursement amount must be greater than zero")
if daily_limit_amount <= 0:
    decision_reasons.append("daily limit amount must be greater than zero")
if disbursement_amount > daily_limit_amount:
    decision_reasons.append("disbursement amount exceeds daily limit amount")
if required_approvals <= 0:
    decision_reasons.append("required approvals must be greater than zero")
if received_approvals < required_approvals:
    decision_reasons.append("received approvals are below required approvals")
if not approval_quorum_hash.startswith("sha256:") or len(approval_quorum_hash) <= len("sha256:"):
    decision_reasons.append("approval quorum hash must be a non-empty sha256 digest")
if not policy_window_open:
    decision_reasons.append("policy approval window is closed")
if ci_fast_gate != "PASS":
    decision_reasons.append("ci-fast-gate-failed")

final_decision = "GO" if not decision_reasons else "NO-GO"
if not decision_reasons:
    decision_reasons.append("all treasury disbursement approval gates satisfied")

payload = {
    "schema_version": "kamn.treasury.disbursement-approval.v1",
    "generated_at": generated_at,
    "disbursement": {
        "disbursement_id": disbursement_id,
        "treasury_account_id": treasury_account_id,
        "destination_account_id": destination_account_id,
        "asset_symbol": asset_symbol,
        "amount": disbursement_amount,
        "daily_limit_amount": daily_limit_amount,
    },
    "approvals": {
        "required": required_approvals,
        "received": received_approvals,
        "approval_quorum_hash": approval_quorum_hash,
    },
    "policy_window_open": policy_window_open,
    "ci_fast_gate": ci_fast_gate,
    "decision_reasons": decision_reasons,
    "final_decision": final_decision,
}

path = pathlib.Path(output_file)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
print(final_decision)
PY
)"

printf 'status=generated\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'final_decision=%s\n' "$final_decision"
