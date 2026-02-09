#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/compliance/generate_soc2_control_evidence_bundle.sh \
    --output-file <path> \
    --control-id <value> \
    --audit-period-start YYYY-MM-DD \
    --audit-period-end YYYY-MM-DD \
    --collector-did <value> \
    --evidence-uri <value> \
    --evidence-sha256 sha256:<64-hex> \
    --tamper-check PASS|FAIL \
    --completeness-check PASS|FAIL \
    --ci-fast-gate PASS|FAIL
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

output_file=""
control_id=""
audit_period_start=""
audit_period_end=""
collector_did=""
evidence_uri=""
evidence_sha256=""
tamper_check=""
completeness_check=""
ci_fast_gate=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --control-id)
      control_id="${2:-}"
      shift 2
      ;;
    --audit-period-start)
      audit_period_start="${2:-}"
      shift 2
      ;;
    --audit-period-end)
      audit_period_end="${2:-}"
      shift 2
      ;;
    --collector-did)
      collector_did="${2:-}"
      shift 2
      ;;
    --evidence-uri)
      evidence_uri="${2:-}"
      shift 2
      ;;
    --evidence-sha256)
      evidence_sha256="${2:-}"
      shift 2
      ;;
    --tamper-check)
      tamper_check="${2:-}"
      shift 2
      ;;
    --completeness-check)
      completeness_check="${2:-}"
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

if [[ -z "$output_file" || -z "$control_id" || -z "$audit_period_start" || -z "$audit_period_end" || -z "$collector_did" || -z "$evidence_uri" || -z "$evidence_sha256" || -z "$tamper_check" || -z "$completeness_check" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all bundle arguments are required"
fi

for field in "$tamper_check" "$completeness_check" "$ci_fast_gate"; do
  if [[ "$field" != "PASS" && "$field" != "FAIL" ]]; then
    fail "check statuses must be PASS or FAIL"
  fi
done

mkdir -p "$(dirname "$output_file")"

generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$control_id" "$audit_period_start" "$audit_period_end" "$collector_did" "$evidence_uri" "$evidence_sha256" "$tamper_check" "$completeness_check" "$ci_fast_gate" <<'PY'
import json
import pathlib
import re
import sys

(
    output_file,
    generated_at,
    control_id,
    audit_period_start,
    audit_period_end,
    collector_did,
    evidence_uri,
    evidence_sha256,
    tamper_check,
    completeness_check,
    ci_fast_gate,
) = sys.argv[1:]

date_pattern = re.compile(r"^\d{4}-\d{2}-\d{2}$")
hash_pattern = re.compile(r"^sha256:[0-9a-f]{64}$")

period_valid = bool(date_pattern.match(audit_period_start)) and bool(
    date_pattern.match(audit_period_end)
) and audit_period_start <= audit_period_end
hash_valid = bool(hash_pattern.match(evidence_sha256))

is_go = (
    tamper_check == "PASS"
    and completeness_check == "PASS"
    and ci_fast_gate == "PASS"
    and period_valid
    and hash_valid
)
final_decision = "GO" if is_go else "NO-GO"

payload = {
    "schema_version": "kamn.compliance.soc2-control-evidence.v1",
    "generated_at": generated_at,
    "control_id": control_id,
    "audit_period": {
        "start": audit_period_start,
        "end": audit_period_end,
    },
    "collector_did": collector_did,
    "evidence_uri": evidence_uri,
    "evidence_sha256": evidence_sha256,
    "checks": {
        "tamper": tamper_check,
        "completeness": completeness_check,
        "ci_fast_gate": ci_fast_gate,
        "period_valid": period_valid,
        "hash_valid": hash_valid,
    },
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

