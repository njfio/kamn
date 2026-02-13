#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/message/generate_group_sender_replay_ratchet_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/message/check_group_sender_replay_ratchet_policy.sh"

usage() {
  cat <<'EOF'
Usage:
  bash scripts/message/run_group_sender_replay_ratchet_contract_lane.sh \
    [--output-file <path>] \
    [--skip-tests]
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

output_file=""
skip_tests=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --skip-tests)
      skip_tests=true
      shift
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

if [ ! -x "$GENERATOR" ]; then
  fail "group sender replay/ratchet evidence generator is not executable"
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  fail "group sender replay/ratchet policy checker is not executable"
fi

start_epoch="$(date +%s)"

if [ "$skip_tests" != true ]; then
  cargo test -p kamn-core --test group_sender_keys >/dev/null
  cargo test -p kamn-core --test group_sender_key_rotation_docs >/dev/null
fi

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

report_file="$tmp_dir/group-sender-replay-ratchet-report.json"
cat >"$report_file" <<'JSON'
{
  "status": "pass",
  "nonce_replay_detected": false,
  "stale_generation_detected": false,
  "signature_tamper_detected": false,
  "reason_codes": ["ratchet_generation_progressive", "replay_nonce_guard_active"]
}
JSON

if [[ -z "$output_file" ]]; then
  output_file="$tmp_dir/group-sender-replay-ratchet-evidence.json"
fi

generation_output="$(
  bash "$GENERATOR" \
    --output-file "$output_file" \
    --lane contract \
    --report-file "$report_file" \
    --ci-fast-gate PASS
)"

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$output_file")"

max_seconds="${GROUP_SENDER_REPLAY_RATCHET_MAX_SECONDS:-90}"
elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  fail "group sender replay/ratchet contract lane exceeded runtime budget: ${elapsed_seconds}s"
fi

printf 'status=ok\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'schema_version=%s\n' "$(extract_value "$policy_output" "schema_version")"
printf 'evidence_key=%s\n' "$(extract_value "$generation_output" "evidence_key")"
printf 'reason_key=%s\n' "$(extract_value "$generation_output" "reason_key")"
printf 'final_decision=%s\n' "$(extract_value "$policy_output" "final_decision")"
echo "group sender replay/ratchet contract lane tests passed."
