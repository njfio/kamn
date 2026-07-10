import { McpSession, readLiveMcpConfig, type LiveMcpAgent } from "./mcp-session.ts";

export type AgentRole = "agent_a" | "agent_b";
export type WorkflowResult = Record<string, unknown>;
type Environment = Record<string, string | undefined>;
type AgentState = { did?: string; session?: McpSession };

export class LiveTaskWorkflow {
	private readonly agents: Record<AgentRole, AgentState> = { agent_a: {}, agent_b: {} };
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
	const state = requiredString(result, "state", step);
	if (expectedId && taskId !== expectedId) throw new Error(`${step} returned a different task ID`);
	if (state !== expectedState) throw new Error(`${step} returned state ${state}; expected ${expectedState}`);
	return taskId;
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
