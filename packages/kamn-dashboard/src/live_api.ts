import type { DashboardDomainSample, DashboardSnapshot } from "./types.ts";

type JsonResponse = {
  ok: boolean;
  status: number;
  json(): Promise<unknown>;
};

export type DashboardFetchLike = (
  url: string,
  init?: {
    headers?: Record<string, string>;
  },
) => Promise<JsonResponse>;

export type DashboardBackendClientOptions = {
  baseUrl: string;
  snapshotPath?: string;
  headers?: Record<string, string>;
  fetchImpl?: DashboardFetchLike;
};

export class DashboardBackendError extends Error {
  readonly code: "network" | "http" | "invalid-response";
  readonly status?: number;

  constructor(code: "network" | "http" | "invalid-response", message: string, status?: number) {
    super(message);
    this.code = code;
    this.status = status;
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function parseDomain(value: unknown): DashboardDomainSample {
  if (!isRecord(value)) {
    throw new DashboardBackendError("invalid-response", "dashboard domain row is not an object");
  }

  const domain = value.domain;
  const latency = value.latency_p99_ms;
  const errorRate = value.error_rate;
  const availability = value.availability;

  if (typeof domain !== "string" || domain.length === 0) {
    throw new DashboardBackendError("invalid-response", "dashboard domain name is invalid");
  }
  if (typeof latency !== "number" || !Number.isFinite(latency)) {
    throw new DashboardBackendError("invalid-response", "dashboard latency value is invalid");
  }
  if (typeof errorRate !== "number" || !Number.isFinite(errorRate)) {
    throw new DashboardBackendError("invalid-response", "dashboard error rate value is invalid");
  }
  if (typeof availability !== "number" || !Number.isFinite(availability)) {
    throw new DashboardBackendError("invalid-response", "dashboard availability value is invalid");
  }

  return {
    domain,
    latency_p99_ms: latency,
    error_rate: errorRate,
    availability,
  };
}

function parseSnapshot(value: unknown): DashboardSnapshot {
  if (!isRecord(value)) {
    throw new DashboardBackendError("invalid-response", "dashboard snapshot payload is not an object");
  }

  const generatedAt = value.generated_at_unix;
  const domains = value.domains;
  if (!Number.isFinite(generatedAt)) {
    throw new DashboardBackendError("invalid-response", "dashboard snapshot timestamp is invalid");
  }
  if (!Array.isArray(domains)) {
    throw new DashboardBackendError("invalid-response", "dashboard snapshot domains field is invalid");
  }

  return {
    generated_at_unix: generatedAt,
    domains: domains.map(parseDomain),
  };
}

function buildSnapshotUrl(baseUrl: string, snapshotPath = "/api/dashboard/snapshot"): string {
  try {
    return new URL(snapshotPath, baseUrl).toString();
  } catch (_error) {
    throw new DashboardBackendError("invalid-response", "dashboard backend base URL is invalid");
  }
}

export async function fetchDashboardSnapshotFromBackend(
  options: DashboardBackendClientOptions,
): Promise<DashboardSnapshot> {
  const url = buildSnapshotUrl(options.baseUrl, options.snapshotPath);

  const defaultFetch: DashboardFetchLike | undefined = globalThis.fetch
    ? async (targetUrl, init) => {
        const response = await globalThis.fetch(targetUrl, init);
        return response as JsonResponse;
      }
    : undefined;
  const fetchImpl = options.fetchImpl ?? defaultFetch;
  if (!fetchImpl) {
    throw new DashboardBackendError(
      "network",
      "dashboard backend fetch implementation is not available",
    );
  }

  let response: JsonResponse;
  try {
    response = await fetchImpl(url, {
      headers: {
        Accept: "application/json",
        ...(options.headers ?? {}),
      },
    });
  } catch (error) {
    const message = error instanceof Error ? error.message : "unknown network error";
    throw new DashboardBackendError("network", `dashboard backend request failed: ${message}`);
  }

  if (!response.ok) {
    throw new DashboardBackendError(
      "http",
      `dashboard backend responded with status ${response.status}`,
      response.status,
    );
  }

  let payload: unknown;
  try {
    payload = await response.json();
  } catch (_error) {
    throw new DashboardBackendError("invalid-response", "dashboard backend returned invalid JSON");
  }

  return parseSnapshot(payload);
}
