#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

bash scripts/dashboard/test_run_backend_session_auth_freshness_lane.sh
bash scripts/dashboard/test_check_backend_session_auth_freshness_policy.sh
bash scripts/dashboard/test_run_backend_session_auth_freshness_contract_lane.sh
bash scripts/frontend/test_run_dashboard_shell_determinism_matrix_lane.sh
bash scripts/frontend/test_check_dashboard_shell_determinism_matrix_policy.sh
bash scripts/frontend/test_run_dashboard_shell_determinism_matrix_contract_lane.sh

if ! grep -Fq "## Frontend Contract Mapping" docs/foundation/operator-dashboard-backend-apis.md; then
  echo "expected frontend contract mapping section in operator-dashboard-backend-apis.md" >&2
  exit 1
fi

if ! grep -Fq "operator session token is required" docs/foundation/operator-dashboard-backend-apis.md; then
  echo "expected operator session requirement in operator-dashboard-backend-apis.md" >&2
  exit 1
fi

if ! grep -Fq "## Backend Session/Auth Freshness Contract" docs/foundation/operator-dashboard-backend-apis.md; then
  echo "expected backend session/auth freshness contract section in operator-dashboard-backend-apis.md" >&2
  exit 1
fi

if ! grep -Fq "Regression: #941" docs/foundation/operator-dashboard-backend-apis.md; then
  echo "expected regression marker for backend session/auth freshness contract in operator-dashboard-backend-apis.md" >&2
  exit 1
fi

if ! grep -Fq "fetchDashboardSnapshotFromBackend(...)" docs/foundation/operator-dashboard-ui-mvp.md; then
  echo "expected live backend snapshot mapping in operator-dashboard-ui-mvp.md" >&2
  exit 1
fi

if ! grep -Fq "## Frontend Shell Determinism Matrix Contract" docs/foundation/operator-dashboard-ui-mvp.md; then
  echo "expected frontend shell determinism matrix contract section in operator-dashboard-ui-mvp.md" >&2
  exit 1
fi

if ! grep -Fq "Regression: #943" docs/foundation/operator-dashboard-ui-mvp.md; then
  echo "expected regression marker for dashboard shell determinism matrix in operator-dashboard-ui-mvp.md" >&2
  exit 1
fi

if ! grep -Fq "Regression: #640" docs/foundation/operator-dashboard-ui-mvp.md; then
  echo "expected regression marker for dashboard session gate in operator-dashboard-ui-mvp.md" >&2
  exit 1
fi

echo "dashboard contract lane tests passed."
