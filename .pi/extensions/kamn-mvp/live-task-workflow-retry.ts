import { delay } from "./live-task-workflow-support.ts";

type WorkflowResult = Record<string, unknown>;
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

function isAmbiguousSettlement(error: unknown): boolean {
	return error instanceof Error && error.message.includes("SETTLEMENT_OUTCOME_AMBIGUOUS");
}

function isFinalizedProjection(result: WorkflowResult): boolean {
	return typeof result.settlement_tx_signature === "string"
		&& result.settlement_tx_signature.length > 0
		&& result.settlement_commitment === "finalized";
}
