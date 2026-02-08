import type { CanonicalMessageEnvelope } from "../../kamn-schema/src/index.ts";

export type TransportMode = "in-memory" | "live";

export interface AgentMetadata {
  agentType: string;
  modelFamily: string;
  capabilities: string[];
}

export interface AgentRecord {
  did: string;
  metadata: AgentMetadata;
}

export interface InboxMessage {
  id: string;
  from: string;
  to: string;
  body: string;
  envelope: CanonicalMessageEnvelope;
}

export interface TaskRecord {
  creator: string;
  taskType: string;
  description: string;
  assignee?: string;
  accepted: boolean;
}

export interface EscrowRecord {
  payer: string;
  payee: string;
  amount: number;
  released: boolean;
}

export interface ReputationRecord {
  id: string;
  score: number;
}

export interface SearchAgentsQuery {
  capability?: string;
  modelFamily?: string;
}

export interface SearchAgentResult {
  id: string;
  metadata: AgentMetadata;
}

export interface ResolveRecord {
  id: string;
  metadata: AgentMetadata;
  serviceEndpoint: string;
}

export interface OpenClawWorkflowRequest {
  requesterDid: string;
  openClawDid: string;
  prompt: string;
  compensation: number;
}

export interface OpenClawWorkflowResult {
  messageId: string;
  taskId: string;
  escrowId: string;
  workflowStatus: "settled";
}
