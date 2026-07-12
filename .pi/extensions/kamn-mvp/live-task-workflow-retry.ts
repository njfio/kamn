import { delay } from "./live-task-workflow-support.ts";

type WorkflowResult = Record<string, unknown>;
type WaitOptions = { timeoutMs: number; pollMs: number };
const MAX_ATTEMPTS = 3;
const RETRY_DELAY_MS = 1000;

export async function reconcileAmbiguousRelease(
	call: () => Promise<WorkflowResult>,
	signal?: AbortSignal,
): Promise<WorkflowResult> {
	for (let attempt = 1; attempt <= MAX_ATTEMPTS; attempt += 1) {
		try { return await call(); } catch (error) {
			if (!isAmbiguousSettlement(error) || attempt === MAX_ATTEMPTS) throw error;
			await delay(RETRY_DELAY_MS, signal);
		}
	}
	throw new Error("Settlement reconciliation attempts exhausted");
}

export async function waitForFinalProjection(
	call: () => Promise<WorkflowResult>,
	signal?: AbortSignal,
): Promise<WorkflowResult> {
	for (let attempt = 1; attempt <= MAX_ATTEMPTS; attempt += 1) {
		const result = await call();
		if (isFinalizedProjection(result)) return result;
		if (attempt < MAX_ATTEMPTS) await delay(RETRY_DELAY_MS, signal);
	}
	throw new Error("Finalized settlement projection was not available");
}

export async function waitForEscrowProjection(
	call: () => Promise<WorkflowResult>,
	signal?: AbortSignal,
): Promise<WorkflowResult> {
	for (let attempt = 1; attempt <= MAX_ATTEMPTS; attempt += 1) {
		try {
			const result = await call();
			if (typeof result.escrow_id === "string" && result.escrow_id.length > 0) return result;
		} catch (error) {
			if (!isMissingEscrowBinding(error)) throw error;
		}
		if (attempt < MAX_ATTEMPTS) await delay(RETRY_DELAY_MS, signal);
	}
	throw new Error("Task-bound escrow projection was not available");
}

export async function waitForTaskState(
	call: () => Promise<WorkflowResult>,
	expectedState: string,
	pendingStates: string[],
	options: WaitOptions,
	signal?: AbortSignal,
): Promise<WorkflowResult> {
	const deadline = Date.now() + options.timeoutMs;
	while (Date.now() <= deadline) {
		if (signal?.aborted) throw new Error("Task state wait aborted");
		const result = await call();
		if (result.state === expectedState) return result;
		if (!pendingStates.includes(String(result.state))) throw new Error(`Task state wait returned unexpected state ${result.state}`);
		await delay(options.pollMs, signal);
	}
	throw new Error(`Task ${expectedState} wait timed out`);
}

function isAmbiguousSettlement(error: unknown): boolean {
	return error instanceof Error && error.message.includes("SETTLEMENT_OUTCOME_AMBIGUOUS");
}

function isMissingEscrowBinding(error: unknown): boolean {
	return error instanceof Error && error.message.includes("TASK_ESCROW_BINDING_MISSING");
}

function isFinalizedProjection(result: WorkflowResult): boolean {
	return typeof result.settlement_tx_signature === "string"
		&& result.settlement_tx_signature.length > 0
		&& result.settlement_commitment === "finalized";
}
