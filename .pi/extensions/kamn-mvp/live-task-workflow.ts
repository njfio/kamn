import { McpSession, readLiveMcpConfig, type LiveMcpAgent } from "./mcp-session.ts";

export type AgentRole = "agent_a" | "agent_b";
export type WorkflowResult = Record<string, unknown>;
type Environment = Record<string, string | undefined>;
type AgentState = { did?: string; session?: McpSession };
type WaitOptions = { timeoutMs: number; pollMs: number };

export class LiveTaskWorkflow {
	private readonly agents: Record<AgentRole, AgentState> = { agent_a: {}, agent_b: {} };
	private readonly observations: Partial<Record<AgentRole, WorkflowResult>> = {};
	private taskId?: string;
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
	private assertDistinctDid(role: AgentRole, did: string) {
		const other = role === "agent_a" ? "agent_b" : "agent_a";
		if (this.agents[other].did === did) throw new Error("Agent A and Agent B registration DIDs must be distinct");
	}
}

function validateTaskResult(result: WorkflowResult, expectedId: string | undefined, expectedState: string, step: string): string {
	const taskId = requiredString(result, "task_id", step);
	const state = validateTaskProjection(result, expectedId, step);
	if (state !== expectedState) throw new Error(`${step} returned state ${state}; expected ${expectedState}`);
	return taskId;
}
function validateTaskProjection(result: WorkflowResult, expectedId: string | undefined, step: string): string {
	const taskId = requiredString(result, "task_id", step);
	if (expectedId && taskId !== expectedId) throw new Error(`${step} returned a different task ID`);
	return requiredString(result, "state", step);
}
function requiredString(result: WorkflowResult, field: string, step: string): string {
	const value = result[field];
	if (typeof value !== "string" || !value.trim()) throw new Error(`${step} omitted ${field}`);
	return value;
}
function requiredText(value: string, field: string): string {
	const trimmed = value.trim();
	if (!trimmed) throw new Error(`Task ${field} must not be blank`);
	return trimmed;
}
function configRole(role: AgentRole): LiveMcpAgent {
	return role === "agent_a" ? "AGENT_A" : "AGENT_B";
}
function agentLabel(role: AgentRole): string {
	return role === "agent_a" ? "Agent A" : "Agent B";
}
function validImportedTaskId(taskId: string): string {
	if (!/^[A-Za-z0-9._:-]{1,200}$/.test(taskId)) throw new Error("Imported task ID is invalid");
	return taskId;
}
function delay(milliseconds: number, signal?: AbortSignal): Promise<void> {
	return new Promise((resolveDelay, reject) => {
		const finish = () => { signal?.removeEventListener("abort", abort); resolveDelay(); };
		const timer = setTimeout(finish, milliseconds);
		const abort = () => {
			clearTimeout(timer);
			signal?.removeEventListener("abort", abort);
			reject(new Error("Task acceptance wait aborted"));
		};
		signal?.addEventListener("abort", abort, { once: true });
	});
}
function validateWaitOptions(options: WaitOptions) {
	if (!Number.isInteger(options.timeoutMs) || options.timeoutMs <= 0) throw new Error("Task acceptance timeout must be positive");
	if (!Number.isInteger(options.pollMs) || options.pollMs <= 0) throw new Error("Task acceptance poll interval must be positive");
}
