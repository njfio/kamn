import type { DashboardSnapshot } from "./types.ts";

const SNAPSHOT: DashboardSnapshot = {
  generated_at_unix: 1_700_000_500,
  domains: [
    {
      domain: "reputation",
      latency_p99_ms: 420,
      error_rate: 0.012,
      availability: 0.9993,
    },
    {
      domain: "escrow",
      latency_p99_ms: 760,
      error_rate: 0.071,
      availability: 0.971,
    },
    {
      domain: "messaging",
      latency_p99_ms: 560,
      error_rate: 0.027,
      availability: 0.994,
    },
  ],
};

export function fetchMockDashboardSnapshot(): DashboardSnapshot {
  return {
    generated_at_unix: SNAPSHOT.generated_at_unix,
    domains: SNAPSHOT.domains.map((domain) => ({ ...domain })),
  };
}
