import {
  createCanonicalMessageEnvelope,
  SchemaError,
  validateCanonicalMessageEnvelope,
} from "../../kamn-schema/src/index.ts";
import type {
  AgentRecord,
  InboxMessage,
  EscrowRecord,
  ReputationRecord,
  ResolveRecord,
  SearchAgentResult,
  SearchAgentsQuery,
  TaskRecord,
} from "./types.ts";

export class SDKError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "SDKError";
  }
}

export class KAMNClient {
  private agentSeq = 1;
  private messageSeq = 1;
  private taskSeq = 1;
  private escrowSeq = 1;

  private readonly agents = new Map<string, AgentRecord>();
  private readonly inboxes = new Map<string, InboxMessage[]>();
  private readonly tasks = new Map<string, TaskRecord>();
  private readonly escrows = new Map<string, EscrowRecord>();
  private readonly balances = new Map<string, number>();
  private readonly reputation = new Map<string, ReputationRecord>();

  register(agentType: string, modelFamily: string, capabilities: string[]): string {
    if (!agentType.trim()) {
      throw new SDKError("agentType must not be empty");
    }
    if (!modelFamily.trim()) {
      throw new SDKError("modelFamily must not be empty");
    }
    if (capabilities.length === 0) {
      throw new SDKError("capabilities must not be empty");
    }

    const did = `kamn:did:agent:agent_${this.agentSeq}`;
    this.agentSeq += 1;

    const record: AgentRecord = {
      did,
      metadata: {
        agentType,
        modelFamily,
        capabilities: [...capabilities],
      },
    };
    this.agents.set(did, record);
    this.inboxes.set(did, []);
    this.balances.set(did, 100);
    this.reputation.set(did, { id: did, score: 500 });

    return did;
  }

  resolve(did: string): ResolveRecord {
    const agent = this.agents.get(did);
    if (!agent) {
      throw new SDKError(`unknown did: ${did}`);
    }

    return {
      id: agent.did,
      metadata: {
        agentType: agent.metadata.agentType,
        modelFamily: agent.metadata.modelFamily,
        capabilities: [...agent.metadata.capabilities],
      },
      serviceEndpoint: `kamn://messaging/${agent.did}`,
    };
  }

  send(fromDid: string, toDid: string, body: string): string {
    this.ensureKnownAgent(fromDid);
    this.ensureKnownAgent(toDid);
    if (!body.trim()) {
      throw new SDKError("message body must not be empty");
    }

    const messageId = `msg_${this.messageSeq}`;
    const messageNonce = this.messageSeq;
    this.messageSeq += 1;

    try {
      const envelope = createCanonicalMessageEnvelope({
        id: messageId,
        from: fromDid,
        to: [toDid],
        nonce: messageNonce,
        messageType: "Request",
        body: { text: body },
        recipientKeys: [`${toDid}#enc-1`],
      });
      validateCanonicalMessageEnvelope(envelope);

      const inbox = this.inboxes.get(toDid);
      if (!inbox) {
        throw new SDKError(`missing inbox for did: ${toDid}`);
      }
      inbox.push({
        id: messageId,
        from: fromDid,
        to: toDid,
        body,
        envelope,
      });
    } catch (error) {
      if (error instanceof SchemaError) {
        throw new SDKError(`schema validation failed: ${error.message}`);
      }
      throw error;
    }

    return messageId;
  }

  receive(did: string): InboxMessage[] {
    this.ensureKnownAgent(did);
    const inbox = this.inboxes.get(did);
    if (!inbox) {
      throw new SDKError(`missing inbox for did: ${did}`);
    }

    const drained = [...inbox];
    inbox.length = 0;
    return drained;
  }

  createTask(creatorDid: string, taskType: string, description: string): string {
    this.ensureKnownAgent(creatorDid);
    if (!taskType.trim()) {
      throw new SDKError("taskType must not be empty");
    }
    if (!description.trim()) {
      throw new SDKError("description must not be empty");
    }

    const taskId = `task_${this.taskSeq}`;
    this.taskSeq += 1;

    this.tasks.set(taskId, {
      creator: creatorDid,
      taskType,
      description,
      accepted: false,
    });

    return taskId;
  }

  acceptTask(taskId: string, assigneeDid: string): void {
    this.ensureKnownAgent(assigneeDid);
    const task = this.tasks.get(taskId);
    if (!task) {
      throw new SDKError(`unknown task: ${taskId}`);
    }
    if (task.accepted) {
      throw new SDKError("task already accepted");
    }

    task.assignee = assigneeDid;
    task.accepted = true;
  }

  createEscrow(payerDid: string, payeeDid: string, amount: number): string {
    this.ensureKnownAgent(payerDid);
    this.ensureKnownAgent(payeeDid);
    if (!Number.isInteger(amount) || amount <= 0) {
      throw new SDKError("escrow amount must be a positive integer");
    }

    const payerBalance = this.balance(payerDid);
    if (payerBalance < amount) {
      throw new SDKError("insufficient funds");
    }

    const escrowId = `escrow_${this.escrowSeq}`;
    this.escrowSeq += 1;

    this.balances.set(payerDid, payerBalance - amount);
    this.escrows.set(escrowId, {
      payer: payerDid,
      payee: payeeDid,
      amount,
      released: false,
    });

    return escrowId;
  }

  releaseEscrow(escrowId: string): void {
    const escrow = this.escrows.get(escrowId);
    if (!escrow) {
      throw new SDKError(`unknown escrow: ${escrowId}`);
    }
    if (escrow.released) {
      throw new SDKError("escrow already released");
    }

    const payeeBalance = this.balance(escrow.payee);
    this.balances.set(escrow.payee, payeeBalance + escrow.amount);
    escrow.released = true;
  }

  balance(did: string): number {
    this.ensureKnownAgent(did);
    const balance = this.balances.get(did);
    if (balance === undefined) {
      throw new SDKError(`missing balance for did: ${did}`);
    }
    return balance;
  }

  searchAgents(query: SearchAgentsQuery = {}): SearchAgentResult[] {
    const results: SearchAgentResult[] = [];
    for (const agent of this.agents.values()) {
      if (
        query.modelFamily &&
        agent.metadata.modelFamily !== query.modelFamily
      ) {
        continue;
      }
      if (
        query.capability &&
        !agent.metadata.capabilities.includes(query.capability)
      ) {
        continue;
      }
      results.push({
        id: agent.did,
        metadata: {
          agentType: agent.metadata.agentType,
          modelFamily: agent.metadata.modelFamily,
          capabilities: [...agent.metadata.capabilities],
        },
      });
    }

    results.sort((left, right) => left.id.localeCompare(right.id));
    return results;
  }

  getReputation(did: string): ReputationRecord {
    this.ensureKnownAgent(did);
    const reputation = this.reputation.get(did);
    if (!reputation) {
      throw new SDKError(`missing reputation for did: ${did}`);
    }

    return { ...reputation };
  }

  private ensureKnownAgent(did: string): void {
    if (!this.agents.has(did)) {
      throw new SDKError(`unknown did: ${did}`);
    }
  }
}
