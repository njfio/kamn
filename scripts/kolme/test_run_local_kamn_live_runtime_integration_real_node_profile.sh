#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
CI_DOC_FILE="$ROOT_DIR/docs/ci/strategy.md"
README_FILE="$ROOT_DIR/README.md"
TMP_SUMMARY="$(mktemp)"
TMP_ERR="$(mktemp)"
TMP_RUNTIME_SUMMARY="$(mktemp)"
TMP_RUNTIME_POLICY="$(mktemp)"
trap 'rm -f "$TMP_SUMMARY" "$TMP_ERR" "$TMP_RUNTIME_SUMMARY" "$TMP_RUNTIME_POLICY"' EXIT

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

if [ ! -x "$RUNNER" ]; then
  echo "expected local KAMN live runtime integration runner to be executable" >&2
  exit 1
fi

if ! grep -q -- "--runtime-profile" "$RUNNER"; then
  echo "expected local KAMN live runtime integration runner to expose runtime profile option" >&2
  exit 1
fi

if ! grep -q -- "--runtime-profile real-node" "$DOC_FILE"; then
  echo "expected Kolme devnet ops docs to reference real-node runtime profile command marker" >&2
  exit 1
fi

if ! grep -q -- "--runtime-profile real-node" "$CI_DOC_FILE"; then
  echo "expected CI strategy docs to reference real-node runtime profile command marker" >&2
  exit 1
fi

if ! grep -q -- "--runtime-profile real-node" "$README_FILE"; then
  echo "expected README to reference real-node runtime profile command marker" >&2
  exit 1
fi

dry_run_output="$(
  bash "$RUNNER" \
    --mode dry-run \
    --runtime-profile real-node \
    --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider \
    --runtime-commit-live-summary "$TMP_RUNTIME_SUMMARY" \
    --runtime-commit-live-policy-report "$TMP_RUNTIME_POLICY" \
    --output-json "$TMP_SUMMARY"
)"

assert_eq "$(extract_value "$dry_run_output" "status")" "ok" "expected real-node profile dry-run to pass"
assert_eq "$(extract_value "$dry_run_output" "lane_mode")" "dry-run" "expected dry-run mode marker for real-node profile"
assert_eq "$(extract_value "$dry_run_output" "reason_code")" "dry_run_no_commands_executed" "expected dry-run reason marker for real-node profile"
assert_eq "$(extract_value "$dry_run_output" "ci_fast_gate_eligible")" "false" "expected local-only fast-gate marker for real-node profile"

python3 - "$TMP_SUMMARY" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if summary.get("schema_version") != "kamn.kolme.local-kamn-live-runtime-integration-summary.v1":
    raise SystemExit("unexpected integration summary schema for real-node profile")
if summary.get("runtime_profile") != "real-node":
    raise SystemExit("expected runtime_profile=real-node in integration summary")
if summary.get("runtime_provider_client_contract") != "KolmeRuntimeCommitLiveProvider":
    raise SystemExit("expected live provider contract marker in integration summary")
runtime_commit_command = summary.get("runtime_commit_command")
if not isinstance(runtime_commit_command, str):
    raise SystemExit("expected runtime_commit_command to be present in integration summary")
if "--require-non-synthetic-run-evidence" not in runtime_commit_command:
    raise SystemExit("expected strict non-synthetic runtime marker in integration runtime_commit_command")
if "integration_kolme_fork_live_node_submit_reaches_endpoint" not in runtime_commit_command:
    raise SystemExit("expected non-synthetic runtime submit probe marker in integration runtime_commit_command")
if "KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1" not in runtime_commit_command:
    raise SystemExit("expected real signing profile marker in integration runtime_commit_command")
if "KAMN_KOLME_LIVE_SIGNER_PROFILE=ops-primary" not in runtime_commit_command:
    raise SystemExit("expected signer profile marker in integration runtime_commit_command")
if "pubkey" not in runtime_commit_command:
    raise SystemExit("expected native payload pubkey marker in integration runtime_commit_command")
if "nonce" not in runtime_commit_command:
    raise SystemExit("expected native payload nonce marker in integration runtime_commit_command")
if "messages" not in runtime_commit_command:
    raise SystemExit("expected native payload messages marker in integration runtime_commit_command")
if summary.get("runtime_commit_command_profile") != "real-node-non-synthetic-v1":
    raise SystemExit("expected deterministic runtime commit command profile marker for real-node profile")
