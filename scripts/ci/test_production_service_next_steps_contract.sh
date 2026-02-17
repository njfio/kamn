#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PLAN_DOC="$ROOT_DIR/docs/plans/2026-02-14-production-service-next-steps.md"
TMP_TAMPERED="$(mktemp)"
trap 'rm -f "$TMP_TAMPERED"' EXIT

if [ ! -f "$PLAN_DOC" ]; then
  echo "expected production-service next-steps plan document to exist" >&2
  exit 1
fi

python3 - "$PLAN_DOC" <<'PY'
from __future__ import annotations

import pathlib
import sys

doc_path = pathlib.Path(sys.argv[1])
text = doc_path.read_text(encoding="utf-8")

required_markers = (
    "Status Truth Snapshot",
    "| 1. HTTP ingress runtime | Delivered (`axum` server + auth/ws integration) |",
    "| 2. Persistent storage | Delivered (sqlite backend adapters + migration parity) |",
    "| 3. Real P2P transport | Delivered (live libp2p provider + lifecycle/fault hardening) |",
    "| 4. Transport-fed consensus pipeline | Delivered (transport-fed convergence + go/no-go evidence gate) |",
    "#3228 -> #3229 -> #3313 -> (#3356, #3314, #3315, #3319, #3470)",
    "#3228 -> #3413 -> #3414 -> (#3443, #3444, #3446, #3447, #3448)",
    "#3333 -> #3471 -> #3472 -> #3473 -> #3474 -> #3490",
    "#3333 -> #3424 -> #3425 -> #3426",
    "Active Open Chains",
    "R26.5 Observability and transport resilience hardening",
    "#3333 -> #3772",
    "#3772 -> #3773",
    "#3772 -> #3774",
    "`#3773 -> #3775 -> (#3782, #3783)`.",
    "`#3774 -> #3780 -> (#3794, #3795)`.",
    "R27.16 Retry/TLS CI Smoke Closure",
    "#4100 -> #4104 -> (#4111, #4112)",
    "retry_tls_smoke_contract_status=verified",
    "retry_tls_live_https_taxonomy_version=kamn.ci.kamn-core-live-https-dependency-posture-reason-taxonomy.v1",
    "retry_tls_submit_finality_taxonomy_version=kamn.kolme.local-runtime-commit-submit-finality-reason-taxonomy.v1",
    "retry/tls local-heavy run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode.",
    "### R27.24 Admission-Backpressure CI Smoke Governance Closure",
    "Active chain: `#4218 -> #4220 -> #4224 -> (#4231, #4232)`.",
    "admission_backpressure_ci_smoke_convergence_status=verified",
    "admission_backpressure_ci_smoke_reason_taxonomy_version=kamn.ci.admission-backpressure-ci-smoke-convergence-reason-taxonomy.v1",
    "admission_backpressure_ci_smoke_max_seconds=120",
    "admission_backpressure_local_heavy_max_seconds=900",
    "test_check_admission_backpressure_ci_smoke_convergence.sh",
    "### R27.25 Replay-Integrity CI Smoke Governance Closure",
    "Active chain: `#4233 -> #4235 -> #4239 -> (#4246, #4247)`.",
    "sqlite_crash_recovery_ci_smoke_convergence_status=verified",
    "sqlite_crash_recovery_ci_smoke_reason_taxonomy_version=kamn.ci.sqlite-crash-recovery-ci-smoke-convergence-reason-taxonomy.v1",
    "sqlite_crash_recovery_ci_smoke_max_seconds=120",
    "sqlite_crash_recovery_local_heavy_max_seconds=900",
    "test_check_sqlite_crash_recovery_ci_smoke_convergence.sh",
    "R27.29 Transport/Observability/TLS CI Smoke Convergence Closure",
    "Active chain: `#4293 -> #4295 -> #4299 -> (#4306, #4307)`.",
    "transport_observability_tls_ci_smoke_convergence_status=verified",
    "transport_observability_tls_reason_taxonomy_version=kamn.ci.transport-observability-tls-ci-smoke-convergence-reason-taxonomy.v1",
    "transport_observability_tls_ci_smoke_max_seconds=120",
    "transport_observability_tls_local_heavy_max_seconds=900",
    "test_check_transport_observability_tls_ci_smoke_convergence.sh",
    "### R27.30 Partition-Finality CI Smoke Governance Closure",
    "Active chain: `#4250 -> #4254 -> (#4261, #4262)`.",
    "partition_finality_ci_smoke_convergence_status=verified",
    "partition_finality_ci_smoke_reason_taxonomy_version=kamn.ci.partition-finality-ci-smoke-convergence-reason-taxonomy.v1",
    "partition_finality_ci_smoke_max_seconds=120",
    "partition_finality_local_heavy_max_seconds=900",
    "test_check_partition_finality_ci_smoke_convergence.sh",
    "### R27.27 Websocket Session CI Smoke Governance Closure",
    "Active chain: `#4265 -> #4269 -> (#4276, #4277)`.",
    "websocket_session_ci_smoke_convergence_status=verified",
    "websocket_session_ci_smoke_reason_taxonomy_version=kamn.ci.websocket-session-ci-smoke-convergence-reason-taxonomy.v1",
    "websocket_session_ci_smoke_max_seconds=120",
    "websocket_session_local_heavy_max_seconds=900",
    "test_check_websocket_session_ci_smoke_convergence.sh",
    "### R27.28 Drift/Failover CI Smoke Governance Closure",
    "Active chain: `#4278 -> #4280 -> #4284 -> (#4291, #4292)`.",
    "failover_drift_ci_smoke_convergence_status=verified",
    "failover_drift_ci_smoke_reason_taxonomy_version=kamn.ci.failover-drift-ci-smoke-convergence-reason-taxonomy.v1",
    "failover_drift_ci_smoke_max_seconds=120",
    "failover_drift_local_heavy_max_seconds=900",
    "test_check_failover_drift_ci_smoke_convergence.sh",
    "Historical Baseline (Superseded)",
)
for marker in required_markers:
    if marker not in text:
        raise SystemExit(f"marker_missing:{marker}")

