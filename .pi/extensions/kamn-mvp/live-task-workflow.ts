import { McpSession, readLiveMcpConfig, type McpSessionProvenance } from "./mcp-session.ts";
import {
	agentLabel, configRole, requiredString, requiredText, validImportedTaskId,
	buildActorEvidence, validateEscrowResult, validateProjection, validateTaskProjection, validateTaskResult, validateWaitOptions,
} from "./live-task-workflow-support.ts";
import { LiveSettlementAgreement } from "./live-settlement-agreement.ts";
import type { AgreementIdentity } from "./live-settlement-agreement.ts";
import { reconcileAmbiguousRelease, waitForEscrowProjection, waitForFinalProjection, waitForTaskState } from "./live-task-workflow-retry.ts";
import { callWithAntiSpamBackoff } from "./mcp-call-retry.ts";
export type AgentRole = "agent_a" | "agent_b" | "agent_c";
export type WorkflowResult = Record<string, unknown>;
type Environment = Record<string, string | undefined>;
type AgentState = { did?: string; session?: McpSession };
type WaitOptions = { timeoutMs: number; pollMs: number };

export class LiveTaskWorkflow {
	private readonly agents: Record<AgentRole, AgentState> = { agent_a: {}, agent_b: {}, agent_c: {} };
	private readonly observations: Partial<Record<AgentRole, WorkflowResult>> = {};
	private readonly projections: Partial<Record<AgentRole, WorkflowResult>> = {};
	private taskId?: string;
	private escrowId?: string;
	private agreement?: LiveSettlementAgreement;
	private agreementIdentity?: AgreementIdentity;
	private readonly env: Environment;
	private readonly cwd: string;
	constructor(env: Environment, cwd: string) {
		this.env = env;
		this.cwd = cwd;
	}
	async register(role: AgentRole, signal?: AbortSignal): Promise<WorkflowResult> {
		const result = await this.call(role, "register", {}, signal);
		const did = requiredString(result, "did", `${agentLabel(role)} registration`);
		this.assertDistinctDid(role, did);
		this.agents[role].did = did;
		return result;
	}
	async queryProfile(role: AgentRole, signal?: AbortSignal): Promise<WorkflowResult> {
		const did = this.registeredDid(role);
		return this.call(role, "query_agent_profile", { did }, signal);
	}
	async createTask(title: string, description: string, providerDid: string, signal?: AbortSignal): Promise<WorkflowResult> {
		const creatorDid = this.registeredDid("agent_a");
		this.agreement = new LiveSettlementAgreement(creatorDid, requiredText(providerDid, "provider DID"), this.env);
		this.agreementIdentity = this.agreement.identity();
		const payload = this.agreement.taskPayload(requiredText(title, "title"), requiredText(description, "description"));
		const result = await this.call("agent_a", "create_task", { payload }, signal);
		this.taskId = validateTaskResult(result, undefined, "submitted", "task creation");
		return result;
	}
	async acceptTask(signal?: AbortSignal): Promise<WorkflowResult> {
		this.registeredDid("agent_b");
		const taskId = this.createdTaskId();
		const payload = LiveSettlementAgreement.taskOperationPayload(this.transitionAgreement(), "accept");
		const result = await this.call("agent_b", "accept_task", { task_id: taskId, payload }, signal);
		validateTaskResult(result, taskId, "accepted", "task acceptance");
		return result;
	}
	async completeTask(signal?: AbortSignal): Promise<WorkflowResult> {
		this.registeredDid("agent_b");
		const taskId = this.createdTaskId();
		const payload = LiveSettlementAgreement.taskOperationPayload(this.transitionAgreement(), "complete");
		const result = await this.call("agent_b", "complete_task", { task_id: taskId, payload }, signal);
		validateTaskResult(result, taskId, "completed", "task completion");
		return result;
	}
	async waitForEscrowFunding(signal?: AbortSignal): Promise<WorkflowResult> {
		this.registeredDid("agent_b");
		const taskId = this.createdTaskId();
		return waitForEscrowProjection(
			() => this.call("agent_b", "query_participant_task_projection", { task_id: taskId }, signal), signal,
		);
	}
	async fundEscrow(signal?: AbortSignal): Promise<WorkflowResult> {
		this.registeredDid("agent_a");
		const payload = this.currentAgreement().fundPayload(this.createdTaskId());
		const result = await this.call("agent_a", "fund_escrow", { payload }, signal);
		this.escrowId = validateEscrowResult(result, undefined, "funded", "escrow funding");
		return result;
	}
	async releaseEscrow(signal?: AbortSignal): Promise<WorkflowResult> {
		this.registeredDid("agent_a");
		const escrowId = this.fundedEscrowId();
		const payload = this.currentAgreement().releasePayload();
		const result = await reconcileAmbiguousRelease(
			() => this.call("agent_a", "release_escrow", { escrow_id: escrowId, payload }, signal), signal,
		);
		validateEscrowResult(result, escrowId, "release-authorized", "escrow release");
		return result;
	}
	async queryParticipantProjection(role: "agent_a" | "agent_b", signal?: AbortSignal): Promise<WorkflowResult> {
		this.registeredDid(role);
		const taskId = this.createdTaskId();
		const result = await waitForFinalProjection(
			() => this.call(role, "query_participant_task_projection", { task_id: taskId }, signal), signal,
		);
		validateProjection(result, taskId, "participant-private", `${agentLabel(role)} participant projection`);
		this.projections[role] = result;
		return result;
	}
	async queryVerifierProjection(signal?: AbortSignal): Promise<WorkflowResult> {
		this.registeredDid("agent_c");
		const taskId = this.createdTaskId();
		const result = await waitForFinalProjection(
			() => this.call("agent_c", "query_verifier_task_projection", { task_id: taskId }, signal), signal,
		);
		validateProjection(result, taskId, "restricted-public", "Agent C verifier projection");
		this.projections.agent_c = result;
		return result;
	}
	provenance(role: AgentRole): McpSessionProvenance {
		return this.session(role).provenance();
	}
	actorEvidence(role: AgentRole, piProcessId: number, handoffDigest: string): Record<string, unknown> {
		const did = this.registeredDid(role);
		const projection = this.projections[role];
		if (!projection) throw new Error(`${agentLabel(role)} final runtime projection is missing`);
		return buildActorEvidence(role, piProcessId, did, this.provenance(role), projection, handoffDigest);
	}
	async queryTask(role: AgentRole, signal?: AbortSignal): Promise<WorkflowResult> {
		this.registeredDid(role);
		const taskId = this.createdTaskId();
		const result = await this.call(role, "query_task", { task_id: taskId }, signal);
		validateTaskResult(result, taskId, "accepted", `${agentLabel(role)} task query`);
		this.observations[role] = result;
		return result;
	}
	importTask(taskId: string, agreement?: AgreementIdentity) {
		const imported = validImportedTaskId(taskId);
		if (this.taskId && this.taskId !== imported) throw new Error("Imported task ID conflicts with current task");
		if (agreement && this.registeredDid("agent_b") !== agreement.provider_did) throw new Error("Imported agreement provider differs from registered Agent B");
		this.taskId = imported;
		this.agreementIdentity = agreement ?? this.agreementIdentity;
	}
	taskHandoff() {
		return { task_id: this.createdTaskId(), ...this.transitionAgreement() };
	}
	currentTaskId(): string {
		return this.createdTaskId();
	}
	acceptedObservation(role: AgentRole): WorkflowResult {
		const observation = this.observations[role];
		if (!observation) throw new Error(`${agentLabel(role)} has not observed accepted task state`);
		return observation;
	}
	async waitForAccepted(role: AgentRole, options: WaitOptions, signal?: AbortSignal): Promise<WorkflowResult> {
		validateWaitOptions(options);
		this.registeredDid(role);
		const taskId = this.createdTaskId();
		const call = () => this.call(role, "query_task", { task_id: taskId }, signal);
		const result = await waitForTaskState(call, "accepted", ["submitted"], options, signal);
		validateTaskProjection(result, taskId, `${agentLabel(role)} task acceptance wait`);
		this.observations[role] = result;
		return result;
	}
	async waitForCompleted(role: "agent_a", options: WaitOptions, signal?: AbortSignal): Promise<WorkflowResult> {
		validateWaitOptions(options);
		this.registeredDid(role);
		const taskId = this.createdTaskId();
		const call = () => this.call(role, "query_task", { task_id: taskId }, signal);
		const result = await waitForTaskState(call, "completed", ["accepted"], options, signal);
		validateTaskProjection(result, taskId, "Agent A task completion wait");
		return result;
	}
	async shutdown(): Promise<void> {
		await Promise.all(Object.values(this.agents).map((agent) => agent.session?.shutdown()));
	}
	private session(role: AgentRole): McpSession {
		const state = this.agents[role];
		state.session ??= new McpSession(readLiveMcpConfig(configRole(role), this.env, this.cwd));
		return state.session;
	}
	private call(role: AgentRole, tool: string, fields: Record<string, unknown>, signal?: AbortSignal) {
		return callWithAntiSpamBackoff(this.session(role), tool, fields, signal);
	}
	private registeredDid(role: AgentRole): string {
		const did = this.agents[role].did;
		if (!did) throw new Error(`Register ${agentLabel(role)} before continuing`);
		return did;
	}
	private createdTaskId(): string {
		if (!this.taskId) throw new Error("Create a task before continuing");
		return this.taskId;
	}
	private fundedEscrowId(): string {
		if (!this.escrowId) throw new Error("Fund escrow before release");
		return this.escrowId;
	}
	private currentAgreement(): LiveSettlementAgreement {
		if (!this.agreement) throw new Error("Create a canonical settlement agreement before continuing");
		return this.agreement;
	}
	private transitionAgreement(): AgreementIdentity {
		if (!this.agreementIdentity) throw new Error("Import the canonical settlement agreement before continuing");
		return this.agreementIdentity;
	}
	private assertDistinctDid(role: AgentRole, did: string) {
		const duplicate = Object.entries(this.agents).some(([otherRole, state]) => otherRole !== role && state.did === did);
		if (duplicate) throw new Error("Agent A, Agent B, and Agent C registration DIDs must be distinct");
	}
}