if summary.get("runtime_commit_policy_command_profile") != "real-node-non-synthetic-v1":
    raise SystemExit("expected deterministic runtime commit policy command profile marker for real-node profile")
if summary.get("runtime_commit_command_profile_version") != "v1":
    raise SystemExit("expected runtime commit command profile marker version for real-node profile")
if summary.get("runtime_signer_profile_selector_env") != "KAMN_KOLME_LIVE_SIGNER_PROFILE":
    raise SystemExit("expected signer profile selector env marker for real-node profile")
if summary.get("runtime_signer_profile") != "ops-primary":
    raise SystemExit("expected signer profile marker for real-node profile")
if summary.get("runtime_signer_private_key_env") != "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX":
    raise SystemExit("expected signer private key env marker for real-node profile")
contracts = summary.get("contracts", {})
if not isinstance(contracts, dict):
    raise SystemExit("expected contracts object in integration summary")
if contracts.get("runtime_profile") != "real-node":
    raise SystemExit("expected contracts.runtime_profile=real-node in integration summary")
if contracts.get("ci_fast_gate_scope") != "local-only":
    raise SystemExit("expected local-only fast-gate scope marker in integration summary")
if contracts.get("runtime_signer_profile_selector_env") != "KAMN_KOLME_LIVE_SIGNER_PROFILE":
    raise SystemExit("expected contracts signer profile selector env marker in integration summary")
if contracts.get("runtime_signer_profile") != "ops-primary":
    raise SystemExit("expected contracts signer profile marker in integration summary")
if contracts.get("runtime_signer_private_key_env") != "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX":
    raise SystemExit("expected contracts signer private key env marker in integration summary")
checks = summary.get("checks", [])
if not isinstance(checks, list) or not checks:
    raise SystemExit("expected checks list in integration summary")
runtime_policy_checks = [
    entry for entry in checks if isinstance(entry, dict) and entry.get("id") == "runtime_commit_policy"
]
if len(runtime_policy_checks) != 1:
    raise SystemExit("expected exactly one runtime_commit_policy check entry in integration summary")
runtime_policy_command = runtime_policy_checks[0].get("command")
if not isinstance(runtime_policy_command, str):
    raise SystemExit("expected runtime_commit_policy check command to be a string")
if "--require-native-payload-evidence" not in runtime_policy_command:
    raise SystemExit("expected native payload evidence marker requirement in runtime policy check command")
PY

set +e
bash "$RUNNER" \
  --mode dry-run \
  --runtime-profile real-node \
  --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider \
  --runtime-commit-command "KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh --expected-provider-client-contract KolmeRuntimeCommitLiveProvider --live-command \"printf 'runtime=in-memory\\n'\" --provider-hint InMemoryKolmeRuntimeCommitClient --output-json $TMP_RUNTIME_SUMMARY --policy-output-json $TMP_RUNTIME_POLICY" \
  --runtime-commit-live-summary "$TMP_RUNTIME_SUMMARY" \
  --runtime-commit-live-policy-report "$TMP_RUNTIME_POLICY" \
  --output-json "$TMP_SUMMARY" >"$TMP_ERR" 2>&1
dry_run_inmemory_exit_code=$?
set -e

if [ "$dry_run_inmemory_exit_code" -eq 0 ]; then
  echo "expected real-node profile dry-run to fail closed when runtime commit command references InMemory provider" >&2
  exit 1
fi

if ! grep -q "must not reference InMemoryKolmeRuntimeCommitClient" "$TMP_ERR"; then
  echo "expected deterministic in-memory provider rejection message for real-node profile dry-run" >&2
  exit 1
fi

set +e
bash "$RUNNER" \
  --mode run \
  --runtime-profile real-node \
  --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider \
  --runtime-commit-live-summary "$TMP_RUNTIME_SUMMARY" \
  --runtime-commit-live-policy-report "$TMP_RUNTIME_POLICY" \
  --output-json "$TMP_SUMMARY" >"$TMP_ERR" 2>&1
run_without_opt_in_code=$?
set -e

if [ "$run_without_opt_in_code" -eq 0 ]; then
  echo "expected real-node profile run mode without opt-in to fail closed" >&2
  exit 1
fi

if ! grep -q "requires explicit local-only opt-in" "$TMP_ERR"; then
  echo "expected deterministic opt-in failure message for real-node profile run mode" >&2
  exit 1
fi

echo "local KAMN live runtime integration real-node profile tests passed."
