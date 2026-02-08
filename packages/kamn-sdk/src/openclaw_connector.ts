import { KAMNClient, SDKError } from "./memory_client.ts";
import type {
  OpenClawWorkflowRequest,
  OpenClawWorkflowResult,
} from "./types.ts";

export class OpenClawConnector {
  private readonly client: KAMNClient;

  constructor(client: KAMNClient) {
    this.client = client;
  }

  registerOpenClawAgent(modelFamily: string): string {
    if (!modelFamily.trim()) {
      throw new SDKError("modelFamily must not be empty");
    }

    return this.client.register("assistant", modelFamily, [
      "text",
      "code",
      "openclaw",
      "payments",
    ]);
  }

  runReferenceWorkflow(
    request: OpenClawWorkflowRequest,
  ): OpenClawWorkflowResult {
    if (!request.prompt.trim()) {
      throw new SDKError("prompt must not be empty");
    }
    if (!Number.isInteger(request.compensation) || request.compensation <= 0) {
      throw new SDKError("compensation must be a positive integer");
    }

    const openClaw = this.client.resolve(request.openClawDid);
    if (!openClaw.metadata.capabilities.includes("openclaw")) {
      throw new SDKError("openClawDid must have openclaw capability");
    }

    const messageId = this.client.send(
      request.requesterDid,
      request.openClawDid,
      request.prompt,
    );

    const taskId = this.client.createTask(
      request.requesterDid,
      "openclaw.reference.workflow",
      `OpenClaw request: ${request.prompt}`,
    );
    this.client.acceptTask(taskId, request.openClawDid);

    const escrowId = this.client.createEscrow(
      request.requesterDid,
      request.openClawDid,
      request.compensation,
    );
    this.client.releaseEscrow(escrowId);

    return {
      messageId,
      taskId,
      escrowId,
      workflowStatus: "settled",
    };
  }
}
