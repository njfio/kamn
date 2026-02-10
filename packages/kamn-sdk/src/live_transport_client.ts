import { KAMNClient, SDKError, TransportModeMismatchError } from "./memory_client.ts";
import type {
  InboxMessage,
  ReputationRecord,
  ResolveRecord,
  SearchAgentResult,
  SearchAgentsQuery,
  TransportMode,
} from "./types.ts";

export type LiveTransportOperation =
  | "register"
  | "resolve"
  | "send"
  | "receive"
  | "createTask"
  | "acceptTask"
  | "createEscrow"
  | "releaseEscrow"
  | "balance"
  | "searchAgents"
  | "getReputation";

export interface LiveTransportBackendRequest {
  endpoint: string;
  operation: LiveTransportOperation;
  payload: Record<string, unknown>;
}

export type LiveTransportBackendResponse =
  | { status: "ok"; value: unknown }
  | { status: "error"; reason: string };

export interface LiveTransportBackendAdapter {
  invoke(request: LiveTransportBackendRequest): LiveTransportBackendResponse;
}

export class LiveTransportBackendAdapterError extends SDKError {
  readonly operation: LiveTransportOperation;

  constructor(operation: LiveTransportOperation, reason: string) {
    super(`backend adapter operation ${operation} failed: ${reason}`);
    this.name = "LiveTransportBackendAdapterError";
    this.operation = operation;
  }
}

export class LiveTransportConfig {
  readonly endpoint: string;

  constructor(endpoint: string) {
    const normalized = endpoint.trim().toLowerCase();
    if (!normalized.startsWith("https://") && !normalized.startsWith("wss://")) {
      throw new SDKError("transport endpoint must start with https:// or wss://");
    }
    if (normalized.length <= "https://a".length) {
      throw new SDKError("transport endpoint must include host information");
    }
    this.endpoint = endpoint;
  }
}

export class LiveTransportKAMNClient {
  private static readonly endpointRegistry = new Map<string, KAMNClient>();
  private static readonly backendAdapterRegistry = new Map<
    string,
    LiveTransportBackendAdapter
  >();

  readonly config: LiveTransportConfig;
  private readonly delegate: KAMNClient;
  private readonly backendAdapter?: LiveTransportBackendAdapter;

  static registerBackendAdapter(endpoint: string, adapter: LiveTransportBackendAdapter): void {
    const config = new LiveTransportConfig(endpoint);
    LiveTransportKAMNClient.backendAdapterRegistry.set(config.endpoint, adapter);
  }

  static clearBackendAdapters(): void {
    LiveTransportKAMNClient.backendAdapterRegistry.clear();
  }

  constructor(endpoint: string) {
    this.config = new LiveTransportConfig(endpoint);
    this.delegate =
      LiveTransportKAMNClient.endpointRegistry.get(this.config.endpoint) ??
      new KAMNClient();

    if (!LiveTransportKAMNClient.endpointRegistry.has(this.config.endpoint)) {
      LiveTransportKAMNClient.endpointRegistry.set(this.config.endpoint, this.delegate);
    }
    this.backendAdapter = LiveTransportKAMNClient.backendAdapterRegistry.get(this.config.endpoint);
  }

  transportMode(): TransportMode {
    return "live";
  }

  assertTransportMode(expected: TransportMode): void {
    const found = this.transportMode();
    if (found !== expected) {
      throw new TransportModeMismatchError(expected, found);
    }
  }

  register(agentType: string, modelFamily: string, capabilities: string[]): string {
    return this.invokeWithAdapter(
      "register",
      { agentType, modelFamily, capabilities: [...capabilities] },
      (value) => this.requireStringValue("register", value),
      () => this.delegate.register(agentType, modelFamily, capabilities),
    );
  }

  resolve(did: string): ResolveRecord {
    return this.invokeWithAdapter(
      "resolve",
      { did },
      (value) => this.requireResolveRecordValue("resolve", value),
      () => this.delegate.resolve(did),
    );
  }

  send(fromDid: string, toDid: string, body: string): string {
    return this.invokeWithAdapter(
      "send",
      { fromDid, toDid, body },
      (value) => this.requireStringValue("send", value),
      () => this.delegate.send(fromDid, toDid, body),
    );
  }

  receive(did: string): InboxMessage[] {
    return this.invokeWithAdapter(
      "receive",
      { did },
      (value) => this.requireInboxMessagesValue("receive", value),
      () => this.delegate.receive(did),
    );
  }

  createTask(creatorDid: string, taskType: string, description: string): string {
    return this.invokeWithAdapter(
      "createTask",
      { creatorDid, taskType, description },
      (value) => this.requireStringValue("createTask", value),
      () => this.delegate.createTask(creatorDid, taskType, description),
    );
  }

  acceptTask(taskId: string, assigneeDid: string): void {
    this.invokeWithAdapter(
      "acceptTask",
      { taskId, assigneeDid },
      () => undefined,
      () => this.delegate.acceptTask(taskId, assigneeDid),
    );
  }

