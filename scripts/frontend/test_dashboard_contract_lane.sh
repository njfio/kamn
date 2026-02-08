#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

bash scripts/frontend/test_dashboard_package.sh

if ! grep -Fq "## Frontend Contract Mapping" docs/foundation/operator-dashboard-backend-apis.md; then
  echo "expected frontend contract mapping section in operator-dashboard-backend-apis.md" >&2
  exit 1
fi

if ! grep -Fq "operator session token is required" docs/foundation/operator-dashboard-backend-apis.md; then
  echo "expected operator session requirement in operator-dashboard-backend-apis.md" >&2
  exit 1
fi

if ! grep -Fq "fetchDashboardSnapshotFromBackend(...)" docs/foundation/operator-dashboard-ui-mvp.md; then
  echo "expected live backend snapshot mapping in operator-dashboard-ui-mvp.md" >&2
  exit 1
fi

if ! grep -Fq "Regression: #640" docs/foundation/operator-dashboard-ui-mvp.md; then
  echo "expected regression marker for dashboard session gate in operator-dashboard-ui-mvp.md" >&2
  exit 1
fi

echo "dashboard contract lane tests passed."
