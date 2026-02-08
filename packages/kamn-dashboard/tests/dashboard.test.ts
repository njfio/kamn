import test from "node:test";
import assert from "node:assert/strict";

import {
  buildDashboardShell,
  buildDashboardShellFromBackend,
  fetchDashboardSnapshotFromBackend,
  mapSnapshotToDashboardModel,
  mapSeverityLevel,
  renderDashboardHtml,
  renderDashboardState,
} from "../src/index.ts";

const ACTIVE_SESSION = {
  accessToken: "token-ops-1",
  role: "operator",
  expiresAtUnix: 1_700_003_000,
} as const;

test("unit maps severity to critical when error rate exceeds critical threshold", () => {
  const severity = mapSeverityLevel({
    value: 0.09,
    warningThreshold: 0.03,
    criticalThreshold: 0.06,
    higherIsWorse: true,
  });

  assert.equal(severity, "critical");
});

test("functional marks stale banner when snapshot age exceeds threshold", () => {
  const model = mapSnapshotToDashboardModel(
    {
      generated_at_unix: 1_700_000_000,
      domains: [
        {
          domain: "messaging",
          latency_p99_ms: 580,
          error_rate: 0.01,
          availability: 0.999,
        },
      ],
    },
    1_700_000_900,
    300,
  );

  assert.equal(model.staleBanner, true);
  assert.equal(model.domains[0]?.isStale, true);
});

test("integration renders deterministic dashboard shell sections", () => {
  const html = renderDashboardHtml({
    generatedAtUnix: 1_700_000_000,
    staleBanner: false,
    summaryCards: [
      { id: "critical-alerts", label: "Critical Alerts", value: "2", severity: "critical" },
      { id: "warning-alerts", label: "Warnings", value: "3", severity: "warning" },
    ],
    domains: [
      {
        domain: "messaging",
        latencyP99Ms: 580,
        errorRate: 0.01,
        availability: 0.999,
        severity: "warning",
        isStale: false,
      },
    ],
  });

  assert.match(html, /Critical Alerts/);
  assert.match(html, /domain-table/);
});

test("integration builds dashboard shell from deterministic mock adapter data", () => {
  const html = buildDashboardShell(1_700_001_200);
  assert.match(html, /KAMN Operator Dashboard MVP/);
  assert.match(html, /summary-grid/);
  assert.match(html, /reputation/);
});

test("integration returns empty state when shell receives null snapshot", () => {
  const html = buildDashboardShell(1_700_001_200, null);
  assert.match(html, /dashboard-empty/);
});

test("functional returns empty state when snapshot has no domains", () => {
  const html = buildDashboardShell(1_700_001_200, {
    generated_at_unix: 1_700_001_000,
    domains: [],
  });
  assert.match(html, /dashboard-empty/);
});

test("functional renders deterministic loading state", () => {
  const html = renderDashboardState({ state: "loading" });
  assert.match(html, /dashboard-loading/);
  assert.match(html, /Loading dashboard/);
});

test("integration renders explicit error state shell", () => {
  const html = renderDashboardState({
    state: "error",
    message: "mock adapter unavailable",
  });
  assert.match(html, /dashboard-error/);
  assert.match(html, /mock adapter unavailable/);
});

test("functional renders empty state when no rows are available", () => {
  const html = renderDashboardState({ state: "empty" });
  assert.match(html, /dashboard-empty/);
  assert.match(html, /No dashboard data available/);
});

test("regression renders critical badge and stale banner together", () => {
  // Regression: #591
  const html = renderDashboardHtml({
    generatedAtUnix: 1_700_000_000,
    staleBanner: true,
    summaryCards: [
      { id: "critical-alerts", label: "Critical Alerts", value: "4", severity: "critical" },
    ],
    domains: [
      {
        domain: "escrow",
        latencyP99Ms: 750,
        errorRate: 0.08,
        availability: 0.96,
        severity: "critical",
        isStale: true,
      },
    ],
  });

  assert.match(html, /stale-data-banner/);
  assert.match(html, /severity-critical/);
});

test("unit fetches dashboard snapshot from live backend client", async () => {
  const snapshot = await fetchDashboardSnapshotFromBackend({
    baseUrl: "https://dashboard.internal",
    session: ACTIVE_SESSION,
    sessionNowUnix: 1_700_002_200,
    fetchImpl: async (url, init) => {
      assert.equal(url, "https://dashboard.internal/api/dashboard/snapshot");
      assert.equal(init?.headers?.Authorization, "Bearer token-ops-1");
      assert.equal(init?.headers?.["X-KAMN-Role"], "operator");
      return {
        ok: true,
        status: 200,
        json: async () => ({
          generated_at_unix: 1_700_002_000,
          domains: [
            {
              domain: "messaging",
              latency_p99_ms: 470,
              error_rate: 0.012,
              availability: 0.998,
            },
          ],
        }),
      };
    },
  });

  assert.equal(snapshot.generated_at_unix, 1_700_002_000);
  assert.equal(snapshot.domains[0]?.domain, "messaging");
});

test("functional builds dashboard shell from live backend snapshot", async () => {
  const html = await buildDashboardShellFromBackend(1_700_002_200, {
    baseUrl: "https://dashboard.internal",
    session: ACTIVE_SESSION,
    sessionNowUnix: 1_700_002_200,
    fetchImpl: async () => ({
      ok: true,
      status: 200,
      json: async () => ({
        generated_at_unix: 1_700_002_000,
        domains: [
          {
            domain: "messaging",
            latency_p99_ms: 470,
            error_rate: 0.012,
            availability: 0.998,
          },
        ],
      }),
    }),
  });

  assert.match(html, /summary-grid/);
  assert.match(html, /messaging/);
});

test("regression renders error shell when live backend request fails", async () => {
  // Regression: #639
  const html = await buildDashboardShellFromBackend(1_700_002_200, {
    baseUrl: "https://dashboard.internal",
    session: ACTIVE_SESSION,
    sessionNowUnix: 1_700_002_200,
    fetchImpl: async () => ({
      ok: false,
      status: 503,
      json: async () => ({}),
    }),
  });

  assert.match(html, /dashboard-error/);
  assert.match(html, /503/);
});

test("regression rejects live backend access without operator session", async () => {
  // Regression: #640
  const html = await buildDashboardShellFromBackend(1_700_002_200, {
    baseUrl: "https://dashboard.internal",
    fetchImpl: async () => {
      throw new Error("fetch should not be called without session");
    },
  });

  assert.match(html, /dashboard-error/);
  assert.match(html, /operator session is required/i);
});

test("regression rejects expired or unauthorized session role", async () => {
  // Regression: #640
  const expiredHtml = await buildDashboardShellFromBackend(1_700_002_200, {
    baseUrl: "https://dashboard.internal",
    session: {
      accessToken: "token-expired",
      role: "operator",
      expiresAtUnix: 1_700_002_100,
    },
    sessionNowUnix: 1_700_002_200,
    fetchImpl: async () => {
      throw new Error("fetch should not be called with expired session");
    },
  });
  assert.match(expiredHtml, /session expired/i);

  const unauthorizedRoleHtml = await buildDashboardShellFromBackend(1_700_002_200, {
    baseUrl: "https://dashboard.internal",
    session: {
      accessToken: "token-viewer",
      role: "viewer",
      expiresAtUnix: 1_700_003_000,
    },
    sessionNowUnix: 1_700_002_200,
    allowedRoles: ["operator", "admin"],
    fetchImpl: async () => {
      throw new Error("fetch should not be called with unauthorized role");
    },
  });
  assert.match(unauthorizedRoleHtml, /session role not allowed/i);
});
