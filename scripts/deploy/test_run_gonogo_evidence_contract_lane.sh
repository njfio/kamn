#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/deploy/run_gonogo_evidence_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/deploy/run_gonogo_evidence_deep_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/deploy/gonogo_evidence_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/deploy_gonogo_evidence_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected go/no-go evidence fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected go/no-go evidence deep-lane runner to be executable" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected go/no-go evidence shared contract-lane module to be executable" >&2
  exit 1
fi

tmp_out="$(mktemp)"
trap 'rm -f "$tmp_out"' EXIT

bash "$FAST_SCRIPT" >"$tmp_out"
if ! grep -q "go/no-go evidence contract lane tests passed." "$tmp_out"; then
  echo "expected go/no-go evidence contract lane success marker" >&2
  exit 1
fi
if ! grep -q "^incident_gonogo_boundary_reason_taxonomy_status=verified$" "$tmp_out"; then
  echo "expected go/no-go evidence contract lane to emit incident boundary reason taxonomy status marker" >&2
  exit 1
fi
if ! grep -q "^incident_gonogo_boundary_reason_taxonomy_version=kamn.release.gonogo-incident-boundary-reason-taxonomy.v1$" "$tmp_out"; then
  echo "expected go/no-go evidence contract lane to emit deterministic incident boundary reason taxonomy version marker" >&2
  exit 1
fi
if ! grep -q "^incident_gonogo_boundary_reason_codes_csv=incident_gonogo_ci_smoke_seconds_exceeded,incident_gonogo_local_heavy_seconds_exceeded,incident_gonogo_local_heavy_opt_in_missing,incident_gonogo_evidence_convergence_mismatch$" "$tmp_out"; then
  echo "expected go/no-go evidence contract lane to emit deterministic incident boundary reason-code taxonomy marker" >&2
  exit 1
fi
if ! grep -q "^incident_gonogo_ci_smoke_max_seconds=120$" "$tmp_out"; then
  echo "expected go/no-go evidence contract lane to emit CI smoke boundary max-seconds marker" >&2
  exit 1
fi
if ! grep -q "^incident_gonogo_local_heavy_max_seconds=900$" "$tmp_out"; then
  echo "expected go/no-go evidence contract lane to emit local-heavy boundary max-seconds marker" >&2
  exit 1
fi
if ! grep -q "^ci_smoke_lane_cost_profile=low$" "$tmp_out"; then
  echo "expected go/no-go evidence contract lane to declare low-cost CI smoke profile marker" >&2
  exit 1
fi

if [ ! -L "$FAST_SCRIPT" ]; then
  echo "expected go/no-go evidence contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$FAST_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected go/no-go evidence contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$FAST_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected go/no-go evidence wrapper to resolve deploy manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "gonogo_evidence_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected go/no-go evidence manifest to dispatch shared contract module" >&2
  exit 1
fi

if ! grep -q "generate_gonogo_evidence_bundle.sh" "$SHARED_CONTRACT"; then
  echo "expected go/no-go evidence shared contract module to execute evidence bundle generator" >&2
  exit 1
fi

if ! grep -q "check_gonogo_evidence_policy.sh" "$SHARED_CONTRACT"; then
  echo "expected go/no-go evidence shared contract module to execute policy checker" >&2
  exit 1
fi

if ! grep -q -- "--audit-integrity-report-file" "$SHARED_CONTRACT"; then
  echo "expected go/no-go evidence shared contract module to exercise audit-integrity gate arguments" >&2
  exit 1
fi

if ! grep -q -- "--slo-policy-report-file" "$SHARED_CONTRACT"; then
  echo "expected go/no-go evidence shared contract module to exercise SLO policy gate arguments" >&2
  exit 1
fi

if ! grep -q -- "--incident-readiness-report-file" "$SHARED_CONTRACT"; then
  echo "expected go/no-go evidence shared contract module to exercise incident-readiness gate arguments" >&2
  exit 1
fi

set +e
ci_smoke_overflow_output="$(bash "$FAST_SCRIPT" --max-seconds 121 2>&1)"
ci_smoke_overflow_code=$?
set -e
if [ "$ci_smoke_overflow_code" -eq 0 ]; then
  echo "expected go/no-go evidence contract lane to fail when ci smoke max-seconds exceeds boundary" >&2
  exit 1
fi
if ! printf '%s\n' "$ci_smoke_overflow_output" | grep -Fq "incident_gonogo_ci_smoke_seconds_exceeded"; then
  echo "expected ci smoke boundary overflow to emit deterministic fail-closed reason code marker" >&2
  exit 1
fi

if ! grep -Fq "run_gonogo_evidence_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute fast-lane contract checks first" >&2
  exit 1
fi

if ! grep -q "final_decision=NO-GO" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to validate NO-GO decision path" >&2
  exit 1
fi

set +e
missing_opt_in_output="$(bash "$DEEP_SCRIPT" 2>&1)"
missing_opt_in_code=$?
set -e
if [ "$missing_opt_in_code" -eq 0 ]; then
  echo "expected go/no-go evidence deep lane to require explicit local-heavy opt-in" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_opt_in_output" | grep -Fq "incident_gonogo_local_heavy_opt_in_missing"; then
  echo "expected missing local-heavy opt-in to emit deterministic fail-closed reason code marker" >&2
  exit 1
fi

set +e
local_heavy_budget_overflow_output="$(
  KAMN_GONOGO_GATE_LOCAL_OPT_IN=1 bash "$DEEP_SCRIPT" --max-seconds 901 2>&1
)"
local_heavy_budget_overflow_code=$?
set -e
if [ "$local_heavy_budget_overflow_code" -eq 0 ]; then
  echo "expected go/no-go evidence deep lane to fail when local-heavy max-seconds exceeds boundary" >&2
  exit 1
fi
if ! printf '%s\n' "$local_heavy_budget_overflow_output" | grep -Fq "incident_gonogo_local_heavy_seconds_exceeded"; then
  echo "expected local-heavy boundary overflow to emit deterministic fail-closed reason code marker" >&2
  exit 1
fi

deep_opt_in_output="$(KAMN_GONOGO_GATE_LOCAL_OPT_IN=1 bash "$DEEP_SCRIPT" --max-seconds 900)"
if ! printf '%s\n' "$deep_opt_in_output" | grep -q "go/no-go evidence deep lane tests passed."; then
  echo "expected go/no-go evidence deep lane run to pass when local-heavy opt-in is explicit" >&2
  exit 1
fi
if ! printf '%s\n' "$deep_opt_in_output" | grep -q "^local_heavy_lane_execution_mode=opt_in$"; then
  echo "expected go/no-go evidence deep lane to emit local-heavy execution mode marker when opt-in is present" >&2
  exit 1
fi

echo "go/no-go evidence contract lane script tests passed."
