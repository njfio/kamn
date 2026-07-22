import type { LiveMcpAgent } from "./mcp-session.ts";
import type { McpSessionProvenance } from "./mcp-session.ts";
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
	requiredDigest(result, "receipt_chain_commitment", step);
	requiredDigest(result, "public_commitment", step);
}
export function buildActorEvidence(
	role: AgentRole,
	piProcessId: number,
	did: string,
	provenance: McpSessionProvenance,
	projection: WorkflowResult,
	handoffDigest: string,
) {
	validateProjectedReceipts(role, provenance, projection);
	return {
		...actorProvenance(role, piProcessId, did, provenance),
		...sharedProjectionFields(projection),
		view_scope: requiredString(projection, "view_scope", `${agentLabel(role)} projection`),
		...(role === "agent_c" ? {} : {
			participant_role: requiredString(projection, "participant_role", `${agentLabel(role)} projection`),
		}),
		source_handoff_digest: handoffDigest,
		handoff_authorized: false,
	};
}
function actorProvenance(role: AgentRole, piProcessId: number, did: string, provenance: McpSessionProvenance) {
	return {
		actor: role,
		pi_process_id: piProcessId,
		did,
		mcp_child_process_id: provenance.child_process_id,
		first_request_id: provenance.first_request_id,
		last_request_id: provenance.last_request_id,
		transport_response_digests: provenance.transport_response_digests,
		service_profile_commitment: provenance.service_profile_commitment,
		service_receipts: provenance.service_authority_receipts,
	};
}
function sharedProjectionFields(projection: WorkflowResult) {
	return {
		task_id: requiredString(projection, "task_id", "runtime projection"),
		transaction_id: requiredString(projection, "transaction_id", "runtime projection"),
		escrow_id: requiredString(projection, "escrow_id", "runtime projection"),
		amount_lamports: requiredPositiveNumber(projection, "amount_lamports"),
		network: requiredString(projection, "network", "runtime projection"),
		settlement_tx_signature: requiredString(projection, "settlement_tx_signature", "runtime projection"),
		settlement_commitment: requiredString(projection, "settlement_commitment", "runtime projection"),
		receipt_chain_commitment: requiredDigest(projection, "receipt_chain_commitment", "runtime projection"),
		public_commitment: requiredDigest(projection, "public_commitment", "runtime projection"),
	};
}
function validateProjectedReceipts(role: AgentRole, provenance: McpSessionProvenance, projection: WorkflowResult) {
	if (role === "agent_c") {
		if (projection.receipt_chain_receipts !== undefined) throw new Error("Agent C projection leaked private receipt authority");
		return;
	}
	if (!Array.isArray(projection.receipt_chain_receipts)) throw new Error(`${agentLabel(role)} projection omitted receipt_chain_receipts`);
	const projected = projection.receipt_chain_receipts.map((entry) => projectedReceipt(entry, role));
	const authoritative = provenance.service_authority_receipts.map((receipt) => ({
		receipt_id: receipt.service_receipt_id, receipt_digest: receipt.service_receipt_digest,
		action: receipt.action, resource_id: receipt.resource_id, resulting_state: receipt.resulting_state,
	}));
	if (JSON.stringify(projected) !== JSON.stringify(authoritative)) throw new Error(`${agentLabel(role)} projection receipt authority mismatch`);
}
function projectedReceipt(value: unknown, role: AgentRole) {
	if (typeof value !== "object" || value === null || Array.isArray(value)) throw new Error(`${agentLabel(role)} projection receipt is invalid`);
	const receipt = value as WorkflowResult;
	return {
		receipt_id: requiredString(receipt, "receipt_id", "projection receipt"),
		receipt_digest: requiredDigest(receipt, "receipt_digest", "projection receipt"),
		action: requiredString(receipt, "action", "projection receipt"),
		resource_id: requiredString(receipt, "resource_id", "projection receipt"),
		resulting_state: requiredString(receipt, "resulting_state", "projection receipt"),
	};
}
function requiredDigest(result: WorkflowResult, field: string, step: string): string {
	const value = requiredString(result, field, step);
	if (/^sha256:[0-9a-f]{64}$/.test(value)) return value;
	throw new Error(`${step} returned invalid ${field}`);
}
function requiredPositiveNumber(result: WorkflowResult, field: string): number {
	const value = result[field];
	if (typeof value === "number" && Number.isSafeInteger(value) && value > 0) return value;
	throw new Error(`runtime projection omitted ${field}`);
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
