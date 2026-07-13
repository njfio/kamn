#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
CONFIG_FILE="$ROOT_DIR/fixtures/ci/kolme_manifest_migration_contract_groups.json"
DELETION_MANIFEST="$ROOT_DIR/fixtures/ci/superseded_script_deletion_manifest.json"
MANIFEST_RUNNER="$ROOT_DIR/scripts/framework/run_manifest_lane.sh"
GROUP_KEY="tranche1"
MAX_SECONDS_ENV="KAMN_KOLME_TRANCHE1_DISPATCH_PARITY_MAX_SECONDS"
DEFAULT_MAX_SECONDS=360

test_harness_require_file "$CONFIG_FILE" "expected migration group config file to exist: $CONFIG_FILE"
test_harness_require_file "$DELETION_MANIFEST" "expected deletion manifest to exist: $DELETION_MANIFEST"

test_harness_require_file "$MANIFEST_RUNNER" "expected manifest runner script to exist: $MANIFEST_RUNNER"

max_seconds="${!MAX_SECONDS_ENV:-$DEFAULT_MAX_SECONDS}"
if ! [[ "$max_seconds" =~ ^[0-9]+$ ]] || [ "$max_seconds" -le 0 ]; then
  echo "$MAX_SECONDS_ENV must be a positive integer" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

