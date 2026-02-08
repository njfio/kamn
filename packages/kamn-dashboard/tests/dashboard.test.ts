import test from "node:test";
import assert from "node:assert/strict";

import {
  buildDashboardShell,
  mapSnapshotToDashboardModel,
  mapSeverityLevel,
  renderDashboardHtml,
} from "../src/index.ts";

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
