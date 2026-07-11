import { McpSession, readLiveMcpConfig, type McpSessionProvenance } from "./mcp-session.ts";
import {
	agentLabel, configRole, delay, requiredString, requiredText, validImportedTaskId,
	validateEscrowResult, validateProjection, validateTaskProjection, validateTaskResult, validateWaitOptions,
} from "./live-task-workflow-support.ts";

export type AgentRole = "agent_a" | "agent_b" | "agent_c";
export type WorkflowResult = Record<string, unknown>;
type Environment = Record<string, string | undefined>;
type AgentState = { did?: string; session?: McpSession };
type WaitOptions = { timeoutMs: number; pollMs: number };

export class LiveTaskWorkflow {
	private readonly agents: Record<AgentRole, AgentState> = { agent_a: {}, agent_b: {}, agent_c: {} };
	private readonly observations: Partial<Record<AgentRole, WorkflowResult>> = {};
	private taskId?: string;
	private escrowId?: string;
	private readonly env: Environment;
	private readonly cwd: string;
	constructor(env: Environment, cwd: string) {
		this.env = env;
		this.cwd = cwd;
	}
	async register(role: AgentRole, signal?: AbortSignal): Promise<WorkflowResult> {
		const result = await this.session(role).call("register", {}, signal);
		const did = requiredString(result, "did", `${agentLabel(role)} registration`);
		this.assertDistinctDid(role, did);
		this.agents[role].did = did;
		return result;
	}
	async queryProfile(role: AgentRole, signal?: AbortSignal): Promise<WorkflowResult> {
		const did = this.registeredDid(role);
		return this.session(role).call("query_agent_profile", { did }, signal);
	}
	async createTask(title: string, description: string, signal?: AbortSignal): Promise<WorkflowResult> {
		this.registeredDid("agent_a");
		const payload = JSON.stringify({ title: requiredText(title, "title"), description: requiredText(description, "description") });
		const result = await this.session("agent_a").call("create_task", { payload }, signal);
		this.taskId = validateTaskResult(result, undefined, "submitted", "task creation");
		return result;
	}
	async acceptTask(signal?: AbortSignal): Promise<WorkflowResult> {
		this.registeredDid("agent_b");
		const taskId = this.createdTaskId();
		const result = await this.session("agent_b").call("accept_task", { task_id: taskId }, signal);
		validateTaskResult(result, taskId, "accepted", "task acceptance");
		return result;
	}
	async completeTask(signal?: AbortSignal): Promise<WorkflowResult> {
		this.registeredDid("agent_b");
		const taskId = this.createdTaskId();
		const result = await this.session("agent_b").call("complete_task", { task_id: taskId }, signal);
		validateTaskResult(result, taskId, "completed", "task completion");
		return result;
	}
	async fundEscrow(payload: string, signal?: AbortSignal): Promise<WorkflowResult> {
		this.registeredDid("agent_a");
		const result = await this.session("agent_a").call("fund_escrow", { payload: requiredText(payload, "escrow payload") }, signal);
		this.escrowId = validateEscrowResult(result, undefined, "funded", "escrow funding");
		return result;
	}
	async releaseEscrow(signal?: AbortSignal): Promise<WorkflowResult> {
		this.registeredDid("agent_a");
		const escrowId = this.fundedEscrowId();
		const result = await this.session("agent_a").call("release_escrow", { escrow_id: escrowId }, signal);
		validateEscrowResult(result, escrowId, "released", "escrow release");
		return result;
	}
	async queryParticipantProjection(role: "agent_a" | "agent_b", signal?: AbortSignal): Promise<WorkflowResult> {
		this.registeredDid(role);
		const taskId = this.createdTaskId();
		const result = await this.session(role).call("query_participant_task_projection", { task_id: taskId }, signal);
		validateProjection(result, taskId, "participant-private", `${agentLabel(role)} participant projection`);
		return result;
	}
	async queryVerifierProjection(signal?: AbortSignal): Promise<WorkflowResult> {
		this.registeredDid("agent_c");
		const taskId = this.createdTaskId();
		const result = await this.session("agent_c").call("query_verifier_task_projection", { task_id: taskId }, signal);
		validateProjection(result, taskId, "restricted-public", "Agent C verifier projection");
		return result;
	}
	provenance(role: AgentRole): McpSessionProvenance {
		return this.session(role).provenance();
	}
	async queryTask(role: AgentRole, signal?: AbortSignal): Promise<WorkflowResult> {
		this.registeredDid(role);
		const taskId = this.createdTaskId();
		const result = await this.session(role).call("query_task", { task_id: taskId }, signal);
		validateTaskResult(result, taskId, "accepted", `${agentLabel(role)} task query`);
		this.observations[role] = result;
		return result;
	}
	importTask(taskId: string) {
		const imported = validImportedTaskId(taskId);
		if (this.taskId && this.taskId !== imported) throw new Error("Imported task ID conflicts with current task");
		this.taskId = imported;
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
		const deadline = Date.now() + options.timeoutMs;
		while (Date.now() <= deadline) {
			if (signal?.aborted) throw new Error(`${agentLabel(role)} task acceptance wait aborted`);
			const result = await this.session(role).call("query_task", { task_id: taskId }, signal);
			const state = validateTaskProjection(result, taskId, `${agentLabel(role)} task acceptance wait`);
			if (state === "accepted") {
				this.observations[role] = result;
				return result;
			}
			if (state !== "submitted") throw new Error(`${agentLabel(role)} task acceptance wait returned unexpected state ${state}`);
			await delay(options.pollMs, signal);
		}
		throw new Error(`${agentLabel(role)} task acceptance wait timed out`);
	}
	async shutdown(): Promise<void> {
		await Promise.all(Object.values(this.agents).map((agent) => agent.session?.shutdown()));
	}
	private session(role: AgentRole): McpSession {
		const state = this.agents[role];
		state.session ??= new McpSession(readLiveMcpConfig(configRole(role), this.env, this.cwd));
		return state.session;
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
	private assertDistinctDid(role: AgentRole, did: string) {
		const duplicate = Object.entries(this.agents).some(([otherRole, state]) => otherRole !== role && state.did === did);
		if (duplicate) throw new Error("Agent A, Agent B, and Agent C registration DIDs must be distinct");
	}
}