mapfile -t lane_specs < <(python3 - "$CONFIG_FILE" "$GROUP_KEY" <<'PY'
import json
import sys
from pathlib import Path

config_path = Path(sys.argv[1])
group_key = sys.argv[2]
payload = json.loads(config_path.read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.kolme-manifest-migration-contract-groups.v1":
    raise SystemExit("unexpected migration group schema")
group = payload.get("groups", {}).get(group_key)
if not isinstance(group, dict):
    raise SystemExit(f"missing migration group: {group_key}")
lanes = group.get("lanes")
if not isinstance(lanes, list) or not lanes:
    raise SystemExit(f"migration group {group_key} lanes must be non-empty")
for lane in lanes:
    lane_script = lane.get("lane_script")
    manifest_file = lane.get("manifest_file")
    lane_id = lane.get("lane_id")
    contract_script = lane.get("contract_script")
    if not all(isinstance(field, str) and field for field in (lane_script, manifest_file, lane_id, contract_script)):
        raise SystemExit(f"invalid lane entry in group {group_key}")
    print(f"{lane_script}|{manifest_file}|{lane_id}|{contract_script}")
PY
)

if [ "${#lane_specs[@]}" -eq 0 ]; then
  echo "expected non-empty lane specs for group $GROUP_KEY" >&2
  exit 1
fi

normalize_output() {
  python3 - "$1" <<'PY'
import re
import sys
from pathlib import Path

patterns = (
    re.compile(r"^\s*(Compiling|Checking|Documenting)\s+.+$"),
    re.compile(r"^\s*Finished `.* target\(s\) in .*s$"),
    re.compile(r"^\s*Running tests/.+\(target/.+\)$"),
    re.compile(r"^\s*Blocking waiting for file lock on .+$"),
)
for raw in Path(sys.argv[1]).read_text(encoding="utf-8").splitlines():
    line = raw.rstrip()
    if not line:
        continue
    if any(pattern.match(line) for pattern in patterns):
        continue
    line = re.sub(r"finished in [0-9.]+s$", "finished in <elapsed>s", line)
    print(line)
PY
}

volatile_runtime_left="$TMP_DIR/volatile-runtime-left.out"
volatile_runtime_right="$TMP_DIR/volatile-runtime-right.out"
volatile_runtime_left_normalized="$TMP_DIR/volatile-runtime-left.normalized.out"
volatile_runtime_right_normalized="$TMP_DIR/volatile-runtime-right.normalized.out"
cat >"$volatile_runtime_left" <<'EOF'
running 8 tests
test regression_notifications_consumer_fails_closed_on_decode_and_retry_exhaustion ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
Kolme notifications consumer contract lane tests passed.
status=ok
EOF
cat >"$volatile_runtime_right" <<'EOF'
running 8 tests
test regression_notifications_consumer_fails_closed_on_decode_and_retry_exhaustion ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
Kolme notifications consumer contract lane tests passed.
status=ok
EOF
normalize_output "$volatile_runtime_left" >"$volatile_runtime_left_normalized"
normalize_output "$volatile_runtime_right" >"$volatile_runtime_right_normalized"
if ! diff -u "$volatile_runtime_left_normalized" "$volatile_runtime_right_normalized" >/dev/null; then
  echo "expected tranche-1 output normalizer to ignore volatile Rust test elapsed times" >&2
  diff -u "$volatile_runtime_left_normalized" "$volatile_runtime_right_normalized" >&2 || true
  exit 1
fi

run_parity_lane_command() {
  lane_id="$1"
  shift
  if [ "$lane_id" = "kolme.notifications.consumer.contract" ]; then
    env KAMN_KOLME_NOTIFICATIONS_CONSUMER_MAX_SECONDS=180 "$@"
    return
  fi
  "$@"
}

budget_probe="$TMP_DIR/notifications-budget-probe.sh"
cat >"$budget_probe" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "${KAMN_KOLME_NOTIFICATIONS_CONSUMER_MAX_SECONDS:-unset}"
EOF
chmod +x "$budget_probe"
budget_probe_output="$TMP_DIR/notifications-budget-probe.out"
run_parity_lane_command "kolme.notifications.consumer.contract" "$budget_probe" >"$budget_probe_output"
if ! grep -Fxq "180" "$budget_probe_output"; then
  echo "expected notifications consumer parity runs to use local-heavy budget override" >&2
  cat "$budget_probe_output" >&2 || true
  exit 1
fi

start_epoch="$(date +%s)"

for spec in "${lane_specs[@]}"; do
  IFS='|' read -r lane_script manifest_file lane_id contract_script <<<"$spec"

  lane_script_path="$ROOT_DIR/$lane_script"
  manifest_path="$ROOT_DIR/$manifest_file"
  contract_script_path="$ROOT_DIR/$contract_script"

  if [ -e "$lane_script_path" ]; then
    echo "expected superseded lane script to remain deleted: $lane_script_path" >&2
    exit 1
  fi

  if ! grep -Fq "\"script_path\": \"$lane_script\"" "$DELETION_MANIFEST"; then
    echo "expected deleted lane script in deletion manifest: $lane_script" >&2
    exit 1
  fi

  if [ ! -f "$manifest_path" ]; then
    echo "expected manifest file for parity check: $manifest_path" >&2
    exit 1
  fi

  if [ ! -f "$contract_script_path" ]; then
    echo "expected contract implementation for parity check: $contract_script_path" >&2
    exit 1
  fi

  wrapper_output="$TMP_DIR/${lane_id//./_}.manifest.out"
  direct_output="$TMP_DIR/${lane_id//./_}.direct.out"
  wrapper_normalized="$TMP_DIR/${lane_id//./_}.manifest.normalized.out"
  wrapper_contract_normalized="$TMP_DIR/${lane_id//./_}.manifest-contract.normalized.out"
  direct_normalized="$TMP_DIR/${lane_id//./_}.direct.normalized.out"

  if ! run_parity_lane_command "$lane_id" bash "$MANIFEST_RUNNER" --manifest "$manifest_path" --phase contract >"$wrapper_output" 2>&1; then
    echo "expected manifest lane command to pass for parity check: $manifest_file" >&2
    cat "$wrapper_output" >&2 || true
    exit 1
  fi

  if ! run_parity_lane_command "$lane_id" python3 "$contract_script_path" >"$direct_output" 2>&1; then
    echo "expected direct contract command to pass for parity check: $contract_script" >&2
    cat "$direct_output" >&2 || true
    exit 1
  fi

  normalize_output "$wrapper_output" >"$wrapper_normalized"
  normalize_output "$direct_output" >"$direct_normalized"
  grep -Ev '^(lane_id|phase|exit_code|status)=' "$wrapper_normalized" >"$wrapper_contract_normalized"

  if ! grep -Fxq "lane_id=$lane_id" "$wrapper_normalized"; then
    echo "expected manifest output to include lane_id marker: $lane_id" >&2
    cat "$wrapper_normalized" >&2 || true
    exit 1
  fi

  if ! grep -Fxq "status=ok" "$wrapper_normalized"; then
    echo "expected manifest output to include status=ok for lane: $lane_id" >&2
    cat "$wrapper_normalized" >&2 || true
    exit 1
  fi

  if ! diff -u "$wrapper_contract_normalized" "$direct_normalized" >/dev/null; then
    echo "expected manifest/direct contract outputs to match for lane: $lane_id" >&2
    diff -u "$wrapper_contract_normalized" "$direct_normalized" >&2 || true
    exit 1
  fi

  elapsed_seconds=$(( $(date +%s) - start_epoch ))
  if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
    echo "tranche-1 dispatch execution parity contract exceeded runtime budget: ${elapsed_seconds}s" >&2
    exit 1
  fi

done

echo "Kolme tranche-1 dispatch execution parity contract tests passed."
