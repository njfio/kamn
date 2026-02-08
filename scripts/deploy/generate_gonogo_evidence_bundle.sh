#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/deploy/generate_gonogo_evidence_bundle.sh \
    --output-file <path> \
    --release-candidate <value> \
    --schema-target-version <value> \
    --runtime-image-digest <value> \
    --ci-fast-gate PASS|FAIL \
    --ci-deep-lane PASS|FAIL \
    --rollback-precheck PASS|FAIL \
    --rollback-trigger-status CLEAR|TRIGGERED \
    --required-approvals <n> \
    --received-approvals <n>
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
release_candidate=""
schema_target_version=""
runtime_image_digest=""
ci_fast_gate=""
ci_deep_lane=""
rollback_precheck=""
rollback_trigger_status=""
required_approvals=""
received_approvals=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --release-candidate)
      release_candidate="${2:-}"
      shift 2
      ;;
    --schema-target-version)
      schema_target_version="${2:-}"
      shift 2
      ;;
    --runtime-image-digest)
      runtime_image_digest="${2:-}"
      shift 2
      ;;
    --ci-fast-gate)
      ci_fast_gate="${2:-}"
      shift 2
      ;;
    --ci-deep-lane)
      ci_deep_lane="${2:-}"
      shift 2
      ;;
    --rollback-precheck)
      rollback_precheck="${2:-}"
      shift 2
      ;;
    --rollback-trigger-status)
      rollback_trigger_status="${2:-}"
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
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument: $1"
      ;;
  esac
done

if [[ -z "$output_file" || -z "$release_candidate" || -z "$schema_target_version" || -z "$runtime_image_digest" || -z "$ci_fast_gate" || -z "$ci_deep_lane" || -z "$rollback_precheck" || -z "$rollback_trigger_status" || -z "$required_approvals" || -z "$received_approvals" ]]; then
  usage
  fail "all bundle arguments are required"
fi

require_int "required-approvals" "$required_approvals"
require_int "received-approvals" "$received_approvals"

if (( required_approvals < 1 )); then
  fail "required-approvals must be >= 1"
fi

mkdir -p "$(dirname "$output_file")"

generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$release_candidate" "$schema_target_version" "$runtime_image_digest" "$ci_fast_gate" "$ci_deep_lane" "$rollback_precheck" "$rollback_trigger_status" "$required_approvals" "$received_approvals" <<'PY'
import json
import pathlib
import sys


def fail(message: str) -> None:
    raise ValueError(message)


(
    output_file,
    generated_at,
    release_candidate,
    schema_target_version,
    runtime_image_digest,
    ci_fast_gate,
    ci_deep_lane,
    rollback_precheck,
    rollback_trigger_status,
    required_approvals_raw,
    received_approvals_raw,
) = sys.argv[1:]

required_approvals = int(required_approvals_raw)
received_approvals = int(received_approvals_raw)

if received_approvals < 0:
    fail("received-approvals must be >= 0")

for field_name, status in (
    ("ci-fast-gate", ci_fast_gate),
    ("ci-deep-lane", ci_deep_lane),
    ("rollback-precheck", rollback_precheck),
):
    if status not in {"PASS", "FAIL"}:
        fail(f"{field_name} must be PASS or FAIL")

if rollback_trigger_status not in {"CLEAR", "TRIGGERED"}:
    fail("rollback-trigger-status must be CLEAR or TRIGGERED")

is_go = (
    ci_fast_gate == "PASS"
    and ci_deep_lane == "PASS"
    and rollback_precheck == "PASS"
    and rollback_trigger_status == "CLEAR"
    and received_approvals >= required_approvals
)
final_decision = "GO" if is_go else "NO-GO"

payload = {
    "schema_version": "kamn.release.gonogo.v1",
    "generated_at": generated_at,
    "release_candidate": release_candidate,
    "schema_target_version": schema_target_version,
    "runtime_image_digest": runtime_image_digest,
    "gates": {
        "ci_fast_gate": ci_fast_gate,
        "ci_deep_lane": ci_deep_lane,
        "rollback_precheck": rollback_precheck,
    },
    "rollback_trigger_status": rollback_trigger_status,
    "approvals": {
        "required": required_approvals,
        "received": received_approvals,
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