for reason, snippet in (
    ("ingress_hand_rolled_tcp_listener", "Hand-rolled `TcpListener` + manual HTTP parser"),
    ("storage_file_only_no_sqlite", "File-based JSON dumps"),
    ("runtime_mode_full_missing", "A production node needs to be a daemon AND serve an API"),
):
    if snippet in text:
        raise SystemExit(f"stale_claim_detected:{reason}")
PY

cp "$PLAN_DOC" "$TMP_TAMPERED"
cat >>"$TMP_TAMPERED" <<'TXT'
Stale baseline regression fixture:
Hand-rolled `TcpListener` + manual HTTP parser
TXT

set +e
tampered_output="$(
  python3 - "$TMP_TAMPERED" 2>&1 <<'PY'
from __future__ import annotations

import pathlib
import sys

doc_path = pathlib.Path(sys.argv[1])
text = doc_path.read_text(encoding="utf-8")

for reason, snippet in (
    ("ingress_hand_rolled_tcp_listener", "Hand-rolled `TcpListener` + manual HTTP parser"),
    ("storage_file_only_no_sqlite", "File-based JSON dumps"),
    ("runtime_mode_full_missing", "A production node needs to be a daemon AND serve an API"),
):
    if snippet in text:
        raise SystemExit(f"stale_claim_detected:{reason}")
PY
)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered production-service next-steps doc to fail stale-claim contract" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'stale_claim_detected:ingress_hand_rolled_tcp_listener'; then
  echo "expected deterministic stale-claim marker for tampered production-service next-steps doc" >&2
  exit 1
fi

echo "production-service next-steps docs contract tests passed."
