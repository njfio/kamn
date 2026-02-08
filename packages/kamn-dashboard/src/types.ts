export type SeverityLevel = "healthy" | "warning" | "critical";

export type SeverityInput = {
  value: number;
  warningThreshold: number;
  criticalThreshold: number;
  higherIsWorse: boolean;
};

export type DashboardDomainSample = {
  domain: string;
  latency_p99_ms: number;
  error_rate: number;
  availability: number;
};

export type DashboardSnapshot = {
  generated_at_unix: number;
  domains: DashboardDomainSample[];
};

export type DashboardOperatorRole = "viewer" | "operator" | "admin";

export type DashboardOperatorSession = {
  accessToken: string;
  role: DashboardOperatorRole;
  expiresAtUnix: number;
};

export type DashboardSummaryCard = {
  id: string;
  label: string;
  value: string;
  severity: SeverityLevel;
};

export type DashboardDomainRow = {
  domain: string;
  latencyP99Ms: number;
  errorRate: number;
  availability: number;
  severity: SeverityLevel;
  isStale: boolean;
};

export type DashboardModel = {
  generatedAtUnix: number;
  staleBanner: boolean;
  summaryCards: DashboardSummaryCard[];
  domains: DashboardDomainRow[];
};

export type DashboardRenderState =
  | {
      state: "loading";
    }
  | {
      state: "error";
      message: string;
    }
  | {
      state: "empty";
    }
  | {
      state: "ready";
      model: DashboardModel;
    };