  createEscrow(payerDid: string, payeeDid: string, amount: number): string {
    return this.invokeWithAdapter(
      "createEscrow",
      { payerDid, payeeDid, amount },
      (value) => this.requireStringValue("createEscrow", value),
      () => this.delegate.createEscrow(payerDid, payeeDid, amount),
    );
  }

  releaseEscrow(escrowId: string): void {
    this.invokeWithAdapter(
      "releaseEscrow",
      { escrowId },
      () => undefined,
      () => this.delegate.releaseEscrow(escrowId),
    );
  }

  balance(did: string): number {
    return this.invokeWithAdapter(
      "balance",
      { did },
      (value) => this.requireNumberValue("balance", value),
      () => this.delegate.balance(did),
    );
  }

  searchAgents(query: SearchAgentsQuery = {}): SearchAgentResult[] {
    return this.invokeWithAdapter(
      "searchAgents",
      { ...query },
      (value) => this.requireSearchAgentsValue("searchAgents", value),
      () => this.delegate.searchAgents(query),
    );
  }

  getReputation(did: string): ReputationRecord {
    return this.invokeWithAdapter(
      "getReputation",
      { did },
      (value) => this.requireReputationValue("getReputation", value),
      () => this.delegate.getReputation(did),
    );
  }

  private invokeWithAdapter<T>(
    operation: LiveTransportOperation,
    payload: Record<string, unknown>,
    normalize: (value: unknown) => T,
    fallback: () => T,
  ): T {
    if (!this.backendAdapter) {
      return fallback();
    }
    const response = this.backendAdapter.invoke({
      endpoint: this.config.endpoint,
      operation,
      payload,
    });

    if (response.status === "error") {
      const reason =
        typeof response.reason === "string" && response.reason.trim()
          ? response.reason.trim()
          : "backend adapter returned unknown error";
      throw new LiveTransportBackendAdapterError(operation, reason);
    }

    if (response.status !== "ok") {
      this.throwInvalidAdapterResponse(operation, "expected status ok|error");
    }
    return normalize(response.value);
  }

  private throwInvalidAdapterResponse(operation: LiveTransportOperation, reason: string): never {
    throw new SDKError(`backend adapter invalid response for operation ${operation}: ${reason}`);
  }

  private requireStringValue(operation: LiveTransportOperation, value: unknown): string {
    if (typeof value !== "string" || value.trim() === "") {
      this.throwInvalidAdapterResponse(operation, "expected string value");
    }
    return value;
  }

  private requireNumberValue(operation: LiveTransportOperation, value: unknown): number {
    if (typeof value !== "number" || !Number.isFinite(value)) {
      this.throwInvalidAdapterResponse(operation, "expected numeric value");
    }
    return value;
  }

  private requireResolveRecordValue(
    operation: LiveTransportOperation,
    value: unknown,
  ): ResolveRecord {
    if (
      typeof value !== "object" ||
      value === null ||
      typeof (value as { id?: unknown }).id !== "string" ||
      typeof (value as { serviceEndpoint?: unknown }).serviceEndpoint !== "string"
    ) {
      this.throwInvalidAdapterResponse(operation, "expected resolve record");
    }
    return value as ResolveRecord;
  }

  private requireSearchAgentsValue(
    operation: LiveTransportOperation,
    value: unknown,
  ): SearchAgentResult[] {
    if (!Array.isArray(value)) {
      this.throwInvalidAdapterResponse(operation, "expected search agent result array");
    }
    return value as SearchAgentResult[];
  }

  private requireReputationValue(
    operation: LiveTransportOperation,
    value: unknown,
  ): ReputationRecord {
    if (
      typeof value !== "object" ||
      value === null ||
      typeof (value as { id?: unknown }).id !== "string" ||
      typeof (value as { score?: unknown }).score !== "number"
    ) {
      this.throwInvalidAdapterResponse(operation, "expected reputation record");
    }
    return value as ReputationRecord;
  }

  private requireInboxMessagesValue(
    operation: LiveTransportOperation,
    value: unknown,
  ): InboxMessage[] {
    if (!Array.isArray(value)) {
      this.throwInvalidAdapterResponse(operation, "expected inbox message array");
    }
    const normalized: InboxMessage[] = [];
    for (const entry of value) {
      if (typeof entry !== "object" || entry === null) {
        this.throwInvalidAdapterResponse(operation, "expected inbox message object");
      }

      const candidate = entry as {
        id?: unknown;
        from?: unknown;
        to?: unknown;
        body?: unknown;
        envelope?: unknown;
      };
      if (
        typeof candidate.id !== "string" ||
        typeof candidate.from !== "string" ||
        typeof candidate.to !== "string" ||
        typeof candidate.body !== "string"
      ) {
        this.throwInvalidAdapterResponse(operation, "expected inbox message string fields");
      }

      normalized.push({
        id: candidate.id,
        from: candidate.from,
        to: candidate.to,
        body: candidate.body,
        envelope: (candidate.envelope ?? { id: candidate.id }) as InboxMessage["envelope"],
      });
    }
    return normalized;
  }
}
