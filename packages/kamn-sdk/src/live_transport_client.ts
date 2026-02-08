import { KAMNClient, SDKError, TransportModeMismatchError } from "./memory_client.ts";
import type {
  InboxMessage,
  ReputationRecord,
  ResolveRecord,
  SearchAgentResult,
  SearchAgentsQuery,
  TransportMode,
} from "./types.ts";

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

  readonly config: LiveTransportConfig;
  private readonly delegate: KAMNClient;

  constructor(endpoint: string) {
    this.config = new LiveTransportConfig(endpoint);
    this.delegate =
      LiveTransportKAMNClient.endpointRegistry.get(this.config.endpoint) ??
      new KAMNClient();

    if (!LiveTransportKAMNClient.endpointRegistry.has(this.config.endpoint)) {
      LiveTransportKAMNClient.endpointRegistry.set(this.config.endpoint, this.delegate);
    }
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
    return this.delegate.register(agentType, modelFamily, capabilities);
  }

  resolve(did: string): ResolveRecord {
    return this.delegate.resolve(did);
  }

  send(fromDid: string, toDid: string, body: string): string {
    return this.delegate.send(fromDid, toDid, body);
  }

  receive(did: string): InboxMessage[] {
    return this.delegate.receive(did);
  }

  createTask(creatorDid: string, taskType: string, description: string): string {
    return this.delegate.createTask(creatorDid, taskType, description);
  }

  acceptTask(taskId: string, assigneeDid: string): void {
    this.delegate.acceptTask(taskId, assigneeDid);
  }

  createEscrow(payerDid: string, payeeDid: string, amount: number): string {
    return this.delegate.createEscrow(payerDid, payeeDid, amount);
  }

  releaseEscrow(escrowId: string): void {
    this.delegate.releaseEscrow(escrowId);
  }

  balance(did: string): number {
    return this.delegate.balance(did);
  }

  searchAgents(query: SearchAgentsQuery = {}): SearchAgentResult[] {
    return this.delegate.searchAgents(query);
  }

  getReputation(did: string): ReputationRecord {
    return this.delegate.getReputation(did);
  }
}
