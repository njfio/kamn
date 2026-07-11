import type { LiveMcpAgent } from "./mcp-session.ts";
import type { AgentRole, WorkflowResult } from "./live-task-workflow.ts";

export function validateTaskResult(result: WorkflowResult, expectedId: string | undefined, expectedState: string, step: string): string {
	const taskId = requiredString(result, "task_id", step);
	const state = validateTaskProjection(result, expectedId, step);
	if (state !== expectedState) throw new Error(`${step} returned state ${state}; expected ${expectedState}`);
	return taskId;
}
export function validateTaskProjection(result: WorkflowResult, expectedId: string | undefined, step: string): string {
	const taskId = requiredString(result, "task_id", step);
	if (expectedId && taskId !== expectedId) throw new Error(`${step} returned a different task ID`);
	return requiredString(result, "state", step);
}
export function validateEscrowResult(result: WorkflowResult, expectedId: string | undefined, expectedState: string, step: string): string {
	const escrowId = requiredString(result, "escrow_id", step);
	if (expectedId && escrowId !== expectedId) throw new Error(`${step} returned a different escrow ID`);
	const state = requiredString(result, "state", step);
	if (state !== expectedState) throw new Error(`${step} returned state ${state}; expected ${expectedState}`);
	return escrowId;
}
export function validateProjection(result: WorkflowResult, taskId: string, scope: string, step: string) {
	if (requiredString(result, "task_id", step) !== taskId) throw new Error(`${step} returned a different task ID`);
	if (requiredString(result, "view_scope", step) !== scope) throw new Error(`${step} returned a different view scope`);
	requiredString(result, "public_commitment", step);
}
export function requiredString(result: WorkflowResult, field: string, step: string): string {
	const value = result[field];
	if (typeof value !== "string" || !value.trim()) throw new Error(`${step} omitted ${field}`);
	return value;
}
export function requiredText(value: string, field: string): string {
	const trimmed = value.trim();
	if (!trimmed) throw new Error(`Task ${field} must not be blank`);
	return trimmed;
}
export function configRole(role: AgentRole): LiveMcpAgent {
	return ({ agent_a: "AGENT_A", agent_b: "AGENT_B", agent_c: "AGENT_C" } as const)[role];
}
export function agentLabel(role: AgentRole): string {
	return ({ agent_a: "Agent A", agent_b: "Agent B", agent_c: "Agent C" } as const)[role];
}
export function validImportedTaskId(taskId: string): string {
	if (!/^[A-Za-z0-9._:-]{1,200}$/.test(taskId)) throw new Error("Imported task ID is invalid");
	return taskId;
}
export function delay(milliseconds: number, signal?: AbortSignal): Promise<void> {
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
export function validateWaitOptions(options: { timeoutMs: number; pollMs: number }) {
	if (!Number.isInteger(options.timeoutMs) || options.timeoutMs <= 0) throw new Error("Task acceptance timeout must be positive");
	if (!Number.isInteger(options.pollMs) || options.pollMs <= 0) throw new Error("Task acceptance poll interval must be positive");
}
