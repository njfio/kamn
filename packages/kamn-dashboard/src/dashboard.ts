import { fetchMockDashboardSnapshot } from "./mock_api.ts";
import type {
  DashboardDomainSample,
  DashboardModel,
  DashboardRenderState,
  DashboardSummaryCard,
  DashboardSnapshot,
  SeverityInput,
  SeverityLevel,
} from "./types.ts";

const DEFAULT_STALE_AFTER_SECONDS = 300;

function severityRank(severity: SeverityLevel): number {
  switch (severity) {
    case "critical":
      return 2;
    case "warning":
      return 1;
    default:
      return 0;
  }
}

function maxSeverity(levels: SeverityLevel[]): SeverityLevel {
  let current: SeverityLevel = "healthy";
  for (const level of levels) {
    if (severityRank(level) > severityRank(current)) {
      current = level;
    }
  }
  return current;
}

export function mapSeverityLevel(input: SeverityInput): SeverityLevel {
  if (input.higherIsWorse) {
    if (input.value >= input.criticalThreshold) {
      return "critical";
    }
    if (input.value >= input.warningThreshold) {
      return "warning";
    }
    return "healthy";
  }

  if (input.value <= input.criticalThreshold) {
    return "critical";
  }
  if (input.value <= input.warningThreshold) {
    return "warning";
  }
  return "healthy";
}

function mapDomainSeverity(sample: DashboardDomainSample): SeverityLevel {
  const latency = mapSeverityLevel({
    value: sample.latency_p99_ms,
    warningThreshold: 500,
    criticalThreshold: 700,
    higherIsWorse: true,
  });
  const errorRate = mapSeverityLevel({
    value: sample.error_rate,
    warningThreshold: 0.03,
    criticalThreshold: 0.06,
    higherIsWorse: true,
  });
  const availability = mapSeverityLevel({
    value: sample.availability,
    warningThreshold: 0.995,
    criticalThreshold: 0.98,
    higherIsWorse: false,
  });

  return maxSeverity([latency, errorRate, availability]);
}

function buildSummaryCards(
  domains: DashboardModel["domains"],
  staleBanner: boolean,
): DashboardSummaryCard[] {
  const criticalCount = domains.filter((domain) => domain.severity === "critical").length;
  const warningCount = domains.filter((domain) => domain.severity === "warning").length;

  return [
    {
      id: "critical-alerts",
      label: "Critical Alerts",
      value: String(criticalCount),
      severity: criticalCount > 0 ? "critical" : "healthy",
    },
    {
      id: "warning-alerts",
      label: "Warnings",
      value: String(warningCount),
      severity: warningCount > 0 ? "warning" : "healthy",
    },
    {
      id: "snapshot-freshness",
      label: "Snapshot Freshness",
      value: staleBanner ? "Stale" : "Fresh",
      severity: staleBanner ? "critical" : "healthy",
    },
  ];
}

export function mapSnapshotToDashboardModel(
  snapshot: DashboardSnapshot,
  nowUnix: number,
  staleAfterSeconds = DEFAULT_STALE_AFTER_SECONDS,
): DashboardModel {
  const staleBanner = nowUnix - snapshot.generated_at_unix > staleAfterSeconds;
  const domains = snapshot.domains
    .map((domain) => ({
      domain: domain.domain,
      latencyP99Ms: domain.latency_p99_ms,
      errorRate: domain.error_rate,
      availability: domain.availability,
      severity: mapDomainSeverity(domain),
      isStale: staleBanner,
    }))
    .sort((left, right) => left.domain.localeCompare(right.domain));

  return {
    generatedAtUnix: snapshot.generated_at_unix,
    staleBanner,
    summaryCards: buildSummaryCards(domains, staleBanner),
    domains,
  };
}

function renderReadyBody(model: DashboardModel): string {
  const staleBanner = model.staleBanner
    ? '<div class="stale-data-banner severity-critical">Snapshot is stale</div>'
    : "";

  const cards = model.summaryCards
    .map(
      (card) =>
        `<article class="summary-card severity-${card.severity}" data-card-id="${card.id}"><h2>${card.label}</h2><p>${card.value}</p></article>`,
    )
    .join("");

  const rows = model.domains
    .map(
      (domain) =>
        `<tr class="domain-row severity-${domain.severity}"><td>${domain.domain}</td><td>${domain.latencyP99Ms}</td><td>${domain.errorRate.toFixed(3)}</td><td>${domain.availability.toFixed(3)}</td><td>${domain.isStale ? "stale" : "fresh"}</td></tr>`,
    )
    .join("");

  return `<main class="dashboard-shell">
      ${staleBanner}
      <section class="summary-grid">${cards}</section>
      <section class="domain-table-shell">
        <table class="domain-table">
          <thead>
            <tr><th>Domain</th><th>Latency P99 (ms)</th><th>Error Rate</th><th>Availability</th><th>Freshness</th></tr>
          </thead>
          <tbody>${rows}</tbody>
        </table>
      </section>
    </main>`;
}

function renderShell(body: string): string {
  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>KAMN Operator Dashboard MVP</title>
  </head>
  <body>
    ${body}
  </body>
</html>`;
}

export function renderDashboardState(state: DashboardRenderState): string {
  switch (state.state) {
    case "loading":
      return renderShell(
        '<main class="dashboard-shell"><section class="dashboard-loading" role="status" aria-live="polite">Loading dashboard...</section></main>',
      );
    case "error":
      return renderShell(
        `<main class="dashboard-shell"><section class="dashboard-error" role="alert">Dashboard error: ${state.message}</section></main>`,
      );
    case "empty":
      return renderShell(
        '<main class="dashboard-shell"><section class="dashboard-empty">No dashboard data available.</section></main>',
      );
    case "ready":
      return renderShell(renderReadyBody(state.model));
  }
}

export function renderDashboardHtml(model: DashboardModel): string {
  return renderDashboardState({
    state: "ready",
    model,
  });
}

export function buildDashboardShell(
  nowUnix: number,
  snapshot: DashboardSnapshot | null = undefined,
): string {
  if (snapshot === null) {
    return renderDashboardState({ state: "empty" });
  }

  const resolvedSnapshot = snapshot ?? fetchMockDashboardSnapshot();
  const model = mapSnapshotToDashboardModel(resolvedSnapshot, nowUnix);
  if (model.domains.length === 0) {
    return renderDashboardState({ state: "empty" });
  }
  return renderDashboardState({
    state: "ready",
    model,
  });
}
